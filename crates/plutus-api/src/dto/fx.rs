use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::FxRateDaily;

/// Daily FX rate `base → quote`. Used by holdings rollup to convert
/// transaction `trade_currency` to the account's `base_currency` at the
/// trade date. Shared across users.
///
/// Convention: `1 base_currency = rate quote_currency`. So `USD/HKD =
/// 7.8` means 1 USD buys 7.8 HKD.
#[derive(Debug, Serialize, ToSchema)]
pub struct FxOut {
    /// Primary key.
    pub id: i64,
    /// ISO-4217 base currency.
    pub base_currency: String,
    /// ISO-4217 quote currency.
    pub quote_currency: String,
    /// ISO date `YYYY-MM-DD`.
    pub rate_date: String,
    /// `1 base = rate quote`.
    #[schema(value_type = String)]
    pub rate: Decimal,
    /// Provenance (data vendor).
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

/// `POST /fx` body. Upserts against
/// `(base_currency, quote_currency, rate_date)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct FxIn {
    /// ISO-4217 base.
    pub base_currency: String,
    /// ISO-4217 quote.
    pub quote_currency: String,
    /// ISO date.
    pub rate_date: String,
    /// `1 base = rate quote`.
    #[schema(value_type = String)]
    pub rate: Decimal,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "agent".into()
}
