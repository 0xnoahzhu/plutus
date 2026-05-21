//! Holdings = positions derived from transactions. We delegate the actual
//! cost-basis math to `plutus_core::cost_basis`.

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

use plutus_core::cost_basis::{compute_position, CostBasisMethod, Position, TxLot};
use plutus_core::transaction::TransactionKind;

use crate::db::{Db, Result};
use crate::models::Transaction;

#[derive(Debug, Clone)]
pub struct Holding {
    pub stock_id: i64,
    pub account_id: Option<i64>,
    pub position: Position,
}

pub async fn compute_all(db: &Db, user_id: i64, method: CostBasisMethod) -> Result<Vec<Holding>> {
    let txs = super::transactions::list(db, user_id).await?;
    // When the user happens to have all their transactions under a
    // single account, surface that account_id even though we didn't
    // filter by it — the caller didn't ask for a per-account view but
    // the answer is unambiguous, so paint it on instead of returning
    // `null` and forcing a second lookup. Mixed-account holdings keep
    // `account_id: None` because the answer is genuinely ambiguous.
    let single_account = single_account_id(&txs);
    Ok(group_and_compute(&txs, method, single_account))
}

pub async fn compute_for_account(
    db: &Db,
    user_id: i64,
    account_id: i64,
    method: CostBasisMethod,
) -> Result<Vec<Holding>> {
    let txs = super::transactions::list_for_account(db, user_id, account_id).await?;
    // Account is fixed by the caller — paint it on every row so the UI
    // can deep-link back to /accounts/{id} without an extra round-trip.
    Ok(group_and_compute(&txs, method, Some(account_id)))
}

/// Decimal places we serialize monetary amounts to. The cost-basis math
/// can produce arbitrary precision (it divides at every weighted-avg
/// step); clamp at four places so callers see consistent values across
/// rows without losing meaningful cents. Quantity is left un-rounded
/// because fractional-share corner cases need the precision.
const MONEY_DP: u32 = 4;

fn group_and_compute(
    txs: &[Transaction],
    method: CostBasisMethod,
    account_id: Option<i64>,
) -> Vec<Holding> {
    let mut by_stock: HashMap<i64, Vec<TxLot>> = HashMap::new();
    for tx in txs {
        let Some(stock_id) = tx.stock_id else {
            continue;
        };
        let Ok(kind) = TransactionKind::from_str(&tx.kind) else {
            continue;
        };
        by_stock.entry(stock_id).or_default().push(TxLot {
            kind,
            quantity: tx.quantity,
            price: tx.price,
            commission_base: tx.commission * tx.fx_rate_to_base,
            fx_to_base: tx.fx_rate_to_base,
            sort_key: tx.executed_at.as_nanosecond() as i64,
        });
    }
    by_stock
        .into_iter()
        .map(|(stock_id, lots)| {
            let mut position = compute_position(&lots, method);
            // Normalize monetary fields. Quantity stays full-precision.
            position.avg_cost_trade = position.avg_cost_trade.round_dp(MONEY_DP);
            position.cost_base = position.cost_base.round_dp(MONEY_DP);
            position.realized_pnl_base = position.realized_pnl_base.round_dp(MONEY_DP);
            Holding {
                stock_id,
                account_id,
                position,
            }
        })
        .filter(|h| {
            h.position.quantity != Decimal::ZERO || h.position.realized_pnl_base != Decimal::ZERO
        })
        .collect()
}

/// Returns `Some(account_id)` iff every share-moving transaction in
/// `txs` is on the same account. `None` if mixed or empty.
fn single_account_id(txs: &[Transaction]) -> Option<i64> {
    let mut found: Option<i64> = None;
    for tx in txs {
        if tx.stock_id.is_none() {
            continue;
        }
        match found {
            None => found = Some(tx.account_id),
            Some(prev) if prev == tx.account_id => {}
            Some(_) => return None,
        }
    }
    found
}
