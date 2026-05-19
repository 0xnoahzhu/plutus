use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::self_exams::LocalizedSelfExam;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SelfExamOut {
    pub id: i64,
    pub kind: String,
    pub period_start: String,
    pub period_end: String,
    pub headline: Option<String>,
    pub content_md: Option<String>,
    pub metrics: Option<String>,
    pub recommendation_ids: Option<String>,
    pub notes: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LocalizedSelfExam> for SelfExamOut {
    fn from(e: LocalizedSelfExam) -> Self {
        Self {
            id: e.id,
            kind: e.kind,
            period_start: e.period_start,
            period_end: e.period_end,
            headline: e.headline,
            content_md: e.content_md,
            metrics: e.metrics,
            recommendation_ids: e.recommendation_ids,
            notes: e.notes,
            source: e.source,
            created_at: e.created_at.to_string(),
            updated_at: e.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SelfExamIn {
    pub kind: String,
    pub period_start: String,
    pub period_end: String,
    pub metrics: Option<serde_json::Value>,
    pub recommendation_ids: Option<Vec<i64>>,
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob —
    /// `{ "<locale>": { "headline": "...", "content_md": "...", "notes": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
