//! Daily and weekly reports on the watchlist. Per-user (uniqueness on
//! `(user_id, kind, period_start)`).
//!
//! Translatable content (headline / summary_md / content_md / notes) lives
//! in the `content` JSONB column on the DB side. Because toasty 0.6 doesn't
//! speak JSONB, the model omits that column entirely — raw `tokio_postgres`
//! SQL in `queries::watchlist_reports` handles read/write of localized
//! content. The fields declared here are the metadata columns toasty can
//! manage (filtering, indexing, ownership checks).

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
    pub sentiment: Option<String>,
    pub sentiment_score: Option<Decimal>,
    /// Free-form JSON blob the agent populates. Not translated — metrics
    /// are numbers / codes, not human-readable text.
    pub metrics: Option<String>,
    pub source: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
