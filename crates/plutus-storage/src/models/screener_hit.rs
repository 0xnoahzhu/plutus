//! Per-stock hit from one `screener_runs` row.

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
    pub rationale_md: Option<String>,
    pub metrics: Option<String>, // JSON: per-stock metrics at scan time
    /// JSON map of locale → overrides for rationale_md.
    pub translations: Option<String>,
    pub created_at: jiff::Timestamp,
}
