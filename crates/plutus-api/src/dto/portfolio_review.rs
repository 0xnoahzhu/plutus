use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::portfolio_reviews::LocalizedPortfolioReview;

/// A periodic portfolio review the agent writes for the user — weekly,
/// monthly, quarterly. Carries a free-form markdown body plus a sentiment
/// snapshot. Upserted by `(user_id, kind, period_start)`.
///
/// Translatable fields (`headline`, `summary_md`, `content_md`,
/// `decisions_md`) are projected from `content.<locale>` for the
/// `?locale=` of the request.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PortfolioReviewOut {
    /// Primary key.
    pub id: i64,
    /// Cadence — `weekly` | `monthly` | `quarterly`. Part of the natural
    /// key.
    pub kind: String,
    /// ISO date `YYYY-MM-DD`. Part of the natural key.
    pub period_start: String,
    /// ISO date `YYYY-MM-DD`. The agent picks both ends; the server doesn't
    /// enforce that `end > start`, so be careful.
    pub period_end: String,
    /// Localized headline — one-line takeaway for the home feed.
    pub headline: Option<String>,
    /// Localized markdown — short summary (3-5 sentences).
    pub summary_md: Option<String>,
    /// Localized markdown — full review body. Sections agreed by the agent
    /// (e.g. "Macro", "Portfolio", "Watchlist").
    pub content_md: Option<String>,
    /// Localized markdown — what the user should do this period.
    pub decisions_md: Option<String>,
    /// Sentiment flag — `bullish` / `bearish` / `neutral`. Used in the UI
    /// for a status pill.
    pub sentiment: Option<String>,
    /// Numeric sentiment in `[-1, 1]`, or whatever range the agent uses
    /// (server doesn't clamp). Useful for sparkline-style sentiment over
    /// time.
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// JSON-stringified per-metric values (e.g.
    /// `{"return_pct":3.4,"sharpe":1.6}`). Free-form.
    pub metrics: Option<String>,
    /// Provenance — `agent` (default), `manual`, or vendor.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp. Refreshed on upsert.
    pub updated_at: String,
    /// RFC 3339 UTC timestamp when this user opened the item's detail
    /// page. `null` while the item is still unread.
    pub read_at: Option<String>,
}

impl From<LocalizedPortfolioReview> for PortfolioReviewOut {
    fn from(r: LocalizedPortfolioReview) -> Self {
        Self {
            id: r.id,
            kind: r.kind,
            period_start: r.period_start,
            period_end: r.period_end,
            headline: r.headline,
            summary_md: r.summary_md,
            content_md: r.content_md,
            decisions_md: r.decisions_md,
            sentiment: r.sentiment,
            sentiment_score: r.sentiment_score,
            metrics: r.metrics,
            source: r.source,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
            read_at: None,
        }
    }
}

/// `POST /portfolio-reviews` body. Upserts against `(user_id, kind,
/// period_start)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PortfolioReviewIn {
    /// `weekly` | `monthly` | `quarterly`.
    pub kind: String,
    /// ISO date `YYYY-MM-DD`. Part of the natural key.
    pub period_start: String,
    /// ISO date `YYYY-MM-DD`.
    pub period_end: String,
    /// Sentiment label — `bullish` / `bearish` / `neutral`.
    pub sentiment: Option<String>,
    /// Numeric sentiment.
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// Per-metric values as a JSON object; the server stringifies.
    pub metrics: Option<serde_json::Value>,
    /// Default: `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "headline": "...", "summary_md": "...",
    /// "content_md": "...", "decisions_md": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
