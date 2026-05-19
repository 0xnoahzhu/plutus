//! Whole-portfolio retrospectives — weekly, monthly, quarterly. Different from
//! `watchlist_reports` which scope to a single watchlist.

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
    pub headline: String,
    pub summary_md: Option<String>,
    pub content_md: Option<String>,
    pub decisions_md: Option<String>,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<Decimal>,
    pub metrics: Option<String>, // JSON
    pub language: String,
    pub source: String,
    /// JSON map of locale → overrides for headline / summary_md / content_md /
    /// decisions_md.
    pub translations: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
