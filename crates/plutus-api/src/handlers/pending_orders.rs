//! Per-user pending limit orders — what the user has live at their
//! broker (or intends to place). All routes go through `require_user`;
//! admin actors get 403 (admin has no per-user data of its own).
//!
//! Validation lives here at the DTO boundary so the storage layer can
//! stay schemaless. The interesting non-trivial rule is the order-type
//! ↔ price-fields invariant: `limit` needs `limit_price`, `stop` needs
//! `stop_price`, `stop_limit` needs both.

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::pending_order::{PendingOrderIn, PendingOrderOut, PendingOrderPatch};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub account_id: Option<i64>,
    pub stock_id: Option<i64>,
    pub status: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
) -> ApiResult<Json<Vec<PendingOrderOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::pending_orders::list(
        &state.db,
        plutus_storage::queries::pending_orders::ListFilter {
            user_id,
            account_id: f.account_id,
            stock_id: f.stock_id,
            status: f.status.as_deref(),
        },
    )
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<Json<PendingOrderOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::pending_orders::get(&state.db, user_id, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<PendingOrderIn>,
) -> ApiResult<Json<PendingOrderOut>> {
    let user_id = require_user(&actor.0)?;
    validate_side(&input.side)?;
    validate_order_type(&input.order_type)?;
    validate_price_fields(&input.order_type, input.limit_price, input.stop_price)?;
    if input.quantity <= rust_decimal::Decimal::ZERO {
        return Err(ApiError::BadRequest("quantity must be > 0".to_string()));
    }
    let tif = input.time_in_force.as_deref().unwrap_or("gtc");
    validate_tif(tif)?;
    let expires_at = match input.expires_at.as_deref() {
        Some(s) => Some(parse_ts(s, "expires_at")?),
        None => None,
    };
    let placed_at = match input.placed_at.as_deref() {
        Some(s) => Some(parse_ts(s, "placed_at")?),
        None => None,
    };
    let row = plutus_storage::queries::pending_orders::create(
        &state.db,
        plutus_storage::queries::pending_orders::NewOrder {
            user_id,
            account_id: input.account_id,
            stock_id: input.stock_id,
            trade_plan_level_id: input.trade_plan_level_id,
            side: &input.side,
            order_type: &input.order_type,
            limit_price: input.limit_price,
            stop_price: input.stop_price,
            quantity: input.quantity,
            time_in_force: tif,
            expires_at,
            broker_order_ref: input.broker_order_ref.as_deref(),
            notes: input.notes.as_deref(),
            placed_at,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn update(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(patch): Json<PendingOrderPatch>,
) -> ApiResult<Json<PendingOrderOut>> {
    let user_id = require_user(&actor.0)?;
    if let Some(s) = patch.side.as_deref() {
        validate_side(s)?;
    }
    if let Some(s) = patch.order_type.as_deref() {
        validate_order_type(s)?;
    }
    if let Some(s) = patch.time_in_force.as_deref() {
        validate_tif(s)?;
    }
    if let Some(s) = patch.status.as_deref() {
        validate_status(s)?;
    }
    if let Some(q) = patch.quantity {
        if q <= rust_decimal::Decimal::ZERO {
            return Err(ApiError::BadRequest("quantity must be > 0".to_string()));
        }
    }
    // Parse expires_at outer/inner: outer Some = "field present"; inner
    // Some/None = the actual value to set / clear.
    let expires_at = match patch.expires_at {
        Some(Some(s)) => Some(Some(parse_ts(&s, "expires_at")?)),
        Some(None) => Some(None),
        None => None,
    };
    let row = plutus_storage::queries::pending_orders::update(
        &state.db,
        user_id,
        id,
        plutus_storage::queries::pending_orders::OrderPatch {
            account_id: patch.account_id,
            side: patch.side.as_deref(),
            order_type: patch.order_type.as_deref(),
            limit_price: patch.limit_price,
            stop_price: patch.stop_price,
            quantity: patch.quantity,
            time_in_force: patch.time_in_force.as_deref(),
            expires_at,
            broker_order_ref: patch.broker_order_ref.as_ref().map(|o| o.as_deref()),
            notes: patch.notes.as_ref().map(|o| o.as_deref()),
            status: patch.status.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let user_id = require_user(&actor.0)?;
    plutus_storage::queries::pending_orders::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Validators ───────────────────────────────────────────────────────────

fn validate_side(s: &str) -> ApiResult<()> {
    match s {
        "buy" | "sell" => Ok(()),
        _ => Err(ApiError::BadRequest(format!(
            "side must be buy|sell; got {s}"
        ))),
    }
}

fn validate_order_type(s: &str) -> ApiResult<()> {
    match s {
        "limit" | "stop" | "stop_limit" => Ok(()),
        _ => Err(ApiError::BadRequest(format!(
            "order_type must be limit|stop|stop_limit; got {s}"
        ))),
    }
}

fn validate_tif(s: &str) -> ApiResult<()> {
    match s {
        "gtc" | "day" | "gtd" => Ok(()),
        _ => Err(ApiError::BadRequest(format!(
            "time_in_force must be gtc|day|gtd; got {s}"
        ))),
    }
}

fn validate_status(s: &str) -> ApiResult<()> {
    match s {
        "open" | "filled" | "cancelled" | "expired" => Ok(()),
        _ => Err(ApiError::BadRequest(format!(
            "status must be open|filled|cancelled|expired; got {s}"
        ))),
    }
}

/// Per-type price-field invariant:
///   - `limit`      → must have limit_price, no stop_price
///   - `stop`       → must have stop_price, no limit_price
///   - `stop_limit` → must have both
fn validate_price_fields(
    order_type: &str,
    limit_price: Option<rust_decimal::Decimal>,
    stop_price: Option<rust_decimal::Decimal>,
) -> ApiResult<()> {
    match order_type {
        "limit" => {
            if limit_price.is_none() {
                return Err(ApiError::BadRequest(
                    "limit order requires limit_price".to_string(),
                ));
            }
        }
        "stop" => {
            if stop_price.is_none() {
                return Err(ApiError::BadRequest(
                    "stop order requires stop_price".to_string(),
                ));
            }
        }
        "stop_limit" => {
            if limit_price.is_none() || stop_price.is_none() {
                return Err(ApiError::BadRequest(
                    "stop_limit order requires both limit_price and stop_price".to_string(),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_ts(s: &str, field: &str) -> ApiResult<jiff::Timestamp> {
    s.parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("{field}: {e}")))
}
