use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::OhlcvDaily;

#[derive(Debug, Serialize, ToSchema)]
pub struct OhlcvOut {
    pub id: i64,
    pub stock_id: i64,
    pub trade_date: String,
    #[schema(value_type = String)]
    pub open: Decimal,
    #[schema(value_type = String)]
    pub high: Decimal,
    #[schema(value_type = String)]
    pub low: Decimal,
    #[schema(value_type = String)]
    pub close: Decimal,
    #[schema(value_type = String)]
    pub adjusted_close: Option<Decimal>,
    pub volume: i64,
    pub source: String,
}

impl From<OhlcvDaily> for OhlcvOut {
    fn from(o: OhlcvDaily) -> Self {
        Self {
            id: o.id,
            stock_id: o.stock_id,
            trade_date: o.trade_date,
            open: o.open,
            high: o.high,
            low: o.low,
            close: o.close,
            adjusted_close: o.adjusted_close,
            volume: o.volume,
            source: o.source,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OhlcvIn {
    pub trade_date: String,
    #[schema(value_type = String)]
    pub open: Decimal,
    #[schema(value_type = String)]
    pub high: Decimal,
    #[schema(value_type = String)]
    pub low: Decimal,
    #[schema(value_type = String)]
    pub close: Decimal,
    #[schema(value_type = String)]
    pub adjusted_close: Option<Decimal>,
    pub volume: i64,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "agent".into()
}
