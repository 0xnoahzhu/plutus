use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{TradePlan, TradePlanLevel};

/// A user's plan for trading one stock — entry/stop/target price points.
/// Each plan has many `TradePlanLevel` children. Plans are user-scoped;
/// other users can't see them.
///
/// `rationale` is plain English here (not multi-locale) — these are notes
/// to yourself.
#[derive(Debug, Serialize, ToSchema)]
pub struct TradePlanOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Free-form English rationale (the thesis driving these levels).
    pub rationale: Option<String>,
    /// `active` (default) | `paused` | `done`. Filter your UI on this.
    pub status: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

impl From<TradePlan> for TradePlanOut {
    fn from(p: TradePlan) -> Self {
        Self {
            id: p.id,
            stock_id: p.stock_id,
            rationale: p.rationale,
            status: p.status,
            created_at: p.created_at.to_string(),
            updated_at: p.updated_at.to_string(),
        }
    }
}

/// `POST /trade-plans` body. Creates a new plan in `active` status. Add
/// price levels via `POST /trade-plans/{id}/levels`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct TradePlanIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Free-form English rationale.
    pub rationale: Option<String>,
}

/// `PATCH /trade-plans/{id}` body. All fields optional — absent fields
/// stay untouched. `rationale` uses `Option<Option<String>>` so the caller
/// can explicitly clear it back to `NULL` (send `"rationale": null` to
/// clear; omit the key to leave alone).
#[derive(Debug, Deserialize, ToSchema)]
pub struct TradePlanPatch {
    /// `Some(None)` → set to NULL; `Some(Some(v))` → set to `v`;
    /// `None` (key omitted) → leave alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<Option<String>>,
    /// `active` | `paused` | `done`.
    pub status: Option<String>,
}

/// One price level within a trade plan — entry, stop-loss, take-profit,
/// or trim. Use `quantity` for absolute share counts or `fraction_pct` for
/// "sell 25% of the position", not both.
#[derive(Debug, Serialize, ToSchema)]
pub struct TradePlanLevelOut {
    /// Primary key.
    pub id: i64,
    /// FK to `trade_plans.id`.
    pub plan_id: i64,
    /// FK to `stocks.id` (denormalized from the parent plan for quick
    /// per-stock queries).
    pub stock_id: i64,
    /// `entry` | `stop_loss` | `take_profit` | `trim` | `add`.
    pub kind: String,
    /// Trigger price.
    #[schema(value_type = String)]
    pub price: Decimal,
    /// Absolute share count for this level. Mutually exclusive with
    /// `fraction_pct`.
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Decimal>,
    /// Fraction (in `[0, 1]`) of the current position to trade at this
    /// level. Mutually exclusive with `quantity`.
    #[schema(value_type = Option<String>)]
    pub fraction_pct: Option<Decimal>,
    /// `active` (default) | `triggered` | `cancelled`. Flipping to
    /// `triggered` server-side stamps `triggered_at`.
    pub status: String,
    /// RFC 3339 UTC timestamp the level was triggered. `null` while
    /// active.
    pub triggered_at: Option<String>,
    /// Free-form English notes.
    pub notes: Option<String>,
    /// Visual ordering hint for the UI when listing levels.
    pub sort_order: Option<i32>,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

impl From<TradePlanLevel> for TradePlanLevelOut {
    fn from(l: TradePlanLevel) -> Self {
        Self {
            id: l.id,
            plan_id: l.plan_id,
            stock_id: l.stock_id,
            kind: l.kind,
            price: l.price,
            quantity: l.quantity,
            fraction_pct: l.fraction_pct,
            status: l.status,
            triggered_at: l.triggered_at.map(|t| t.to_string()),
            notes: l.notes,
            sort_order: l.sort_order,
            created_at: l.created_at.to_string(),
            updated_at: l.updated_at.to_string(),
        }
    }
}

/// `POST /trade-plans/{id}/levels` body. Always inserts a new level row.
#[derive(Debug, Deserialize, ToSchema)]
pub struct TradePlanLevelIn {
    /// `entry` | `stop_loss` | `take_profit` | `trim` | `add`.
    pub kind: String,
    /// Trigger price.
    #[schema(value_type = String)]
    pub price: Decimal,
    /// Shares to trade. Mutually exclusive with `fraction_pct`.
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Decimal>,
    /// Fraction of the position. Mutually exclusive with `quantity`.
    #[schema(value_type = Option<String>)]
    pub fraction_pct: Option<Decimal>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// Visual ordering hint.
    pub sort_order: Option<i32>,
}

/// `PATCH /trade-plans/levels/{id}` body. Optional `Option<Option<...>>`
/// fields allow explicit clearing to `NULL` — see [`TradePlanPatch`].
#[derive(Debug, Deserialize, ToSchema)]
pub struct TradePlanLevelPatch {
    /// `entry` | `stop_loss` | `take_profit` | `trim` | `add`.
    pub kind: Option<String>,
    /// New trigger price.
    #[schema(value_type = Option<String>)]
    pub price: Option<Decimal>,
    /// Shares; `Some(None)` clears.
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Option<Decimal>>,
    /// Fraction of position; `Some(None)` clears.
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub fraction_pct: Option<Option<Decimal>>,
    /// Notes; `Some(None)` clears.
    #[serde(default)]
    pub notes: Option<Option<String>>,
    /// Sort order; `Some(None)` clears.
    #[serde(default)]
    pub sort_order: Option<Option<i32>>,
    /// `active` / `triggered` / `cancelled`. Flipping to `triggered`
    /// stamps `triggered_at` server-side.
    pub status: Option<String>,
}
