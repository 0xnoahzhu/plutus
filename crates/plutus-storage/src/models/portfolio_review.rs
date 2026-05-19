//! Whole-portfolio retrospectives — weekly, monthly, quarterly. Different from
//! `watchlist_reports` which scope to a single watchlist.
//!
//! Translatable content (headline, summary_md, content_md, decisions_md)
//! lives in the `content` JSONB column on the DB side. Because toasty 0.6
//! doesn't speak JSONB, the model omits that column entirely — raw
//! `tokio_postgres` SQL in `queries::portfolio_reviews` handles read/write
//! of localized content.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "portfolio_reviews"]
pub struct PortfolioReview {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    pub kind: String, // "weekly" / "monthly" / "quarterly"
    pub period_start: String,
    pub period_end: String,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<Decimal>,
    pub metrics: Option<String>, // JSON
    pub source: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
