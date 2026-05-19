//! Per-stock hit from one `screener_runs` row.
//!
//! Translatable content (rationale_md) lives in the `content` JSONB column
//! on the DB side. Because toasty 0.6 doesn't speak JSONB, the model omits
//! that column entirely — raw `tokio_postgres` SQL in `queries::screeners`
//! handles read/write of localized content.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "screener_hits"]
pub struct ScreenerHit {
    #[key]
    #[auto]
    pub id: i64,
    /// Denormalized from the parent run so per-user filtering doesn't need a join.
    #[index]
    pub user_id: i64,
    #[index]
    pub run_id: i64,
    #[index]
    pub stock_id: i64,
    pub rank: Option<i32>,
    pub score: Option<Decimal>,
    pub metrics: Option<String>, // JSON: per-stock metrics at scan time
    pub created_at: jiff::Timestamp,
}
