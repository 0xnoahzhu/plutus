use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::PendingOrder;

/// A limit / stop order the user has placed (or intends to place) with
/// their broker. Once the broker fills it, the user posts a matching
/// `transaction` and flips this row's `status` to `filled`. Per-user.
///
/// Optionally linked to a `trade_plan_level` so the user can see "this
/// open order is the stop-loss leg of my AAPL plan."
#[derive(Debug, Serialize, ToSchema)]
pub struct PendingOrderOut {
    /// Primary key.
    pub id: i64,
    /// FK to `accounts.id`.
    pub account_id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Optional FK to `trade_plan_levels.id`. Set when this order was
    /// placed to honor a specific level of a trade plan.
    pub trade_plan_level_id: Option<i64>,
    /// `buy` | `sell`.
    pub side: String,
    /// `limit` | `stop` | `stop_limit`.
    pub order_type: String,
    /// Limit price (required for `limit` and `stop_limit`).
    #[schema(value_type = Option<String>)]
    pub limit_price: Option<Decimal>,
    /// Stop trigger (required for `stop` and `stop_limit`).
    #[schema(value_type = Option<String>)]
    pub stop_price: Option<Decimal>,
    /// Shares to trade.
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// `gtc` (good-till-cancelled, default) | `day` | `gtd` (good-till-date).
    pub time_in_force: String,
    /// RFC 3339 UTC timestamp. Only meaningful for `gtd`.
    pub expires_at: Option<String>,
    /// `open` (default) | `filled` | `cancelled` | `expired`. Flipping to
    /// `filled` / `cancelled` stamps `filled_at` / `cancelled_at`
    /// server-side.
    pub status: String,
    /// RFC 3339 UTC timestamp the order was placed.
    pub placed_at: String,
    /// RFC 3339 UTC timestamp the order was filled, or `null`.
    pub filled_at: Option<String>,
    /// RFC 3339 UTC timestamp the order was cancelled, or `null`.
    pub cancelled_at: Option<String>,
    /// Broker's order id, for reconciliation.
    pub broker_order_ref: Option<String>,
    /// Free-form English notes.
    pub notes: Option<String>,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

impl From<PendingOrder> for PendingOrderOut {
    fn from(o: PendingOrder) -> Self {
        Self {
            id: o.id,
            account_id: o.account_id,
            stock_id: o.stock_id,
            trade_plan_level_id: o.trade_plan_level_id,
            side: o.side,
            order_type: o.order_type,
            limit_price: o.limit_price,
            stop_price: o.stop_price,
            quantity: o.quantity,
            time_in_force: o.time_in_force,
            expires_at: o.expires_at.map(|t| t.to_string()),
            status: o.status,
            placed_at: o.placed_at.to_string(),
            filled_at: o.filled_at.map(|t| t.to_string()),
            cancelled_at: o.cancelled_at.map(|t| t.to_string()),
            broker_order_ref: o.broker_order_ref,
            notes: o.notes,
            created_at: o.created_at.to_string(),
            updated_at: o.updated_at.to_string(),
        }
    }
}

/// `POST /pending-orders` body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PendingOrderIn {
    /// FK to `accounts.id`. Must belong to the caller.
    pub account_id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Optional FK to `trade_plan_levels.id`.
    pub trade_plan_level_id: Option<i64>,
    /// `buy` | `sell`.
    pub side: String,
    /// `limit` | `stop` | `stop_limit`.
    pub order_type: String,
    /// Limit price (required for `limit` / `stop_limit`).
    #[schema(value_type = Option<String>)]
    pub limit_price: Option<Decimal>,
    /// Stop trigger (required for `stop` / `stop_limit`).
    #[schema(value_type = Option<String>)]
    pub stop_price: Option<Decimal>,
    /// Shares.
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// `gtc` (default) | `day` | `gtd`.
    #[serde(default)]
    pub time_in_force: Option<String>,
    /// Required for `gtd`. RFC 3339.
    pub expires_at: Option<String>,
    /// Broker order id.
    pub broker_order_ref: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Defaults to server-side `now()` when omitted.
    pub placed_at: Option<String>,
}

/// `PATCH /pending-orders/{id}` body. All fields optional —
/// `Option<Option<T>>` fields can be cleared to NULL by sending `null`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PendingOrderPatch {
    /// Reassign to a different account.
    pub account_id: Option<i64>,
    /// `buy` | `sell`.
    pub side: Option<String>,
    /// `limit` | `stop` | `stop_limit`.
    pub order_type: Option<String>,
    /// Limit price; send `null` to clear.
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub limit_price: Option<Option<Decimal>>,
    /// Stop trigger; send `null` to clear.
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub stop_price: Option<Option<Decimal>>,
    /// New share count.
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Decimal>,
    /// `gtc` | `day` | `gtd`.
    pub time_in_force: Option<String>,
    /// `gtd` expiry. Send `null` to clear.
    #[serde(default)]
    pub expires_at: Option<Option<String>>,
    /// Broker ref; send `null` to clear.
    #[serde(default)]
    pub broker_order_ref: Option<Option<String>>,
    /// Notes; send `null` to clear.
    #[serde(default)]
    pub notes: Option<Option<String>>,
    /// `open` | `filled` | `cancelled` | `expired`. Flipping to `filled`
    /// or `cancelled` stamps `filled_at` / `cancelled_at` server-side. To
    /// record the filled transaction, separately POST a matching row to
    /// `/transactions`.
    pub status: Option<String>,
}
