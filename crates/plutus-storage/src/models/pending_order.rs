//! A limit-style order the user has placed with their broker (or intends
//! to place). One row per order. Flat, single-tier — unlike
//! `trade_plan_levels`, which is a child of `trade_plans`, a pending
//! order stands on its own and points at a specific account.
//!
//! Provenance back to `trade_plan_levels.id` is optional: the user can
//! place an order against a planned level, or fire one off ad-hoc. The
//! FK is `ON DELETE SET NULL` so deleting a plan doesn't nuke the
//! historical record of an order that was actually submitted.
//!
//! No automatic broker sync — the user records what they placed, the
//! same way `transactions` works. Status flips are manual: `open` while
//! live at the broker, then `filled` / `cancelled` / `expired` once the
//! user knows what happened.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "pending_orders"]
pub struct PendingOrder {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    #[index]
    pub account_id: i64,
    /// Denormalized for "all my open orders for AAPL" queries without a
    /// join — same trick as `trade_plan_levels.stock_id`.
    #[index]
    pub stock_id: i64,
    /// Optional back-pointer to the planned level that motivated this
    /// order. `ON DELETE SET NULL` at the DB layer so the order outlives
    /// a deleted plan.
    pub trade_plan_level_id: Option<i64>,
    /// `buy` | `sell`. Enforced at the API DTO layer, not the DB.
    pub side: String,
    /// `limit` | `stop` | `stop_limit`. Plain market orders aren't
    /// "pending" by definition and aren't modeled here.
    pub order_type: String,
    /// Triggering price for `limit` and `stop_limit`. NULL for plain
    /// `stop` orders (which use `stop_price` only).
    pub limit_price: Option<Decimal>,
    /// Stop trigger for `stop` and `stop_limit`. NULL for plain `limit`.
    pub stop_price: Option<Decimal>,
    pub quantity: Decimal,
    /// `gtc` (good till cancelled) | `day` | `gtd` (good till date).
    /// Only `gtd` looks at `expires_at`.
    pub time_in_force: String,
    pub expires_at: Option<jiff::Timestamp>,
    /// `open` | `filled` | `cancelled` | `expired`. Storage layer
    /// stamps/clears the matching `*_at` columns when status flips,
    /// same pattern as `trade_plan_levels.triggered_at`.
    #[index]
    pub status: String,
    pub placed_at: jiff::Timestamp,
    pub filled_at: Option<jiff::Timestamp>,
    pub cancelled_at: Option<jiff::Timestamp>,
    /// Optional broker-side ref ID. Free-form so each broker's
    /// confirmation format (numeric, alphanumeric, UUID) fits.
    pub broker_order_ref: Option<String>,
    /// Quick personal annotation. Plain text, single language —
    /// consistent with `trade_plans.rationale` and `trade_plan_levels.notes`.
    pub notes: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
