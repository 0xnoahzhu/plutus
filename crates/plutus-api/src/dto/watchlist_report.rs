use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::watchlist_reports::LocalizedWatchlistReport;

/// One watchlist report, with translatable text already projected for the
/// caller's locale by the storage layer. `headline` / `summary_md` /
/// `content_md` / `notes` come from the row's `content` JSONB blob; if the
/// requested locale is missing a particular field the storage layer falls
/// back to `en`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WatchlistReportOut {
    pub id: i64,
    pub kind: String,
    pub period_start: String,
    pub period_end: String,
    pub headline: Option<String>,
    pub summary_md: Option<String>,
    pub content_md: Option<String>,
    pub notes: Option<String>,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// JSON blob; agent picks its own schema. Surface raw for the frontend to
    /// render however it likes (key-value list, chart, etc).
    pub metrics: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LocalizedWatchlistReport> for WatchlistReportOut {
    fn from(r: LocalizedWatchlistReport) -> Self {
        Self {
            id: r.id,
            kind: r.kind,
            period_start: r.period_start,
            period_end: r.period_end,
            headline: r.headline,
            summary_md: r.summary_md,
            content_md: r.content_md,
            notes: r.notes,
            sentiment: r.sentiment,
            sentiment_score: r.sentiment_score,
            metrics: r.metrics,
            source: r.source,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }
    }
}

/// Upsert input. `content` is the full multi-locale blob —
/// `{ "<locale>": { "headline": "...", "summary_md": "...", ... } }`. The
/// storage layer writes it verbatim; partial-locale updates (e.g. adding
/// a `zh-CN` translation to an existing English row) are the caller's
/// responsibility to merge before sending.
#[derive(Debug, Deserialize, ToSchema)]
pub struct WatchlistReportIn {
    /// "daily" or "weekly".
    pub kind: String,
    /// ISO YYYY-MM-DD. For daily, equals `period_end`.
    pub period_start: String,
    pub period_end: String,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    pub metrics: Option<serde_json::Value>,
    #[serde(default = "default_source")]
    pub source: String,
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
