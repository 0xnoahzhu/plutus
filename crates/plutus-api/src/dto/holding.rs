use rust_decimal::Decimal;
use serde::Serialize;
use utoipa::ToSchema;

use plutus_storage::queries::holdings::Holding;

/// A derived open position, rolled up from `transactions`. There's no
/// `holdings` table — the storage layer computes positions on every read
/// using weighted-average cost basis.
///
/// `GET /holdings` returns one row per `(stock_id, account_id)` for the
/// caller's accounts. Closed positions (quantity == 0) are omitted.
#[derive(Debug, Serialize, ToSchema)]
pub struct HoldingOut {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// FK to `accounts.id`. `null` for cross-account rollups (a future
    /// query mode not currently exposed).
    pub account_id: Option<i64>,
    /// Net open shares (positive = long).
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// Weighted-average cost per share in the trade currency.
    #[schema(value_type = String)]
    pub avg_cost_trade: Decimal,
    /// Total cost basis converted to the account's `base_currency` using
    /// each transaction's `fx_rate_to_base`.
    #[schema(value_type = String)]
    pub cost_base: Decimal,
    /// Realized P&L from closed legs, in base currency. Lifetime
    /// (resets only on full close).
    #[schema(value_type = String)]
    pub realized_pnl_base: Decimal,
}

impl From<Holding> for HoldingOut {
    fn from(h: Holding) -> Self {
        Self {
            stock_id: h.stock_id,
            account_id: h.account_id,
            quantity: h.position.quantity,
            avg_cost_trade: h.position.avg_cost_trade,
            cost_base: h.position.cost_base,
            realized_pnl_base: h.position.realized_pnl_base,
        }
    }
}
