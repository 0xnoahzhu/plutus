//! Cost-basis algorithms. Pure functions on slices of `TxLot` produce
//! `Position` aggregates.
//!
//! These intentionally don't depend on the storage layer's row types — callers
//! map their DB rows into `TxLot` first.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::transaction::TransactionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostBasisMethod {
    Fifo,
    Lifo,
    Average,
}

impl Default for CostBasisMethod {
    fn default() -> Self {
        Self::Fifo
    }
}

/// A minimal transaction view sufficient to compute cost basis.
/// Quantities and prices are in the stock's trade currency; `fx_to_base`
/// converts to the account's base currency at execution time.
#[derive(Debug, Clone)]
pub struct TxLot {
    pub kind: TransactionKind,
    pub quantity: Decimal,
    pub price: Decimal,
    pub commission_base: Decimal,
    pub fx_to_base: Decimal,
    /// Monotonic ordering hint (executed_at in epoch nanos, or transaction id).
    pub sort_key: i64,
}

/// Per-stock position summary after applying the chosen method to all lots.
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// Current open quantity (sum of unsold buys).
    pub quantity: Decimal,
    /// Average cost per share in trade currency, for the *open* shares.
    pub avg_cost_trade: Decimal,
    /// Cost basis of open shares in *base* currency.
    pub cost_base: Decimal,
    /// Realized P&L in base currency across all closed lots.
    pub realized_pnl_base: Decimal,
}

impl Position {
    fn empty() -> Self {
        Self {
            quantity: Decimal::ZERO,
            avg_cost_trade: Decimal::ZERO,
            cost_base: Decimal::ZERO,
            realized_pnl_base: Decimal::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
struct OpenLot {
    quantity: Decimal,
    /// Per-share cost in trade currency (incl. proportional commission).
    cost_per_share_trade: Decimal,
    /// Per-share cost in base currency.
    cost_per_share_base: Decimal,
}

/// Compute the open position for a single stock given its lots in any order.
/// Returns `Position::empty()` if no share-moving lots are present.
pub fn compute_position(lots: &[TxLot], method: CostBasisMethod) -> Position {
    let mut share_lots: Vec<&TxLot> = lots.iter().filter(|l| l.kind.moves_shares()).collect();
    share_lots.sort_by_key(|l| l.sort_key);
    if share_lots.is_empty() {
        return Position::empty();
    }

    match method {
        CostBasisMethod::Fifo | CostBasisMethod::Lifo => {
            apply_lots_fifo_or_lifo(&share_lots, method == CostBasisMethod::Lifo)
        }
        CostBasisMethod::Average => apply_lots_average(&share_lots),
    }
}

fn apply_lots_fifo_or_lifo(share_lots: &[&TxLot], lifo: bool) -> Position {
    let mut open: Vec<OpenLot> = Vec::new();
    let mut realized = Decimal::ZERO;

    for lot in share_lots {
        let qty = lot.quantity;
        if qty == Decimal::ZERO {
            continue;
        }
        let buy_like = matches!(lot.kind, TransactionKind::Buy | TransactionKind::CorporateAction)
            && qty > Decimal::ZERO;
        let sell_like = matches!(lot.kind, TransactionKind::Sell) && qty > Decimal::ZERO
            || qty < Decimal::ZERO;

        if buy_like {
            // Distribute commission across the lot.
            let qty_abs = qty.abs();
            let commission_per_share = if qty_abs == Decimal::ZERO {
                Decimal::ZERO
            } else {
                lot.commission_base / qty_abs
            };
            let trade_cost = lot.price;
            let base_cost = lot.price * lot.fx_to_base
                + commission_per_share;
            open.push(OpenLot {
                quantity: qty_abs,
                cost_per_share_trade: trade_cost,
                cost_per_share_base: base_cost,
            });
        } else if sell_like {
            let mut remaining = qty.abs();
            while remaining > Decimal::ZERO && !open.is_empty() {
                let idx = if lifo { open.len() - 1 } else { 0 };
                let take = remaining.min(open[idx].quantity);
                // Proceeds in base currency for the sold piece.
                let proceeds_base = take * lot.price * lot.fx_to_base;
                let cost_base = take * open[idx].cost_per_share_base;
                // Commission on the sell allocated proportionally to the sold piece.
                let total_sell_qty = qty.abs();
                let commission_share = if total_sell_qty == Decimal::ZERO {
                    Decimal::ZERO
                } else {
                    lot.commission_base * (take / total_sell_qty)
                };
                realized += proceeds_base - cost_base - commission_share;
                open[idx].quantity -= take;
                remaining -= take;
                if open[idx].quantity == Decimal::ZERO {
                    open.remove(idx);
                }
            }
            // If remaining > 0, the caller fed us a short sale or an over-sell.
            // We tolerate it by ignoring the leftover; agents shouldn't post invalid lots.
        }
    }

    finalize_open(open, realized)
}

fn apply_lots_average(share_lots: &[&TxLot]) -> Position {
    let mut total_qty = Decimal::ZERO;
    let mut total_cost_trade = Decimal::ZERO;
    let mut total_cost_base = Decimal::ZERO;
    let mut realized = Decimal::ZERO;

    for lot in share_lots {
        let qty = lot.quantity;
        if qty == Decimal::ZERO {
            continue;
        }
        let buy_like = matches!(lot.kind, TransactionKind::Buy | TransactionKind::CorporateAction)
            && qty > Decimal::ZERO;
        let sell_like = matches!(lot.kind, TransactionKind::Sell) && qty > Decimal::ZERO
            || qty < Decimal::ZERO;

        if buy_like {
            let qty_abs = qty.abs();
            total_qty += qty_abs;
            total_cost_trade += qty_abs * lot.price;
            total_cost_base += qty_abs * lot.price * lot.fx_to_base + lot.commission_base;
        } else if sell_like {
            let take = qty.abs().min(total_qty);
            if total_qty > Decimal::ZERO {
                let avg_base = total_cost_base / total_qty;
                let cost_base = take * avg_base;
                let proceeds_base = take * lot.price * lot.fx_to_base;
                realized += proceeds_base - cost_base - lot.commission_base;
                // Reduce running totals proportionally.
                let avg_trade = total_cost_trade / total_qty;
                total_cost_trade -= take * avg_trade;
                total_cost_base -= cost_base;
                total_qty -= take;
            }
        }
    }

    if total_qty == Decimal::ZERO {
        return Position {
            quantity: Decimal::ZERO,
            avg_cost_trade: Decimal::ZERO,
            cost_base: Decimal::ZERO,
            realized_pnl_base: realized,
        };
    }
    Position {
        quantity: total_qty,
        avg_cost_trade: total_cost_trade / total_qty,
        cost_base: total_cost_base,
        realized_pnl_base: realized,
    }
}

fn finalize_open(open: Vec<OpenLot>, realized: Decimal) -> Position {
    let mut qty = Decimal::ZERO;
    let mut cost_trade_weighted = Decimal::ZERO;
    let mut cost_base = Decimal::ZERO;
    for l in &open {
        qty += l.quantity;
        cost_trade_weighted += l.quantity * l.cost_per_share_trade;
        cost_base += l.quantity * l.cost_per_share_base;
    }
    let avg_cost_trade = if qty == Decimal::ZERO {
        Decimal::ZERO
    } else {
        cost_trade_weighted / qty
    };
    Position {
        quantity: qty,
        avg_cost_trade,
        cost_base,
        realized_pnl_base: realized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn buy(qty: i32, price: &str, commission: &str, sort: i64) -> TxLot {
        TxLot {
            kind: TransactionKind::Buy,
            quantity: Decimal::from(qty),
            price: price.parse().unwrap(),
            commission_base: commission.parse().unwrap(),
            fx_to_base: dec!(1.0),
            sort_key: sort,
        }
    }

    fn sell(qty: i32, price: &str, commission: &str, sort: i64) -> TxLot {
        TxLot {
            kind: TransactionKind::Sell,
            quantity: Decimal::from(qty),
            price: price.parse().unwrap(),
            commission_base: commission.parse().unwrap(),
            fx_to_base: dec!(1.0),
            sort_key: sort,
        }
    }

    #[test]
    fn fifo_single_buy_holds() {
        let lots = vec![buy(100, "10", "0", 1)];
        let p = compute_position(&lots, CostBasisMethod::Fifo);
        assert_eq!(p.quantity, dec!(100));
        assert_eq!(p.avg_cost_trade, dec!(10));
        assert_eq!(p.realized_pnl_base, dec!(0));
    }

    #[test]
    fn fifo_partial_sell_realizes_correctly() {
        // Buy 100 @ 10, sell 40 @ 15. Realized = 40 * (15 - 10) = 200.
        let lots = vec![buy(100, "10", "0", 1), sell(40, "15", "0", 2)];
        let p = compute_position(&lots, CostBasisMethod::Fifo);
        assert_eq!(p.quantity, dec!(60));
        assert_eq!(p.realized_pnl_base, dec!(200));
    }

    #[test]
    fn fifo_layered_buys_take_oldest_first() {
        // Buy 50 @ 10, buy 50 @ 20, sell 60 @ 30.
        // FIFO closes 50 @ 10 (gain 50 * 20 = 1000) then 10 @ 20 (gain 10 * 10 = 100). Total 1100.
        let lots = vec![
            buy(50, "10", "0", 1),
            buy(50, "20", "0", 2),
            sell(60, "30", "0", 3),
        ];
        let p = compute_position(&lots, CostBasisMethod::Fifo);
        assert_eq!(p.quantity, dec!(40));
        assert_eq!(p.realized_pnl_base, dec!(1100));
    }

    #[test]
    fn lifo_takes_newest_first() {
        // Same lots, LIFO closes the 20-cost layer first.
        // Sell 60 @ 30: 50 @ 20 (gain 500), 10 @ 10 (gain 200). Total 700.
        let lots = vec![
            buy(50, "10", "0", 1),
            buy(50, "20", "0", 2),
            sell(60, "30", "0", 3),
        ];
        let p = compute_position(&lots, CostBasisMethod::Lifo);
        assert_eq!(p.quantity, dec!(40));
        assert_eq!(p.realized_pnl_base, dec!(700));
    }

    #[test]
    fn average_basis() {
        // Buy 50 @ 10, buy 50 @ 20, sell 60 @ 30.
        // avg cost = (500 + 1000) / 100 = 15. Realized = 60 * (30 - 15) = 900.
        let lots = vec![
            buy(50, "10", "0", 1),
            buy(50, "20", "0", 2),
            sell(60, "30", "0", 3),
        ];
        let p = compute_position(&lots, CostBasisMethod::Average);
        assert_eq!(p.quantity, dec!(40));
        assert_eq!(p.realized_pnl_base, dec!(900));
    }

    #[test]
    fn commission_affects_basis() {
        // Buy 100 @ 10 with $5 commission → base cost adds $5.
        // Sell 100 @ 10 with $5 commission → realized = 1000 - 1005 - 5 = -10.
        let lots = vec![buy(100, "10", "5", 1), sell(100, "10", "5", 2)];
        let p = compute_position(&lots, CostBasisMethod::Fifo);
        assert_eq!(p.quantity, dec!(0));
        assert_eq!(p.realized_pnl_base, dec!(-10));
    }
}
