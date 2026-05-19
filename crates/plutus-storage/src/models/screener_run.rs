//! A single execution of a named screener (e.g. "momentum_breakout").
//! Hits per run live in `screener_hits`. Natural key (name, kind, run_date)
//! enforced at the app layer.

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
    pub description_md: Option<String>,
    pub summary_md: Option<String>,
    pub sentiment: Option<String>,
    pub language: String,
    pub source: String,
    /// JSON map of locale → overrides for name / description_md / summary_md.
    pub translations: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
