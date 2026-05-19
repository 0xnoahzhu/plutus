//! Periodic self-review of past recommendations.

#[derive(Debug, toasty::Model)]
#[table = "self_exams"]
pub struct SelfExam {
    #[key]
    #[auto]
    pub id: i64,
    pub kind: String, // "weekly" / "monthly" / "quarterly"
    pub period_start: String,
    pub period_end: String,
    pub headline: String,
    pub content_md: Option<String>,
    pub metrics: Option<String>,           // JSON: accuracy, hit_rate, avg_pnl, ...
    pub recommendation_ids: Option<String>, // JSON array of rec ids evaluated
    pub notes: Option<String>,
    pub language: String,
    pub source: String,
    /// JSON map of locale → overrides for headline / content_md / notes.
    pub translations: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
