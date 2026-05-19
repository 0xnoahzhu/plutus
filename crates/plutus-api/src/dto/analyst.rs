use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{AnalystEstimate, AnalystRating};

#[derive(Debug, Serialize, ToSchema)]
pub struct AnalystEstimateOut {
    pub id: i64,
    pub stock_id: i64,
    pub metric: String,
    pub fiscal_year: i32,
    pub fiscal_period: String,
    #[schema(value_type = Option<String>)] pub consensus_mean: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub consensus_median: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub estimate_low: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub estimate_high: Option<Decimal>,
    pub num_analysts: Option<i32>,
    pub as_of_date: String,
    pub source: String,
}

impl From<AnalystEstimate> for AnalystEstimateOut {
    fn from(e: AnalystEstimate) -> Self {
        Self {
            id: e.id, stock_id: e.stock_id, metric: e.metric,
            fiscal_year: e.fiscal_year, fiscal_period: e.fiscal_period,
            consensus_mean: e.consensus_mean, consensus_median: e.consensus_median,
            estimate_low: e.estimate_low, estimate_high: e.estimate_high,
            num_analysts: e.num_analysts, as_of_date: e.as_of_date, source: e.source,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalystEstimateIn {
    pub stock_id: i64,
    pub metric: String,
    pub fiscal_year: i32,
    pub fiscal_period: String,
    #[schema(value_type = Option<String>)] pub consensus_mean: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub consensus_median: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub estimate_low: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub estimate_high: Option<Decimal>,
    pub num_analysts: Option<i32>,
    pub as_of_date: String,
    #[serde(default = "default_source")] pub source: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnalystRatingOut {
    pub id: i64,
    pub stock_id: i64,
    pub firm: String,
    pub analyst_name: Option<String>,
    pub rating: String,
    pub rating_action: String,
    pub previous_rating: Option<String>,
    #[schema(value_type = Option<String>)] pub target_price: Option<Decimal>,
    pub target_currency: Option<String>,
    #[schema(value_type = Option<String>)] pub previous_target: Option<Decimal>,
    pub rated_at: String,
    pub source: String,
}

impl From<AnalystRating> for AnalystRatingOut {
    fn from(r: AnalystRating) -> Self {
        Self {
            id: r.id, stock_id: r.stock_id, firm: r.firm,
            analyst_name: r.analyst_name, rating: r.rating,
            rating_action: r.rating_action, previous_rating: r.previous_rating,
            target_price: r.target_price, target_currency: r.target_currency,
            previous_target: r.previous_target, rated_at: r.rated_at,
            source: r.source,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalystRatingIn {
    pub stock_id: i64,
    pub firm: String,
    pub analyst_name: Option<String>,
    pub rating: String,
    pub rating_action: String,
    pub previous_rating: Option<String>,
    #[schema(value_type = Option<String>)] pub target_price: Option<Decimal>,
    pub target_currency: Option<String>,
    #[schema(value_type = Option<String>)] pub previous_target: Option<Decimal>,
    pub rated_at: String,
    #[serde(default = "default_source")] pub source: String,
}

fn default_source() -> String { "agent".into() }
