use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::OhlcvDaily;

/// One daily OHLCV bar for a stock. Upserts against `(stock_id,
/// trade_date)` so a nightly backfill that re-fetches yesterday refreshes
/// the row in place. Shared across users (reference data).
#[derive(Debug, Serialize, ToSchema)]
pub struct OhlcvOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`. Part of the natural key.
    pub stock_id: i64,
    /// ISO date `YYYY-MM-DD` (the trade session date in the market's local
    /// calendar). Part of the natural key.
    pub trade_date: String,
    /// Open price.
    #[schema(value_type = String)]
    pub open: Decimal,
    /// Session high.
    #[schema(value_type = String)]
    pub high: Decimal,
    /// Session low.
    #[schema(value_type = String)]
    pub low: Decimal,
    /// Close price (raw, NOT split/dividend adjusted).
    #[schema(value_type = String)]
    pub close: Decimal,
    /// Split / dividend-adjusted close. `null` if the vendor didn't supply
    /// one. Use this for return calculations.
    #[schema(value_type = String)]
    pub adjusted_close: Option<Decimal>,
    /// Shares traded.
    pub volume: i64,
    /// Provenance (data vendor name).
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

/// `POST /stocks/{id}/ohlcv` or item-of-`POST /ohlcv/batch` body. Upserts
/// against `(stock_id, trade_date)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct OhlcvIn {
    /// FK to `stocks.id`. **Optional** in the per-stock route
    /// (`POST /stocks/{id}/ohlcv`) — the path wins. **Required** in the
    /// cross-stock batch route (`POST /ohlcv/batch`) — items missing
    /// `stock_id` return 400.
    pub stock_id: Option<i64>,
    /// ISO date `YYYY-MM-DD`.
    pub trade_date: String,
    /// Open.
    #[schema(value_type = String)]
    pub open: Decimal,
    /// High.
    #[schema(value_type = String)]
    pub high: Decimal,
    /// Low.
    #[schema(value_type = String)]
    pub low: Decimal,
    /// Close (raw).
    #[schema(value_type = String)]
    pub close: Decimal,
    /// Adjusted close. Optional.
    #[schema(value_type = String)]
    pub adjusted_close: Option<Decimal>,
    /// Shares traded.
    pub volume: i64,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "agent".into()
}

/// `POST /ohlcv/batch` body. Each item MUST carry `stock_id` (there's no
/// path-level disambiguation). Caps at 1000 items; one transaction.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct OhlcvBatchIn {
    pub items: Vec<OhlcvIn>,
}

/// `POST /ohlcv/batch` response.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OhlcvBatchOut {
    /// Number persisted (`== items.len()`).
    pub count: usize,
    /// Persisted rows in input order.
    pub items: Vec<OhlcvOut>,
}
