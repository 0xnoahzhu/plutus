use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::WatchlistReport;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WatchlistReportOut {
    pub id: i64,
    pub kind: String,
    pub period_start: String,
    pub period_end: String,
    pub headline: String,
    pub summary_md: Option<String>,
    pub content_md: Option<String>,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// JSON blob; agent picks its own schema. Surface raw for the frontend to
    /// render however it likes (key-value list, chart, etc).
    pub metrics: Option<String>,
    pub notes: Option<String>,
    pub language: String,
    pub source: String,
    pub translations: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<WatchlistReport> for WatchlistReportOut {
    fn from(r: WatchlistReport) -> Self {
        Self {
            id: r.id,
            kind: r.kind,
            period_start: r.period_start,
            period_end: r.period_end,
            headline: r.headline,
            summary_md: r.summary_md,
            content_md: r.content_md,
            sentiment: r.sentiment,
            sentiment_score: r.sentiment_score,
            metrics: r.metrics,
            notes: r.notes,
            language: r.language,
            source: r.source,
            translations: r.translations,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WatchlistReportIn {
    /// "daily" or "weekly".
    pub kind: String,
    /// ISO YYYY-MM-DD. For daily, equals `period_end`.
    pub period_start: String,
    pub period_end: String,
    pub headline: String,
    pub summary_md: Option<String>,
    pub content_md: Option<String>,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    pub metrics: Option<serde_json::Value>,
    pub notes: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_source")]
    pub source: String,
    pub translations: Option<serde_json::Value>,
}

fn default_language() -> String { "en".into() }
fn default_source() -> String { "agent".into() }
