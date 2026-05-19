//! A single execution of a named screener (e.g. "momentum_breakout").
//! Hits per run live in `screener_hits`. Natural key (user_id, name, kind,
//! run_date) enforced at the app layer.
//!
//! Translatable content (description_md, summary_md) lives in the `content`
//! JSONB column on the DB side. Because toasty 0.6 doesn't speak JSONB, the
//! model omits that column entirely — raw `tokio_postgres` SQL in
//! `queries::screeners` handles read/write of localized content.

#[derive(Debug, toasty::Model)]
#[table = "screener_runs"]
pub struct ScreenerRun {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    #[index]
    pub name: String, // "momentum_breakout" / "value_low_pe" / "rising_shorts"
    pub kind: String, // "weekly" / "daily" / "ad_hoc"
    pub run_date: String,
    pub universe: String, // free-form label, e.g. "US_LARGE_CAP" / "WATCHLIST_2"
    pub universe_size: Option<i32>,
    pub criteria: Option<String>,       // JSON
    pub sentiment: Option<String>,
    pub source: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
