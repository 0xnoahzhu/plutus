//! One correlation-matrix computation. Pairs live in `correlation_pairs`.

#[derive(Debug, toasty::Model)]
#[table = "correlation_runs"]
pub struct CorrelationRun {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    pub kind: String, // "monthly" / "weekly" / "ad_hoc"
    pub run_date: String,
    #[index]
    pub universe_id: i64, // FK -> universe_definitions.id
    pub lookback_days: i32,
    pub method: String, // "pearson" / "spearman" / "kendall"
    pub summary_md: Option<String>,
    pub metrics: Option<String>, // JSON
    pub source: String,
    /// JSON map of locale → overrides for summary_md.
    pub translations: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
