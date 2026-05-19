//! Daily FX rates for unrealized P&L conversion.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "fx_rates_daily"]
pub struct FxRateDaily {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub base_currency: String,
    #[index]
    pub quote_currency: String,
    pub rate_date: String, // ISO date
    pub rate: Decimal,
    pub source: String,
    pub created_at: jiff::Timestamp,
}
