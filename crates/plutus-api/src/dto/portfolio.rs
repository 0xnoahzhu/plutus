use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::portfolio::DailyValue;

/// One day's portfolio rollup, returned by `GET /portfolio/value-series`.
/// The series gives the agent + the home-page chart enough to plot a
/// portfolio-equity curve over time without each caller re-deriving it
/// from `/transactions` + `/holdings` + per-stock `/stocks/:id/ohlcv`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DailyValueOut {
    /// ISO date `YYYY-MM-DD`. Calendar day, not trading day — weekends
    /// + holidays carry the previous trading day's close.
    pub date: String,
    /// Sum of `quantity * close_on_that_day` for every open position
    /// on that date. Uses adjusted close when present (split- /
    /// dividend-corrected). Stocks with no recorded price fall back to
    /// the position's cost basis so the series doesn't dip on missing
    /// data.
    #[schema(value_type = String)]
    pub market_value: Decimal,
    /// Sum of FIFO cost basis for every open position on that date.
    #[schema(value_type = String)]
    pub cost_basis: Decimal,
}

impl From<DailyValue> for DailyValueOut {
    fn from(v: DailyValue) -> Self {
        // Round to four decimal places to match the precision policy
        // used in /holdings — display layers don't have to re-format.
        Self {
            date: v.date,
            market_value: v.market_value.round_dp(4),
            cost_basis: v.cost_basis.round_dp(4),
        }
    }
}
