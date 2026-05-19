//! One price point inside a `trade_plans` row. Direction is implied by `kind`:
//!
//!   - `buy`         — buying-on-dip target; user expects price to drop here
//!   - `stop_loss`   — protective exit if the position turns; price drops here
//!   - `take_profit` — full exit at a target; price rises here
//!   - `trim`        — partial exit / scale-out; price rises here
//!
//! Both `quantity` (absolute share count) and `fraction_pct` (0..100% of
//! position) are nullable — caller picks whichever expression fits the
//! intent. `buy` levels tend to use `quantity` ("buy 100 shares"), `trim`
//! tends to use `fraction_pct` ("trim 25%"); nothing enforces a pairing.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "trade_plan_levels"]
pub struct TradePlanLevel {
    #[key]
    #[auto]
    pub id: i64,
    /// Denormalized from the parent plan so we don't need a join when
    /// filtering "all my active levels across stocks" (same pattern as
    /// `screener_hits.user_id`).
    #[index]
    pub user_id: i64,
    /// Also denormalized — supports "show me every level for AAPL"
    /// without joining trade_plans.
    #[index]
    pub stock_id: i64,
    #[index]
    pub plan_id: i64,
    /// One of `buy` / `stop_loss` / `take_profit` / `trim`. Enforced at
    /// the API DTO layer, not the DB.
    pub kind: String,
    pub price: Decimal,
    /// Absolute share count. Optional — set this OR `fraction_pct`, or
    /// neither (level with no size = trigger price only, user decides
    /// size at execution time).
    pub quantity: Option<Decimal>,
    /// Percent of current position, 0..100. Optional. See `quantity`.
    pub fraction_pct: Option<Decimal>,
    /// `active` while waiting, `triggered` after the user flips it (the
    /// system writes `triggered_at` at that moment), `cancelled` if the
    /// user abandons the level without acting.
    pub status: String,
    pub triggered_at: Option<jiff::Timestamp>,
    /// Per-level free-form note, distinct from the plan-level rationale.
    pub notes: Option<String>,
    /// Optional caller-supplied ordering for UI display. NULL means
    /// "default order" (the storage layer falls back to ascending price).
    pub sort_order: Option<i32>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
