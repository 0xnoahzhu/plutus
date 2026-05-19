use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::SelfExam;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SelfExamOut {
    pub id: i64,
    pub kind: String,
    pub period_start: String,
    pub period_end: String,
    pub headline: String,
    pub content_md: Option<String>,
    pub metrics: Option<String>,
    pub recommendation_ids: Option<String>,
    pub notes: Option<String>,
    pub language: String,
    pub source: String,
    pub translations: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<SelfExam> for SelfExamOut {
    fn from(e: SelfExam) -> Self {
        Self {
            id: e.id, kind: e.kind, period_start: e.period_start, period_end: e.period_end,
            headline: e.headline, content_md: e.content_md, metrics: e.metrics,
            recommendation_ids: e.recommendation_ids, notes: e.notes,
            language: e.language, source: e.source,
            translations: e.translations,
            created_at: e.created_at.to_string(), updated_at: e.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SelfExamIn {
    pub kind: String,
    pub period_start: String,
    pub period_end: String,
    pub headline: String,
    pub content_md: Option<String>,
    pub metrics: Option<serde_json::Value>,
    pub recommendation_ids: Option<Vec<i64>>,
    pub notes: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_source")]
    pub source: String,
    pub translations: Option<serde_json::Value>,
}

fn default_language() -> String { "en".into() }
fn default_source() -> String { "agent".into() }
