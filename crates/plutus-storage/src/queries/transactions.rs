use std::str::FromStr;

use rust_decimal::Decimal;

use plutus_core::cost_basis::{compute_position, CostBasisMethod, Position, TxLot};
use plutus_core::transaction::TransactionKind;

use crate::db::{Db, DbError, Result};
use crate::models::Transaction;

/// All transactions sorted newest first. `id desc` is the deterministic
/// tie-breaker so transactions executed at the same instant (rare but
/// possible in bulk imports) keep a stable relative order across
/// refreshes — important once pagination lands.
pub async fn list(db: &Db, user_id: i64) -> Result<Vec<Transaction>> {
    let rows = db
        .with(async |d| {
            Transaction::all()
                .order_by((
                    Transaction::fields().executed_at().desc(),
                    Transaction::fields().id().desc(),
                ))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn list_for_account(db: &Db, user_id: i64, account_id: i64) -> Result<Vec<Transaction>> {
    let rows = db
        .with(async |d| {
            Transaction::all()
                .filter(Transaction::fields().account_id().eq(account_id))
                .order_by((
                    Transaction::fields().executed_at().desc(),
                    Transaction::fields().id().desc(),
                ))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn list_for_stock(db: &Db, user_id: i64, stock_id: i64) -> Result<Vec<Transaction>> {
    let rows = db
        .with(async |d| {
            Transaction::all()
                .filter(Transaction::fields().stock_id().eq(Some(stock_id)))
                .order_by((
                    Transaction::fields().executed_at().desc(),
                    Transaction::fields().id().desc(),
                ))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn get(db: &Db, user_id: i64, id: i64) -> Result<Transaction> {
    let row = db
        .with(async |d| Transaction::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

#[derive(Debug, Clone)]
pub struct NewTransaction<'a> {
    pub user_id: i64,
    pub account_id: i64,
    pub stock_id: Option<i64>,
    pub kind: &'a str,
    pub executed_at: jiff::Timestamp,
    pub quantity: Decimal,
    pub price: Decimal,
    pub trade_currency: &'a str,
    pub commission: Decimal,
    pub commission_currency: &'a str,
    pub tax: Decimal,
    pub tax_currency: &'a str,
    pub fx_rate_to_base: Decimal,
    pub external_ref: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub source: &'a str,
    pub source_metadata: Option<&'a str>,
}

pub async fn create(db: &Db, input: NewTransaction<'_>) -> Result<Transaction> {
    let now = jiff::Timestamp::now();
    let user_id = input.user_id;
    let account_id = input.account_id;
    let stock_id = input.stock_id;
    let kind = input.kind.to_string();
    let executed_at = input.executed_at;
    let quantity = input.quantity;
    let price = input.price;
    let trade_currency = input.trade_currency.to_string();
    let commission = input.commission;
    let commission_currency = input.commission_currency.to_string();
    let tax = input.tax;
    let tax_currency = input.tax_currency.to_string();
    let fx_rate_to_base = input.fx_rate_to_base;
    let external_ref = input.external_ref.map(str::to_string);
    let notes = input.notes.map(str::to_string);
    let source = input.source.to_string();
    let source_metadata = input.source_metadata.map(str::to_string);

    let row = db
        .with(async |d| {
            toasty::create!(Transaction {
                user_id: user_id,
                account_id: account_id,
                stock_id: stock_id,
                kind: kind,
                executed_at: executed_at,
                quantity: quantity,
                price: price,
                trade_currency: trade_currency,
                commission: commission,
                commission_currency: commission_currency,
                tax: tax,
                tax_currency: tax_currency,
                fx_rate_to_base: fx_rate_to_base,
                external_ref: external_ref,
                notes: notes,
                source: source,
                source_metadata: source_metadata,
                created_at: now,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

/// Partial update of one ledger row. Every field is `Option`: `None`
/// means "not present in the request, leave alone". The nullable columns
/// use `Option<Option<T>>` so a caller can distinguish "leave alone"
/// (outer `None`) from "clear to NULL" (outer `Some(None)`).
///
/// The ledger was originally append-only — the documented fix for a
/// mistyped row was a compensating entry. Editing is safe now because
/// nothing downstream caches the rollup: `holdings`, the portfolio
/// value series, and the per-stock summary all recompute from this
/// table on every read, so a corrected row is reflected immediately.
/// Compensating entries still work and remain the right tool when the
/// original row was *itself* a real event (a partial fill, a reversal)
/// rather than a typo.
#[derive(Debug, Clone, Default)]
pub struct TransactionPatch<'a> {
    pub account_id: Option<i64>,
    pub stock_id: Option<Option<i64>>,
    pub kind: Option<&'a str>,
    pub executed_at: Option<jiff::Timestamp>,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub trade_currency: Option<&'a str>,
    pub commission: Option<Decimal>,
    pub commission_currency: Option<&'a str>,
    pub tax: Option<Decimal>,
    pub tax_currency: Option<&'a str>,
    pub fx_rate_to_base: Option<Decimal>,
    pub external_ref: Option<Option<&'a str>>,
    pub notes: Option<Option<&'a str>>,
    pub source: Option<&'a str>,
    pub source_metadata: Option<Option<&'a str>>,
}

pub async fn update(
    db: &Db,
    user_id: i64,
    id: i64,
    patch: TransactionPatch<'_>,
) -> Result<Transaction> {
    // `get` doubles as the ownership check — a row belonging to another
    // user surfaces as NotFound before any write is attempted.
    let mut row = get(db, user_id, id).await?;
    let now = jiff::Timestamp::now();
    db.with(async |d| {
        let mut q = row.update();
        if let Some(account_id) = patch.account_id {
            q = q.account_id(account_id);
        }
        if let Some(stock_id) = patch.stock_id {
            q = q.stock_id(stock_id);
        }
        if let Some(kind) = patch.kind {
            q = q.kind(kind.to_string());
        }
        if let Some(executed_at) = patch.executed_at {
            q = q.executed_at(executed_at);
        }
        if let Some(quantity) = patch.quantity {
            q = q.quantity(quantity);
        }
        if let Some(price) = patch.price {
            q = q.price(price);
        }
        if let Some(trade_currency) = patch.trade_currency {
            q = q.trade_currency(trade_currency.to_string());
        }
        if let Some(commission) = patch.commission {
            q = q.commission(commission);
        }
        if let Some(commission_currency) = patch.commission_currency {
            q = q.commission_currency(commission_currency.to_string());
        }
        if let Some(tax) = patch.tax {
            q = q.tax(tax);
        }
        if let Some(tax_currency) = patch.tax_currency {
            q = q.tax_currency(tax_currency.to_string());
        }
        if let Some(fx_rate_to_base) = patch.fx_rate_to_base {
            q = q.fx_rate_to_base(fx_rate_to_base);
        }
        if let Some(external_ref) = patch.external_ref {
            q = q.external_ref(external_ref.map(str::to_string));
        }
        if let Some(notes) = patch.notes {
            q = q.notes(notes.map(str::to_string));
        }
        if let Some(source) = patch.source {
            q = q.source(source.to_string());
        }
        if let Some(source_metadata) = patch.source_metadata {
            q = q.source_metadata(source_metadata.map(str::to_string));
        }
        q.updated_at(now).exec(d).await
    })
    .await?;
    get(db, user_id, id).await
}

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let row = get(db, user_id, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}

/// Decimal places monetary aggregates are rounded to. Matches
/// `holdings::MONEY_DP` so the same position rendered on `/holdings` and
/// on the stock detail page agrees to the cent. Quantities stay
/// full-precision for fractional shares.
const MONEY_DP: u32 = 4;

/// Everything the ledger knows about one stock, rolled up. Derived on
/// read from the same rows `/transactions` returns — there's no summary
/// table.
///
/// The `*_trade` amounts are summed in each row's own `trade_currency`,
/// so they only mean something when `trade_currency` below is `Some`
/// (i.e. every contributing row shares one currency). The `*_base`
/// amounts apply each row's `fx_rate_to_base` first and are therefore
/// always comparable.
#[derive(Debug, Clone)]
pub struct StockSummary {
    pub stock_id: i64,
    /// Echoes the `account_id` filter when the caller scoped the query;
    /// `None` for a cross-account rollup.
    pub account_id: Option<i64>,
    /// Every ledger row touching this stock, including the cash-only
    /// kinds (dividends, fees) that don't move shares.
    pub transaction_count: i64,
    pub buy_count: i64,
    pub sell_count: i64,
    /// Gross shares acquired / disposed across the whole history — not
    /// netted, so a round trip shows up in both.
    pub buy_quantity: Decimal,
    pub sell_quantity: Decimal,
    pub buy_amount_trade: Decimal,
    pub sell_amount_trade: Decimal,
    pub buy_amount_base: Decimal,
    pub sell_amount_base: Decimal,
    /// Lifetime commission / tax across every row, converted to base.
    pub commission_total_base: Decimal,
    pub tax_total_base: Decimal,
    /// Open position under the requested cost-basis method. Identical to
    /// what `/holdings` reports for this stock.
    pub position: Position,
    pub first_executed_at: Option<jiff::Timestamp>,
    pub last_executed_at: Option<jiff::Timestamp>,
    /// The single trade currency shared by every contributing row, or
    /// `None` when the history mixes currencies (the UI then hides the
    /// `*_trade` figures rather than labelling them with a wrong code).
    pub trade_currency: Option<String>,
}

/// Roll the ledger up for one stock. `account_id` narrows to a single
/// account; `None` spans every account the user owns.
pub async fn summarize_for_stock(
    db: &Db,
    user_id: i64,
    stock_id: i64,
    account_id: Option<i64>,
    method: CostBasisMethod,
) -> Result<StockSummary> {
    let rows = list_for_stock(db, user_id, stock_id).await?;
    let rows: Vec<Transaction> = match account_id {
        Some(a) => rows.into_iter().filter(|r| r.account_id == a).collect(),
        None => rows,
    };
    Ok(summarize(stock_id, account_id, &rows, method))
}

fn summarize(
    stock_id: i64,
    account_id: Option<i64>,
    rows: &[Transaction],
    method: CostBasisMethod,
) -> StockSummary {
    let mut summary = StockSummary {
        stock_id,
        account_id,
        transaction_count: rows.len() as i64,
        buy_count: 0,
        sell_count: 0,
        buy_quantity: Decimal::ZERO,
        sell_quantity: Decimal::ZERO,
        buy_amount_trade: Decimal::ZERO,
        sell_amount_trade: Decimal::ZERO,
        buy_amount_base: Decimal::ZERO,
        sell_amount_base: Decimal::ZERO,
        commission_total_base: Decimal::ZERO,
        tax_total_base: Decimal::ZERO,
        position: compute_position(&[], method),
        first_executed_at: None,
        last_executed_at: None,
        trade_currency: None,
    };

    let mut lots: Vec<TxLot> = Vec::with_capacity(rows.len());
    // `None` = haven't seen a row yet; `Some(None)` = seen at least two
    // different codes, so the trade-currency totals are meaningless.
    let mut currency: Option<Option<&str>> = None;

    for tx in rows {
        summary.first_executed_at = Some(match summary.first_executed_at {
            Some(t) if t <= tx.executed_at => t,
            _ => tx.executed_at,
        });
        summary.last_executed_at = Some(match summary.last_executed_at {
            Some(t) if t >= tx.executed_at => t,
            _ => tx.executed_at,
        });
        currency = Some(match currency {
            None => Some(tx.trade_currency.as_str()),
            Some(Some(c)) if c == tx.trade_currency => Some(c),
            _ => None,
        });

        // Commission and tax roll up through the trade row's FX rate.
        // Their own currency columns exist for display fidelity, but the
        // ledger has no per-leg rate, so this is the best available
        // conversion — same assumption `holdings` already makes.
        summary.commission_total_base += tx.commission * tx.fx_rate_to_base;
        summary.tax_total_base += tx.tax * tx.fx_rate_to_base;

        let Ok(kind) = TransactionKind::from_str(&tx.kind) else {
            continue;
        };
        if !kind.moves_shares() {
            continue;
        }
        lots.push(TxLot {
            kind,
            quantity: tx.quantity,
            price: tx.price,
            commission_base: tx.commission * tx.fx_rate_to_base,
            fx_to_base: tx.fx_rate_to_base,
            sort_key: tx.executed_at.as_nanosecond() as i64,
        });

        // A negative quantity is the documented way to write a
        // correcting entry, so classify by effective direction rather
        // than by `kind` alone — matching how cost_basis reads the lot.
        let qty = tx.quantity;
        let sell_like = matches!(kind, TransactionKind::Sell) && qty > Decimal::ZERO
            || qty < Decimal::ZERO;
        let gross = qty.abs();
        let amount_trade = gross * tx.price;
        if sell_like {
            summary.sell_count += 1;
            summary.sell_quantity += gross;
            summary.sell_amount_trade += amount_trade;
            summary.sell_amount_base += amount_trade * tx.fx_rate_to_base;
        } else if qty > Decimal::ZERO {
            summary.buy_count += 1;
            summary.buy_quantity += gross;
            summary.buy_amount_trade += amount_trade;
            summary.buy_amount_base += amount_trade * tx.fx_rate_to_base;
        }
    }

    summary.position = compute_position(&lots, method);
    summary.position.avg_cost_trade = summary.position.avg_cost_trade.round_dp(MONEY_DP);
    summary.position.cost_base = summary.position.cost_base.round_dp(MONEY_DP);
    summary.position.realized_pnl_base = summary.position.realized_pnl_base.round_dp(MONEY_DP);
    summary.buy_amount_trade = summary.buy_amount_trade.round_dp(MONEY_DP);
    summary.sell_amount_trade = summary.sell_amount_trade.round_dp(MONEY_DP);
    summary.buy_amount_base = summary.buy_amount_base.round_dp(MONEY_DP);
    summary.sell_amount_base = summary.sell_amount_base.round_dp(MONEY_DP);
    summary.commission_total_base = summary.commission_total_base.round_dp(MONEY_DP);
    summary.tax_total_base = summary.tax_total_base.round_dp(MONEY_DP);
    summary.trade_currency = currency.flatten().map(str::to_string);
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn ts(seconds: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_second(seconds).unwrap()
    }

    /// Build a share-moving row. `quantity` is signed exactly the way the
    /// API documents it, so a `SELL` is written positive here.
    fn tx(kind: &str, quantity: Decimal, price: Decimal, at: i64) -> Transaction {
        Transaction {
            id: at,
            user_id: 1,
            account_id: 1,
            stock_id: Some(7),
            kind: kind.to_string(),
            executed_at: ts(at),
            quantity,
            price,
            trade_currency: "USD".to_string(),
            commission: Decimal::ZERO,
            commission_currency: "USD".to_string(),
            tax: Decimal::ZERO,
            tax_currency: "USD".to_string(),
            fx_rate_to_base: dec!(1),
            external_ref: None,
            notes: None,
            source: "manual".to_string(),
            source_metadata: None,
            created_at: ts(at),
            updated_at: ts(at),
        }
    }

    fn summarize_fifo(rows: &[Transaction]) -> StockSummary {
        summarize(7, None, rows, CostBasisMethod::Fifo)
    }

    #[test]
    fn empty_history_is_all_zeros() {
        let s = summarize_fifo(&[]);
        assert_eq!(s.transaction_count, 0);
        assert_eq!(s.buy_quantity, Decimal::ZERO);
        assert_eq!(s.position.quantity, Decimal::ZERO);
        assert!(s.first_executed_at.is_none());
        assert!(s.trade_currency.is_none());
    }

    #[test]
    fn gross_quantities_are_not_netted() {
        // Buy 100 @ 10, sell 40 @ 15 — 60 shares still open, but the
        // gross figures report the full 100 in and 40 out.
        let rows = vec![
            tx("BUY", dec!(100), dec!(10), 1),
            tx("SELL", dec!(40), dec!(15), 2),
        ];
        let s = summarize_fifo(&rows);
        assert_eq!(s.buy_quantity, dec!(100));
        assert_eq!(s.sell_quantity, dec!(40));
        assert_eq!(s.buy_amount_trade, dec!(1000));
        assert_eq!(s.sell_amount_trade, dec!(600));
        assert_eq!(s.buy_count, 1);
        assert_eq!(s.sell_count, 1);
        assert_eq!(s.position.quantity, dec!(60));
        assert_eq!(s.position.realized_pnl_base, dec!(200));
    }

    #[test]
    fn negative_quantity_counts_as_a_disposal() {
        // A negative quantity is the documented way to write a correcting
        // entry, so it has to land in the sell column even though the row
        // is still labelled BUY — otherwise the panel would claim shares
        // were acquired.
        let rows = vec![
            tx("BUY", dec!(100), dec!(10), 1),
            tx("BUY", dec!(-30), dec!(12), 2),
        ];
        let s = summarize_fifo(&rows);
        assert_eq!(s.buy_quantity, dec!(100));
        assert_eq!(s.sell_quantity, dec!(30));
        assert_eq!(s.sell_amount_trade, dec!(360));
        assert_eq!(s.position.quantity, dec!(70));
    }

    #[test]
    fn cash_only_rows_count_but_move_no_shares() {
        // A dividend belongs in the row count and its tax rolls up, but
        // it must not inflate the traded quantities.
        let mut dividend = tx("DIVIDEND", dec!(50), dec!(1), 2);
        dividend.tax = dec!(7.5);
        let rows = vec![tx("BUY", dec!(100), dec!(10), 1), dividend];
        let s = summarize_fifo(&rows);
        assert_eq!(s.transaction_count, 2);
        assert_eq!(s.buy_count, 1);
        assert_eq!(s.sell_count, 0);
        assert_eq!(s.buy_quantity, dec!(100));
        assert_eq!(s.tax_total_base, dec!(7.5));
        assert_eq!(s.position.quantity, dec!(100));
    }

    #[test]
    fn commission_and_tax_convert_through_the_row_fx_rate() {
        let mut buy = tx("BUY", dec!(10), dec!(100), 1);
        buy.trade_currency = "HKD".to_string();
        buy.fx_rate_to_base = dec!(0.13);
        buy.commission = dec!(20);
        buy.tax = dec!(5);
        let s = summarize_fifo(&[buy]);
        assert_eq!(s.commission_total_base, dec!(2.6));
        assert_eq!(s.tax_total_base, dec!(0.65));
        // Trade-currency amount stays in HKD; the base amount applies fx.
        assert_eq!(s.buy_amount_trade, dec!(1000));
        assert_eq!(s.buy_amount_base, dec!(130));
        assert_eq!(s.trade_currency.as_deref(), Some("HKD"));
    }

    #[test]
    fn mixed_currencies_suppress_the_trade_currency_label() {
        // Summing USD and HKD amounts into one number is meaningless, so
        // the label goes null and the UI falls back to the base figures.
        let mut hkd = tx("BUY", dec!(10), dec!(100), 2);
        hkd.trade_currency = "HKD".to_string();
        hkd.fx_rate_to_base = dec!(0.13);
        let rows = vec![tx("BUY", dec!(5), dec!(20), 1), hkd];
        let s = summarize_fifo(&rows);
        assert!(s.trade_currency.is_none());
        assert_eq!(s.buy_amount_base, dec!(230));
    }

    #[test]
    fn executed_at_bounds_ignore_row_order() {
        // list_for_stock returns newest first, so the loop must not
        // assume ascending input.
        let rows = vec![
            tx("SELL", dec!(10), dec!(15), 300),
            tx("BUY", dec!(10), dec!(10), 100),
            tx("BUY", dec!(10), dec!(12), 200),
        ];
        let s = summarize_fifo(&rows);
        assert_eq!(s.first_executed_at, Some(ts(100)));
        assert_eq!(s.last_executed_at, Some(ts(300)));
        // FIFO closes the 10-cost lot first: 10 * (15 - 10) = 50.
        assert_eq!(s.position.realized_pnl_base, dec!(50));
    }

    #[test]
    fn unknown_kind_is_skipped_not_fatal() {
        let rows = vec![tx("BUY", dec!(10), dec!(10), 1), tx("NONSENSE", dec!(5), dec!(3), 2)];
        let s = summarize_fifo(&rows);
        assert_eq!(s.transaction_count, 2);
        assert_eq!(s.buy_quantity, dec!(10));
        assert_eq!(s.position.quantity, dec!(10));
    }
}
