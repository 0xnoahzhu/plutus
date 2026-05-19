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
    Ok(group_and_compute(&txs, method))
}

pub async fn compute_for_account(
    db: &Db,
    user_id: i64,
    account_id: i64,
    method: CostBasisMethod,
) -> Result<Vec<Holding>> {
    let txs = super::transactions::list_for_account(db, user_id, account_id).await?;
    Ok(group_and_compute(&txs, method))
}

fn group_and_compute(txs: &[Transaction], method: CostBasisMethod) -> Vec<Holding> {
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
        .map(|(stock_id, lots)| Holding {
            stock_id,
            account_id: None,
            position: compute_position(&lots, method),
        })
        .filter(|h| {
            h.position.quantity != Decimal::ZERO || h.position.realized_pnl_base != Decimal::ZERO
        })
        .collect()
}
