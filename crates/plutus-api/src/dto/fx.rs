use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::FxRateDaily;

#[derive(Debug, Serialize, ToSchema)]
pub struct FxOut {
    pub id: i64,
    pub base_currency: String,
    pub quote_currency: String,
    pub rate_date: String,
    #[schema(value_type = String)]
    pub rate: Decimal,
    pub source: String,
}

impl From<FxRateDaily> for FxOut {
    fn from(f: FxRateDaily) -> Self {
        Self {
            id: f.id,
            base_currency: f.base_currency,
            quote_currency: f.quote_currency,
            rate_date: f.rate_date,
            rate: f.rate,
            source: f.source,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FxIn {
    pub base_currency: String,
    pub quote_currency: String,
    pub rate_date: String,
    #[schema(value_type = String)]
    pub rate: Decimal,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "agent".into()
}
