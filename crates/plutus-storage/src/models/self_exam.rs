//! Periodic self-review of past recommendations.
//!
//! Translatable content (headline, content_md, notes) lives in the
//! `content` JSONB column on the DB side. Because toasty 0.6 doesn't speak
//! JSONB, the model omits that column entirely — raw `tokio_postgres` SQL
//! in `queries::self_exams` handles read/write of localized content.

#[derive(Debug, toasty::Model)]
#[table = "self_exams"]
pub struct SelfExam {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    pub kind: String, // "weekly" / "monthly" / "quarterly"
    pub period_start: String,
    pub period_end: String,
    pub metrics: Option<String>,           // JSON: accuracy, hit_rate, avg_pnl, ...
    pub recommendation_ids: Option<String>, // JSON array of rec ids evaluated
    pub source: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
