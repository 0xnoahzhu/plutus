use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{AnalystEstimate, AnalystRating};

/// Per-stock consensus estimate from sell-side analysts for a particular
/// metric and fiscal period. Multiple rows per stock per period — one per
/// metric. Time series: re-POSTing with a newer `as_of_date` accumulates.
/// Shared across users.
#[derive(Debug, Serialize, ToSchema)]
pub struct AnalystEstimateOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Metric estimated — `eps` | `revenue` | `ebitda` | `fcf` | etc.
    pub metric: String,
    /// 4-digit fiscal year.
    pub fiscal_year: i32,
    /// `Q1` / `Q2` / `Q3` / `Q4` / `FY`.
    pub fiscal_period: String,
    /// Mean of analyst estimates.
    #[schema(value_type = Option<String>)] pub consensus_mean: Option<Decimal>,
    /// Median of analyst estimates.
    #[schema(value_type = Option<String>)] pub consensus_median: Option<Decimal>,
    /// Lowest estimate.
    #[schema(value_type = Option<String>)] pub estimate_low: Option<Decimal>,
    /// Highest estimate.
    #[schema(value_type = Option<String>)] pub estimate_high: Option<Decimal>,
    /// Number of analysts contributing.
    pub num_analysts: Option<i32>,
    /// ISO date `YYYY-MM-DD` the snapshot was taken.
    pub as_of_date: String,
    /// Provenance.
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

/// `POST /analyst/estimates` body. Inserts a new row each call — the
/// table accumulates a time series of consensus snapshots.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalystEstimateIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// `eps` | `revenue` | `ebitda` | `fcf`.
    pub metric: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// `Q1`/`Q2`/`Q3`/`Q4`/`FY`.
    pub fiscal_period: String,
    /// Consensus mean.
    #[schema(value_type = Option<String>)] pub consensus_mean: Option<Decimal>,
    /// Consensus median.
    #[schema(value_type = Option<String>)] pub consensus_median: Option<Decimal>,
    /// Min.
    #[schema(value_type = Option<String>)] pub estimate_low: Option<Decimal>,
    /// Max.
    #[schema(value_type = Option<String>)] pub estimate_high: Option<Decimal>,
    /// Coverage count.
    pub num_analysts: Option<i32>,
    /// ISO date `YYYY-MM-DD`.
    pub as_of_date: String,
    /// Default `agent`.
    #[serde(default = "default_source")] pub source: String,
}

/// One analyst rating action — upgrade, downgrade, initiation, target
/// change, etc. Time series row per (firm, stock, rated_at).
/// Shared across users.
#[derive(Debug, Serialize, ToSchema)]
pub struct AnalystRatingOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Research firm (e.g. `goldman_sachs`).
    pub firm: String,
    /// Analyst name if known.
    pub analyst_name: Option<String>,
    /// Current rating — `buy` | `hold` | `sell` | `overweight` |
    /// `underweight` | `outperform` | `underperform`. Free-form.
    pub rating: String,
    /// What changed — `initiation` | `upgrade` | `downgrade` |
    /// `reiterate` | `target_change` | `coverage_dropped`.
    pub rating_action: String,
    /// Previous rating (for context on upgrades / downgrades).
    pub previous_rating: Option<String>,
    /// New price target (in `target_currency`).
    #[schema(value_type = Option<String>)] pub target_price: Option<Decimal>,
    /// ISO-4217 currency.
    pub target_currency: Option<String>,
    /// Previous target.
    #[schema(value_type = Option<String>)] pub previous_target: Option<Decimal>,
    /// ISO date `YYYY-MM-DD` of the rating action.
    pub rated_at: String,
    /// Provenance.
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

/// `POST /analyst/ratings` body. Inserts a new row each call.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalystRatingIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Research firm.
    pub firm: String,
    /// Analyst name.
    pub analyst_name: Option<String>,
    /// New rating string.
    pub rating: String,
    /// `initiation` | `upgrade` | `downgrade` | `reiterate` |
    /// `target_change` | `coverage_dropped`.
    pub rating_action: String,
    /// Prior rating.
    pub previous_rating: Option<String>,
    /// New price target.
    #[schema(value_type = Option<String>)] pub target_price: Option<Decimal>,
    /// ISO-4217 currency.
    pub target_currency: Option<String>,
    /// Prior target.
    #[schema(value_type = Option<String>)] pub previous_target: Option<Decimal>,
    /// ISO date `YYYY-MM-DD`.
    pub rated_at: String,
    /// Default `agent`.
    #[serde(default = "default_source")] pub source: String,
}

fn default_source() -> String { "agent".into() }

/// `POST /analyst/estimates/batch` body. Caps at 1000 items;
/// all-or-nothing transaction. Designed for the agent's nightly
/// consensus refresh sweeps.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalystEstimateBatchIn {
    pub items: Vec<AnalystEstimateIn>,
}

/// `POST /analyst/estimates/batch` response.
#[derive(Debug, Serialize, ToSchema)]
pub struct AnalystEstimateBatchOut {
    /// Number persisted (`== items.len()`).
    pub count: usize,
    /// Rows in input order.
    pub items: Vec<AnalystEstimateOut>,
}

/// `POST /analyst/ratings/batch` body. Same shape as the estimates
/// batch.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalystRatingBatchIn {
    pub items: Vec<AnalystRatingIn>,
}

/// `POST /analyst/ratings/batch` response.
#[derive(Debug, Serialize, ToSchema)]
pub struct AnalystRatingBatchOut {
    pub count: usize,
    pub items: Vec<AnalystRatingOut>,
}
