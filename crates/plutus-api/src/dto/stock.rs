use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{Stock, StockTranslation};

#[derive(Debug, Serialize, ToSchema)]
pub struct StockOut {
    pub id: i64,
    pub market_code: String,
    pub symbol: String,
    pub isin: Option<String>,
    pub figi: Option<String>,
    pub currency: String,
    pub lot_size: Option<i32>,
    pub asset_class: String,
    pub sector_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Stock> for StockOut {
    fn from(s: Stock) -> Self {
        Self {
            id: s.id,
            market_code: s.market_code,
            symbol: s.symbol,
            isin: s.isin,
            figi: s.figi,
            currency: s.currency,
            lot_size: s.lot_size,
            asset_class: s.asset_class,
            sector_code: s.sector_code,
            created_at: s.created_at.to_string(),
            updated_at: s.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StockIn {
    pub market_code: String,
    pub symbol: String,
    pub isin: Option<String>,
    pub figi: Option<String>,
    pub currency: String,
    pub lot_size: Option<i32>,
    pub asset_class: String,
    pub sector_code: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StockPatch {
    pub isin: Option<String>,
    pub figi: Option<String>,
    pub currency: Option<String>,
    pub lot_size: Option<i32>,
    pub asset_class: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StockTranslationOut {
    pub stock_id: i64,
    pub locale: String,
    pub name: String,
    pub description_md: Option<String>,
    pub updated_at: String,
}

impl From<StockTranslation> for StockTranslationOut {
    fn from(t: StockTranslation) -> Self {
        Self {
            stock_id: t.stock_id,
            locale: t.locale,
            name: t.name,
            description_md: t.description_md,
            updated_at: t.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StockTranslationIn {
    pub name: String,
    pub description_md: Option<String>,
}
