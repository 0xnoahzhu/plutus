use rust_decimal::Decimal;
use serde::Serialize;
use utoipa::ToSchema;

use plutus_storage::queries::holdings::Holding;

#[derive(Debug, Serialize, ToSchema)]
pub struct HoldingOut {
    pub stock_id: i64,
    pub account_id: Option<i64>,
    #[schema(value_type = String)]
    pub quantity: Decimal,
    #[schema(value_type = String)]
    pub avg_cost_trade: Decimal,
    #[schema(value_type = String)]
    pub cost_base: Decimal,
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
