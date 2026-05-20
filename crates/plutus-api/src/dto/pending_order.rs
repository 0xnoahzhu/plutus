use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::PendingOrder;

#[derive(Debug, Serialize, ToSchema)]
pub struct PendingOrderOut {
    pub id: i64,
    pub account_id: i64,
    pub stock_id: i64,
    pub trade_plan_level_id: Option<i64>,
    pub side: String,
    pub order_type: String,
    #[schema(value_type = Option<String>)]
    pub limit_price: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub stop_price: Option<Decimal>,
    #[schema(value_type = String)]
    pub quantity: Decimal,
    pub time_in_force: String,
    pub expires_at: Option<String>,
    pub status: String,
    pub placed_at: String,
    pub filled_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub broker_order_ref: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct PendingOrderIn {
    pub account_id: i64,
    pub stock_id: i64,
    pub trade_plan_level_id: Option<i64>,
    /// `buy` | `sell`
    pub side: String,
    /// `limit` | `stop` | `stop_limit`
    pub order_type: String,
    #[schema(value_type = Option<String>)]
    pub limit_price: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub stop_price: Option<Decimal>,
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// `gtc` (default) | `day` | `gtd`
    #[serde(default)]
    pub time_in_force: Option<String>,
    /// Only meaningful for `gtd`. RFC3339 string.
    pub expires_at: Option<String>,
    pub broker_order_ref: Option<String>,
    pub notes: Option<String>,
    /// Optional override — defaults to now. RFC3339.
    pub placed_at: Option<String>,
}

/// PATCH `/pending-orders/:id`. All fields optional. Double-wrapped
/// `Option<Option<...>>` lets the caller clear nullable fields back to
/// NULL (outer `Some` = "I sent this field", inner `None` = "clear it").
#[derive(Debug, Deserialize, ToSchema)]
pub struct PendingOrderPatch {
    pub account_id: Option<i64>,
    pub side: Option<String>,
    pub order_type: Option<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub limit_price: Option<Option<Decimal>>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub stop_price: Option<Option<Decimal>>,
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Decimal>,
    pub time_in_force: Option<String>,
    #[serde(default)]
    pub expires_at: Option<Option<String>>,
    #[serde(default)]
    pub broker_order_ref: Option<Option<String>>,
    #[serde(default)]
    pub notes: Option<Option<String>>,
    /// `open` | `filled` | `cancelled` | `expired`. Flipping to `filled`
    /// or `cancelled` stamps the matching `*_at` server-side.
    pub status: Option<String>,
}
