use serde::Serialize;
use utoipa::ToSchema;

use plutus_storage::models::Market;

#[derive(Debug, Serialize, ToSchema)]
pub struct MarketOut {
    pub code: String,
    pub name: String,
    pub country: String,
    pub timezone: String,
    pub currency_code: String,
    pub default_lot_size: i32,
    pub settlement_days: i32,
}

impl From<Market> for MarketOut {
    fn from(m: Market) -> Self {
        Self {
            code: m.code,
            name: m.name,
            country: m.country,
            timezone: m.timezone,
            currency_code: m.currency_code,
            default_lot_size: m.default_lot_size,
            settlement_days: m.settlement_days,
        }
    }
}
