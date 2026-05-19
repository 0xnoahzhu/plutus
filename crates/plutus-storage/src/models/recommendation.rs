//! Single recommendation the agent makes. Lifecycle: open → closed_*. Self-exams
//! aggregate over these.
//!
//! Translatable content (rationale_md, outcome_md) lives in the `content`
//! JSONB column on the DB side. Because toasty 0.6 doesn't speak JSONB, the
//! model omits that column entirely — raw `tokio_postgres` SQL in
//! `queries::recommendations` handles read/write of localized content.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "recommendations"]
pub struct Recommendation {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    #[index]
    pub stock_id: Option<i64>, // null for macro / sector-level recs
    pub sector_code: Option<String>,
    pub action: String, // "buy" / "sell" / "hold" / "watch" / "avoid"
    pub confidence: Option<Decimal>, // 0..1
    pub target_price: Option<Decimal>,
    pub target_currency: Option<String>,
    pub target_horizon: String, // "1d" / "1w" / "1m" / "3m" / "1y" / "open"
    pub issued_at: jiff::Timestamp,
    // ── evaluation fields filled when the rec is closed ─────────────
    pub status: String, // "open" / "closed_correct" / "closed_wrong" /
                        //  "closed_neutral" / "expired"
    pub pnl_pct: Option<Decimal>,
    pub closed_at: Option<jiff::Timestamp>,
    pub source: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
