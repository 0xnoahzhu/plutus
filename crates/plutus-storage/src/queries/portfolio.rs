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

use plutus_core::cash::{cash_delta_base, CashLot};
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
    /// Cash held that day, across every account.
    ///
    /// Derived by walking the anchor to that date: `anchor + F(date) -
    /// F(anchor_as_of)`, where `F` is the running total of ledger cash
    /// flows. The subtraction is what makes days *before* the anchor
    /// work — cash back then was the anchor minus everything that
    /// happened between.
    pub cash: Decimal,
    /// `cash + market_value` — net worth that day.
    pub total_assets: Decimal,
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
    // A user with only cash transactions still has a net worth worth
    // plotting, so we no longer bail here — the price fetch below just
    // finds nothing and the market-value half stays zero.

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

    // Cash inputs: every flow with its instant, plus each account's
    // anchor. Kept as a flat list because the per-day step just needs
    // "sum the flows on one side of a cutoff".
    let accounts = super::accounts::list(db, user_id).await?;
    let mut flows: Vec<(i64, jiff::Timestamp, Decimal)> = Vec::with_capacity(txs.len());
    for tx in &txs {
        let Ok(kind) = TransactionKind::from_str(&tx.kind) else {
            continue;
        };
        flows.push((
            tx.account_id,
            tx.executed_at,
            cash_delta_base(&CashLot {
                kind,
                quantity: tx.quantity,
                price: tx.price,
                commission: tx.commission,
                tax: tx.tax,
                fx_to_base: tx.fx_rate_to_base,
            }),
        ));
    }

    let mut series = Vec::with_capacity(days as usize);
    let mut cursor = start;
    while cursor <= today {
        let date_str = cursor.to_string(); // ISO YYYY-MM-DD
        let mut market_value = Decimal::ZERO;
        let mut cost_basis = Decimal::ZERO;

        // Cash at end-of-day: anchor + F(day) - F(anchor_as_of), where F
        // is the running flow total. Written as one signed pass so the
        // before-anchor case (where the correction is a subtraction)
        // needs no special branch.
        let day_end = cursor
            .checked_add(1.day())
            .map_err(|e| DbError::Validation(format!("date math: {e}")))?
            .to_zoned(jiff::tz::TimeZone::UTC)
            .map_err(|e| DbError::Validation(format!("date math: {e}")))?
            .timestamp();
        let mut cash = Decimal::ZERO;
        for account in &accounts {
            cash += account.cash_balance;
            let anchor = account.cash_as_of;
            for (account_id, at, delta) in &flows {
                if *account_id != account.id {
                    continue;
                }
                let before_day = *at < day_end;
                let after_anchor = anchor.is_none_or(|a| *at > a);
                match (before_day, after_anchor) {
                    // Happened after the anchor and on/before this day:
                    // already reflected in reality by now, add it.
                    (true, true) => cash += delta,
                    // Happened after this day but on/before the anchor:
                    // the anchor includes it, so back it out to see what
                    // cash looked like on the earlier date.
                    (false, false) => cash -= delta,
                    _ => {}
                }
            }
        }

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
            cash,
            total_assets: cash + market_value,
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

// ── Total assets ─────────────────────────────────────────────────────────

/// Cash for one account: the anchor the user set, plus everything the
/// ledger did after it.
#[derive(Debug, Clone)]
pub struct AccountCash {
    pub account_id: i64,
    pub account_name: String,
    pub base_currency: String,
    /// The balance the user pinned, verbatim.
    pub anchor: Decimal,
    /// When the anchor was true. `None` = no anchor, so `flow` covers
    /// the whole ledger.
    pub anchor_as_of: Option<jiff::Timestamp>,
    /// Net cash the ledger moved after `anchor_as_of`.
    pub flow: Decimal,
    /// `anchor + flow` — what's actually on hand.
    pub cash: Decimal,
}

/// Everything needed to answer "what am I worth". Cash sits beside the
/// market value of the open positions; the two add up to total assets.
#[derive(Debug, Clone)]
pub struct PortfolioSummary {
    pub cash: Decimal,
    /// Market value of open positions, base currency.
    pub market_value: Decimal,
    pub cost_basis: Decimal,
    /// `market_value - cost_basis`.
    pub unrealized_pnl: Decimal,
    /// Lifetime realized P&L across every closed leg.
    pub realized_pnl: Decimal,
    /// `cash + market_value`.
    pub total_assets: Decimal,
    /// Open positions counted, and how many of them had no OHLCV bar to
    /// price against. Unpriced positions fall back to cost basis, which
    /// keeps the total stable but means it's an estimate — the caller
    /// should say so when this is non-zero rather than presenting a
    /// number that quietly mixes market and book values.
    pub position_count: i64,
    pub unpriced_count: i64,
    pub accounts: Vec<AccountCash>,
}

/// Round monetary output to cents. The inputs are already exact
/// decimals; this just stops `1234.5600000001`-style tails from
/// weighted-average division reaching the UI.
const MONEY_DP: u32 = 2;

/// Compute cash, market value and total assets for a user.
pub async fn summary(db: &Db, user_id: i64, method: CostBasisMethod) -> Result<PortfolioSummary> {
    let accounts = super::accounts::list(db, user_id).await?;
    let txs = super::transactions::list(db, user_id).await?;

    // ── Cash, per account ────────────────────────────────────────────
    let mut per_account: Vec<AccountCash> = Vec::with_capacity(accounts.len());
    for account in &accounts {
        let flow: Decimal = txs
            .iter()
            .filter(|t| t.account_id == account.id)
            // Flows at or before the anchor are already baked into the
            // balance the user pinned; counting them again would double
            // every trade made before they set it.
            .filter(|t| {
                account
                    .cash_as_of
                    .is_none_or(|anchored_at| t.executed_at > anchored_at)
            })
            .filter_map(|t| {
                let kind = TransactionKind::from_str(&t.kind).ok()?;
                Some(cash_delta_base(&CashLot {
                    kind,
                    quantity: t.quantity,
                    price: t.price,
                    commission: t.commission,
                    tax: t.tax,
                    fx_to_base: t.fx_rate_to_base,
                }))
            })
            .sum();
        per_account.push(AccountCash {
            account_id: account.id,
            account_name: account.name.clone(),
            base_currency: account.base_currency.clone(),
            anchor: account.cash_balance.round_dp(MONEY_DP),
            anchor_as_of: account.cash_as_of,
            flow: flow.round_dp(MONEY_DP),
            cash: (account.cash_balance + flow).round_dp(MONEY_DP),
        });
    }
    // Transactions on an account that no longer exists still moved cash,
    // but there's no anchor or currency to attribute them to, so they're
    // deliberately dropped rather than folded into an arbitrary account.
    let cash: Decimal = per_account.iter().map(|a| a.cash).sum();

    // ── Positions ────────────────────────────────────────────────────
    let holdings = super::holdings::compute_all(db, user_id, method).await?;
    let stock_ids: Vec<i64> = holdings.iter().map(|h| h.stock_id).collect();
    let closes = super::ohlcv::latest_closes(db, &stock_ids).await?;

    let mut market_value = Decimal::ZERO;
    let mut cost_basis = Decimal::ZERO;
    let mut realized_pnl = Decimal::ZERO;
    let mut position_count = 0_i64;
    let mut unpriced_count = 0_i64;
    for h in &holdings {
        // `compute_all` keeps fully-closed positions around so their
        // realized P&L still counts; only open ones carry value.
        realized_pnl += h.position.realized_pnl_base;
        if h.position.quantity == Decimal::ZERO {
            continue;
        }
        position_count += 1;
        cost_basis += h.position.cost_base;
        match closes.get(&h.stock_id) {
            Some(close) => market_value += h.position.quantity * close,
            None => {
                // No bar ever recorded. Falling back to cost basis keeps
                // the total from dropping a real position to zero; the
                // count tells the caller the number is an estimate.
                unpriced_count += 1;
                market_value += h.position.cost_base;
            }
        }
    }

    let market_value = market_value.round_dp(MONEY_DP);
    let cost_basis = cost_basis.round_dp(MONEY_DP);
    Ok(PortfolioSummary {
        cash,
        market_value,
        cost_basis,
        unrealized_pnl: market_value - cost_basis,
        realized_pnl: realized_pnl.round_dp(MONEY_DP),
        total_assets: cash + market_value,
        position_count,
        unpriced_count,
        accounts: per_account,
    })
}
