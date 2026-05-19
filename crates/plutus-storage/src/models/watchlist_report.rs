//! Daily and weekly reports on the watchlist. Natural key
//! (kind, period_start) enforced at the app layer via upsert.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "watchlist_reports"]
pub struct WatchlistReport {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    /// "daily" / "weekly". Keep loose so we can add "monthly" later without
    /// schema changes.
    pub kind: String,
    /// ISO date. For daily this equals `period_end`; for weekly it's the
    /// Monday of the week.
    pub period_start: String,
    pub period_end: String,
    pub headline: String,
    pub summary_md: Option<String>,
    pub content_md: Option<String>,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<Decimal>,
    /// Free-form JSON blob the agent populates. We don't validate shape so
    /// each watchlist can carry the metrics that matter for its theme.
    pub metrics: Option<String>,
    pub notes: Option<String>,
    pub language: String,
    pub source: String,
    /// JSON map of locale → translated overrides for headline / summary_md /
    /// content_md / notes. Read by API handlers on `?locale=`.
    pub translations: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
