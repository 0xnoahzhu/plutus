//! Sell-side coverage: consensus estimates time series + individual rating
//! actions with target prices.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "analyst_estimates"]
pub struct AnalystEstimate {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub metric: String, // "eps" / "revenue" / "ebitda" / "fcf"
    pub fiscal_year: i32,
    pub fiscal_period: String,
    pub consensus_mean: Option<Decimal>,
    pub consensus_median: Option<Decimal>,
    pub estimate_low: Option<Decimal>,
    pub estimate_high: Option<Decimal>,
    pub num_analysts: Option<i32>,
    pub as_of_date: String, // ISO date — estimates change over time
    pub source: String,
    pub created_at: jiff::Timestamp,
}

#[derive(Debug, toasty::Model)]
#[table = "analyst_ratings"]
pub struct AnalystRating {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub firm: String,           // "Goldman Sachs" / "中金"
    pub analyst_name: Option<String>,
    pub rating: String,         // "buy" / "overweight" / "hold" / "underweight" / "sell"
    pub rating_action: String,  // "initiated" / "upgrade" / "downgrade" / "reiterate"
    pub previous_rating: Option<String>,
    pub target_price: Option<Decimal>,
    pub target_currency: Option<String>,
    pub previous_target: Option<Decimal>,
    pub rated_at: String,       // ISO date
    pub source: String,
    pub created_at: jiff::Timestamp,
}
