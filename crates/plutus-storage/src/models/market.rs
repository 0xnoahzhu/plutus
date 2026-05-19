//! ISO 10383 MIC market reference table. Holds the cross-cutting metadata
//! (timezone, currency, lot size) that varies by market so we don't have to
//! split tables per market.

#[derive(Debug, toasty::Model)]
#[table = "markets"]
pub struct Market {
    #[key]
    pub code: String, // MIC, e.g. "XNAS"
    pub name: String,
    pub country: String,
    pub timezone: String,
    pub currency_code: String,
    pub default_lot_size: i32,
    pub settlement_days: i32,
}
