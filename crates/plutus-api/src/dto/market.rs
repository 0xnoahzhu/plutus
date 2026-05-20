use serde::Serialize;
use utoipa::ToSchema;

use plutus_storage::models::Market;

/// A trading venue (NYSE, NASDAQ, HKEX, etc.). Reference data. Stocks
/// link to a market by `market_code`; the country filter on
/// `GET /stocks?country=US` resolves through this table's `country`.
#[derive(Debug, Serialize, ToSchema)]
pub struct MarketOut {
    /// Primary key — short code (e.g. `us`, `us_etf`, `hk`, `cn_a`,
    /// `cn_etf`).
    pub code: String,
    /// Display name.
    pub name: String,
    /// ISO country code the market lives in.
    pub country: String,
    /// IANA timezone (e.g. `America/New_York`).
    pub timezone: String,
    /// ISO-4217 default trading currency.
    pub currency_code: String,
    /// Default share lot size on this venue (`1` for fractional US
    /// equities, `100` for HK common stock, `200`/`500`+ for some HK
    /// names — overridden per-stock by `stocks.lot_size`).
    pub default_lot_size: i32,
    /// T+N settlement (typically `1` or `2`).
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
