//! Portfolio-level rollups derived from `transactions` + `ohlcv_daily`.
//!
//! [`value_series`] returns one row per calendar day in a lookback
//! window, with the user's total market value and cost basis as of
//! that day. The computation is fully derived — there's no
//! `portfolio_snapshots` table — so adding a backfilled transaction
//! retroactively fixes the series on the next request.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use jiff::civil::Date;
use jiff::ToSpan;
use rust_decimal::Decimal;

use plutus_core::cost_basis::{compute_position, CostBasisMethod, TxLot};
use plutus_core::transaction::TransactionKind;

use crate::db::{Db, DbError, Result};

/// One day's portfolio rollup.
#[derive(Debug, Clone)]
pub struct DailyValue {
    /// ISO date `YYYY-MM-DD` (the day this snapshot is taken for).
    pub date: String,
    /// Sum of `quantity * close_on_that_day` across every open position
    /// on that date. Uses adjusted close when present, falls back to
    /// raw close. Missing prices (weekends, before-IPO) carry forward
    /// the last known close; days with no prior close skip the
    /// contribution entirely.
    pub market_value: Decimal,
    /// Sum of `Position::cost_base` for every still-open position on
    /// that date, from the cost-basis FIFO rollup.
    pub cost_basis: Decimal,
}

/// Compute the per-day portfolio time series for a user over the last
/// `days` calendar days (inclusive of today). Implementation runs in
/// `O(stocks × days)` after the constant-time DB fetches — fine for
/// the current account sizes.
pub async fn value_series(
    db: &Db,
    user_id: i64,
    days: i64,
) -> Result<Vec<DailyValue>> {
    if days <= 0 {
        return Ok(Vec::new());
    }
    let txs = super::transactions::list(db, user_id).await?;
    if txs.is_empty() {
        return Ok(Vec::new());
    }

    // Window: today (server-side) − (days-1) ... today.
    let today = jiff::Zoned::now().date();
    let start = today
        .checked_sub(((days - 1) as i64).days())
        .map_err(|e| DbError::Validation(format!("date math: {e}")))?;

    // Collect the unique stock_ids appearing in transactions. We only
    // need OHLCV for those — everything else in the catalog is
    // irrelevant to this user's history.
    let stock_ids: Vec<i64> = txs
        .iter()
        .filter_map(|t| t.stock_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    if stock_ids.is_empty() {
        // User only has cash transactions (dividends / deposits) — no
        // marketable positions to rollup. Series is empty.
        return Ok(Vec::new());
    }

    // Pull OHLCV for those stocks in one query. `ohlcv_daily.trade_date`
    // is text (ISO YYYY-MM-DD), sortable lexicographically.
    let client = db.raw_client().await?;
    let rows = client
        .query(
            "SELECT stock_id, trade_date, close, adjusted_close \
               FROM ohlcv_daily \
              WHERE stock_id = ANY($1) \
              ORDER BY stock_id ASC, trade_date ASC",
            &[&stock_ids],
        )
        .await
        .map_err(DbError::from)?;

    // Group into a per-stock vector of (date, close) sorted ascending.
    // Use adjusted close when present (correct for splits/dividends),
    // raw close otherwise.
    let mut prices: HashMap<i64, Vec<(String, Decimal)>> = HashMap::new();
    for row in &rows {
        let sid: i64 = row.get("stock_id");
        let date: String = row.get("trade_date");
        let adj: Option<Decimal> = row.get("adjusted_close");
        let close: Decimal = row.get("close");
        prices.entry(sid).or_default().push((date, adj.unwrap_or(close)));
    }

    // Pre-group transactions by stock_id so the per-day loop only
    // touches the right rows.
    let mut txs_by_stock: HashMap<i64, Vec<TxLot>> = HashMap::new();
    let mut tx_dates: HashMap<i64, Vec<Date>> = HashMap::new();
    for tx in &txs {
        let Some(sid) = tx.stock_id else { continue };
        let Ok(kind) = TransactionKind::from_str(&tx.kind) else {
            continue;
        };
        let date = tx.executed_at.to_zoned(jiff::tz::TimeZone::UTC).date();
        txs_by_stock.entry(sid).or_default().push(TxLot {
            kind,
            quantity: tx.quantity,
            price: tx.price,
            commission_base: tx.commission * tx.fx_rate_to_base,
            fx_to_base: tx.fx_rate_to_base,
            sort_key: tx.executed_at.as_nanosecond() as i64,
        });
        tx_dates.entry(sid).or_default().push(date);
    }

    let mut series = Vec::with_capacity(days as usize);
    let mut cursor = start;
    while cursor <= today {
        let date_str = cursor.to_string(); // ISO YYYY-MM-DD
        let mut market_value = Decimal::ZERO;
        let mut cost_basis = Decimal::ZERO;

        for (&sid, lots) in &txs_by_stock {
            // Filter the lots to those whose executed_at <= cursor.
            // Compare via the parallel tx_dates vector — cheap and
            // avoids re-parsing timestamps.
            let dates = tx_dates.get(&sid).map(|v| v.as_slice()).unwrap_or(&[]);
            let filtered: Vec<TxLot> = lots
                .iter()
                .zip(dates.iter())
                .filter(|(_, d)| **d <= cursor)
                .map(|(l, _)| l.clone())
                .collect();
            if filtered.is_empty() {
                continue;
            }
            let pos = compute_position(&filtered, CostBasisMethod::Fifo);
            if pos.quantity == Decimal::ZERO {
                continue;
            }
            cost_basis += pos.cost_base;

            // Carry-forward price lookup: latest `(date, close)` with
            // date <= cursor for this stock. Binary search by string —
            // ISO dates are lexicographically sortable.
            if let Some(close) = latest_close_on_or_before(
                prices.get(&sid).map(|v| v.as_slice()).unwrap_or(&[]),
                &date_str,
            ) {
                market_value += pos.quantity * close;
            } else {
                // No price ever recorded for this stock — fall back to
                // cost basis so we don't artificially zero a held
                // position. Conservative but stable.
                market_value += pos.cost_base;
            }
        }

        series.push(DailyValue {
            date: date_str,
            market_value,
            cost_basis,
        });
        cursor = cursor
            .checked_add(1.day())
            .map_err(|e| DbError::Validation(format!("date math: {e}")))?;
    }

    Ok(series)
}

/// Return the most recent close on or before `target_date`, or `None`
/// if `prices` is empty / starts after `target_date`. `prices` must be
/// sorted ascending by date (we sort in SQL).
fn latest_close_on_or_before(
    prices: &[(String, Decimal)],
    target_date: &str,
) -> Option<Decimal> {
    if prices.is_empty() {
        return None;
    }
    // partition_point returns the first index where the predicate is
    // false. With `d <= target`, that's the count of dates <= target.
    let idx = prices.partition_point(|(d, _)| d.as_str() <= target_date);
    if idx == 0 {
        None
    } else {
        Some(prices[idx - 1].1)
    }
}
