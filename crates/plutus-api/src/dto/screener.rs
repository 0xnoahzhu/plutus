use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::screeners::{LocalizedScreenerHit, LocalizedScreenerRun};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScreenerRunOut {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub run_date: String,
    pub universe: String,
    pub universe_size: Option<i32>,
    pub criteria: Option<String>,
    pub description_md: Option<String>,
    pub summary_md: Option<String>,
    pub sentiment: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LocalizedScreenerRun> for ScreenerRunOut {
    fn from(r: LocalizedScreenerRun) -> Self {
        Self {
            id: r.id,
            name: r.name,
            kind: r.kind,
            run_date: r.run_date,
            universe: r.universe,
            universe_size: r.universe_size,
            criteria: r.criteria,
            description_md: r.description_md,
            summary_md: r.summary_md,
            sentiment: r.sentiment,
            source: r.source,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ScreenerRunIn {
    pub name: String,
    pub kind: String,
    pub run_date: String,
    pub universe: String,
    pub universe_size: Option<i32>,
    pub criteria: Option<serde_json::Value>,
    pub sentiment: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob —
    /// `{ "<locale>": { "description_md": "...", "summary_md": "..." } }`.
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScreenerHitOut {
    pub id: i64,
    pub run_id: i64,
    pub stock_id: i64,
    pub rank: Option<i32>,
    #[schema(value_type = Option<String>)]
    pub score: Option<Decimal>,
    pub rationale_md: Option<String>,
    pub metrics: Option<String>,
    pub created_at: String,
}

impl From<LocalizedScreenerHit> for ScreenerHitOut {
    fn from(h: LocalizedScreenerHit) -> Self {
        Self {
            id: h.id,
            run_id: h.run_id,
            stock_id: h.stock_id,
            rank: h.rank,
            score: h.score,
            rationale_md: h.rationale_md,
            metrics: h.metrics,
            created_at: h.created_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ScreenerHitIn {
    pub stock_id: i64,
    pub rank: Option<i32>,
    #[schema(value_type = Option<String>)]
    pub score: Option<Decimal>,
    pub metrics: Option<serde_json::Value>,
    /// Multi-locale content blob — `{ "<locale>": { "rationale_md": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
