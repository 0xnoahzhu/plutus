//! One off-diagonal entry of a correlation matrix. Canonical ordering
//! `stock_a_id < stock_b_id` is enforced at the app layer to avoid duplicate
//! mirror rows.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "correlation_pairs"]
pub struct CorrelationPair {
    #[key]
    #[auto]
    pub id: i64,
    /// Denormalized from the parent run for per-user filtering without a join.
    #[index]
    pub user_id: i64,
    #[index]
    pub run_id: i64,
    #[index]
    pub stock_a_id: i64,
    #[index]
    pub stock_b_id: i64,
    pub correlation: Decimal,
}
