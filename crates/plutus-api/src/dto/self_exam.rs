use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::self_exams::LocalizedSelfExam;

/// Self-evaluation the agent writes about its own recommendations over a
/// period — "of the 12 recs I issued in March, 7 closed correct, 3 wrong;
/// here's what went wrong." Tied to closed recommendations via
/// `recommendation_ids`. Upserts on `(user_id, kind, period_start)`.
///
/// Translatable fields (`headline`, `content_md`, `notes`) come from
/// `content.<locale>`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SelfExamOut {
    /// Primary key.
    pub id: i64,
    /// `weekly` | `monthly` | `quarterly`. Part of the natural key.
    pub kind: String,
    /// ISO date `YYYY-MM-DD`. Part of the natural key.
    pub period_start: String,
    /// ISO date `YYYY-MM-DD`.
    pub period_end: String,
    /// Localized headline — top-line takeaway.
    pub headline: Option<String>,
    /// Localized markdown body. Free-form; agent decides sections.
    pub content_md: Option<String>,
    /// JSON-stringified scorecard (e.g.
    /// `{"total":12,"correct":7,"wrong":3,"hit_rate":0.58}`).
    pub metrics: Option<String>,
    /// JSON-stringified array of `recommendations.id` values covered by
    /// this exam. Stored as text so reads don't need a JOIN.
    pub recommendation_ids: Option<String>,
    /// Localized free-form notes.
    pub notes: Option<String>,
    /// Provenance — `agent` (default).
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
    /// RFC 3339 UTC timestamp when this user opened the item's detail
    /// page. `null` while the item is still unread.
    pub read_at: Option<String>,
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
            read_at: None,
        }
    }
}

/// `POST /self-exams` body. Upserts against `(user_id, kind,
/// period_start)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SelfExamIn {
    /// `weekly` | `monthly` | `quarterly`.
    pub kind: String,
    /// ISO date `YYYY-MM-DD`. Part of natural key.
    pub period_start: String,
    /// ISO date `YYYY-MM-DD`.
    pub period_end: String,
    /// Scorecard as a JSON object; the server stringifies.
    pub metrics: Option<serde_json::Value>,
    /// Array of `recommendations.id` values covered by this exam. The
    /// server stringifies as JSON for storage.
    pub recommendation_ids: Option<Vec<i64>>,
    /// Default: `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "headline": "...", "content_md": "...",
    /// "notes": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
