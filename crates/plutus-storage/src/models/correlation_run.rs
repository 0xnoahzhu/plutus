//! One correlation-matrix computation. Pairs live in `correlation_pairs`.
//!
//! Translatable content (summary_md) lives in the `content` JSONB column
//! on the DB side. Because toasty 0.6 doesn't speak JSONB, the model omits
//! that column entirely — raw `tokio_postgres` SQL in
//! `queries::correlations` handles read/write of localized content.

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
    pub metrics: Option<String>, // JSON
    pub source: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
