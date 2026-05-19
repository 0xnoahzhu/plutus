use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::recommendations::LocalizedRecommendation;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecommendationOut {
    pub id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<String>,
    pub action: String,
    #[schema(value_type = Option<String>)]
    pub confidence: Option<Decimal>,
    pub rationale_md: Option<String>,
    #[schema(value_type = Option<String>)]
    pub target_price: Option<Decimal>,
    pub target_currency: Option<String>,
    pub target_horizon: String,
    pub issued_at: String,
    pub status: String,
    pub outcome_md: Option<String>,
    #[schema(value_type = Option<String>)]
    pub pnl_pct: Option<Decimal>,
    pub closed_at: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LocalizedRecommendation> for RecommendationOut {
    fn from(r: LocalizedRecommendation) -> Self {
        Self {
            id: r.id,
            stock_id: r.stock_id,
            sector_code: r.sector_code,
            action: r.action,
            confidence: r.confidence,
            rationale_md: r.rationale_md,
            target_price: r.target_price,
            target_currency: r.target_currency,
            target_horizon: r.target_horizon,
            issued_at: r.issued_at.to_string(),
            status: r.status,
            outcome_md: r.outcome_md,
            pnl_pct: r.pnl_pct,
            closed_at: r.closed_at.map(|t| t.to_string()),
            source: r.source,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecommendationIn {
    pub stock_id: Option<i64>,
    pub sector_code: Option<String>,
    pub action: String,
    #[schema(value_type = Option<String>)]
    pub confidence: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub target_price: Option<Decimal>,
    pub target_currency: Option<String>,
    #[serde(default = "default_horizon")]
    pub target_horizon: String,
    /// RFC 3339; defaults to now if omitted.
    pub issued_at: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob — `{ "<locale>": { "rationale_md": "...",
    /// "outcome_md": "..." } }`. `outcome_md` is typically populated later
    /// via the close endpoint.
    pub content: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecommendationClosePatch {
    /// "closed_correct" / "closed_wrong" / "closed_neutral" / "expired".
    pub status: String,
    pub outcome_md: Option<String>,
    #[schema(value_type = Option<String>)]
    pub pnl_pct: Option<Decimal>,
    /// RFC 3339; defaults to now if omitted.
    pub closed_at: Option<String>,
}

fn default_horizon() -> String { "open".into() }
fn default_source() -> String { "agent".into() }
