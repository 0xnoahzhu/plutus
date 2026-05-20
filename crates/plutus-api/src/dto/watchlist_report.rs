use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::watchlist_reports::LocalizedWatchlistReport;

/// A daily or weekly report the agent writes summarizing the user's
/// watchlist — moves, news, what changed. Distinct from
/// `portfolio_review` (broader, periodic) and `market_brief` (per-country,
/// not per-watchlist). Upserts on `(user_id, kind, period_start)`.
///
/// Translatable fields (`headline`, `summary_md`, `content_md`, `notes`)
/// come from `content.<locale>`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WatchlistReportOut {
    /// Primary key.
    pub id: i64,
    /// `daily` | `weekly`. Part of natural key.
    pub kind: String,
    /// ISO date `YYYY-MM-DD`. Equals `period_end` for `kind=daily`.
    /// Part of natural key.
    pub period_start: String,
    /// ISO date `YYYY-MM-DD`.
    pub period_end: String,
    /// Localized one-line takeaway.
    pub headline: Option<String>,
    /// Localized summary markdown.
    pub summary_md: Option<String>,
    /// Localized full body markdown.
    pub content_md: Option<String>,
    /// Localized free-form notes.
    pub notes: Option<String>,
    /// `bullish` / `bearish` / `neutral`.
    pub sentiment: Option<String>,
    /// Numeric sentiment, usually `[-1, 1]`.
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// JSON-stringified per-metric values (e.g.
    /// `{"movers_up":4,"movers_down":3,"avg_move":0.012}`). Agent-defined
    /// shape.
    pub metrics: Option<String>,
    /// Provenance.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
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

/// `POST /watchlist/reports` body. Upserts against
/// `(user_id, kind, period_start)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct WatchlistReportIn {
    /// `daily` | `weekly`.
    pub kind: String,
    /// ISO date `YYYY-MM-DD`. For daily, equals `period_end`.
    pub period_start: String,
    /// ISO date `YYYY-MM-DD`.
    pub period_end: String,
    /// Sentiment label.
    pub sentiment: Option<String>,
    /// Numeric sentiment.
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// Metrics as a JSON object; the server stringifies.
    pub metrics: Option<serde_json::Value>,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "headline": "...", "summary_md": "...",
    /// "content_md": "...", "notes": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
