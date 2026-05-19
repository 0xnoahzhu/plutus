use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{TradePlan, TradePlanLevel};

#[derive(Debug, Serialize, ToSchema)]
pub struct TradePlanOut {
    pub id: i64,
    pub stock_id: i64,
    pub rationale: Option<String>,
    pub status: String,
    pub created_at: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct TradePlanIn {
    pub stock_id: i64,
    pub rationale: Option<String>,
}

/// PATCH `/trade-plans/:id` payload. Every field is optional; absent
/// fields stay untouched. `rationale` is double-wrapped so the caller
/// can clear it back to NULL (outer `Some` = "I sent this field",
/// inner `None` = "set it to null").
#[derive(Debug, Deserialize, ToSchema)]
pub struct TradePlanPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<Option<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TradePlanLevelOut {
    pub id: i64,
    pub plan_id: i64,
    pub stock_id: i64,
    pub kind: String,
    #[schema(value_type = String)]
    pub price: Decimal,
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub fraction_pct: Option<Decimal>,
    pub status: String,
    pub triggered_at: Option<String>,
    pub notes: Option<String>,
    pub sort_order: Option<i32>,
    pub created_at: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct TradePlanLevelIn {
    pub kind: String,
    #[schema(value_type = String)]
    pub price: Decimal,
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub fraction_pct: Option<Decimal>,
    pub notes: Option<String>,
    pub sort_order: Option<i32>,
}

/// PATCH `/trade-plans/levels/:id`. Optional fields (`quantity`,
/// `fraction_pct`, `notes`, `sort_order`) use `Option<Option<...>>` so
/// the caller can clear them back to NULL.
#[derive(Debug, Deserialize, ToSchema)]
pub struct TradePlanLevelPatch {
    pub kind: Option<String>,
    #[schema(value_type = Option<String>)]
    pub price: Option<Decimal>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Option<Decimal>>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub fraction_pct: Option<Option<Decimal>>,
    #[serde(default)]
    pub notes: Option<Option<String>>,
    #[serde(default)]
    pub sort_order: Option<Option<i32>>,
    /// `active` / `triggered` / `cancelled`. Flipping to `triggered`
    /// stamps `triggered_at` server-side.
    pub status: Option<String>,
}
