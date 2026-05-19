use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{CorrelationPair, UniverseDefinition};
use plutus_storage::queries::correlations::LocalizedCorrelationRun;

#[derive(Debug, Serialize, ToSchema)]
pub struct UniverseOut {
    pub id: i64,
    pub name: String,
    pub description_md: Option<String>,
    pub stock_ids: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UniverseDefinition> for UniverseOut {
    fn from(u: UniverseDefinition) -> Self {
        Self {
            id: u.id, name: u.name, description_md: u.description_md,
            stock_ids: u.stock_ids,
            created_at: u.created_at.to_string(), updated_at: u.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UniverseIn {
    pub name: String,
    pub description_md: Option<String>,
    pub stock_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CorrelationRunOut {
    pub id: i64,
    pub kind: String,
    pub run_date: String,
    pub universe_id: i64,
    pub lookback_days: i32,
    pub method: String,
    pub summary_md: Option<String>,
    pub metrics: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LocalizedCorrelationRun> for CorrelationRunOut {
    fn from(r: LocalizedCorrelationRun) -> Self {
        Self {
            id: r.id,
            kind: r.kind,
            run_date: r.run_date,
            universe_id: r.universe_id,
            lookback_days: r.lookback_days,
            method: r.method,
            summary_md: r.summary_md,
            metrics: r.metrics,
            source: r.source,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CorrelationRunIn {
    pub kind: String,
    pub run_date: String,
    pub universe_id: i64,
    pub lookback_days: i32,
    #[serde(default = "default_method")]
    pub method: String,
    pub metrics: Option<serde_json::Value>,
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob — `{ "<locale>": { "summary_md": "..." } }`.
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CorrelationPairOut {
    pub id: i64,
    pub run_id: i64,
    pub stock_a_id: i64,
    pub stock_b_id: i64,
    #[schema(value_type = String)]
    pub correlation: Decimal,
}

impl From<CorrelationPair> for CorrelationPairOut {
    fn from(p: CorrelationPair) -> Self {
        Self {
            id: p.id, run_id: p.run_id,
            stock_a_id: p.stock_a_id, stock_b_id: p.stock_b_id,
            correlation: p.correlation,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CorrelationPairIn {
    pub stock_a_id: i64,
    pub stock_b_id: i64,
    #[schema(value_type = String)]
    pub correlation: Decimal,
}

fn default_method() -> String { "pearson".into() }
fn default_source() -> String { "agent".into() }
