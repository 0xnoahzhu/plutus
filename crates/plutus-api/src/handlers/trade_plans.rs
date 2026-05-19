//! Per-user trade plans: header CRUD + nested price-level CRUD. All
//! routes here go through `require_user`; admin actors get 403 (admin
//! has no per-user data of its own).

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::trade_plan::{
    TradePlanIn, TradePlanLevelIn, TradePlanLevelOut, TradePlanLevelPatch, TradePlanOut,
    TradePlanPatch,
};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub stock_id: Option<i64>,
    pub status: Option<String>,
}

// ── Plans ────────────────────────────────────────────────────────────────

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
) -> ApiResult<Json<Vec<TradePlanOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::trade_plans::list(
        &state.db,
        plutus_storage::queries::trade_plans::ListFilter {
            user_id,
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
) -> ApiResult<Json<TradePlanOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::trade_plans::get(&state.db, user_id, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<TradePlanIn>,
) -> ApiResult<Json<TradePlanOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::trade_plans::create(
        &state.db,
        plutus_storage::queries::trade_plans::NewPlan {
            user_id,
            stock_id: input.stock_id,
            rationale: input.rationale.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn update(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(patch): Json<TradePlanPatch>,
) -> ApiResult<Json<TradePlanOut>> {
    let user_id = require_user(&actor.0)?;
    if let Some(s) = patch.status.as_deref() {
        validate_plan_status(s)?;
    }
    let row = plutus_storage::queries::trade_plans::update(
        &state.db,
        user_id,
        id,
        plutus_storage::queries::trade_plans::PlanPatch {
            rationale: patch.rationale.as_ref().map(|o| o.as_deref()),
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
    plutus_storage::queries::trade_plans::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Levels ───────────────────────────────────────────────────────────────

pub async fn list_levels(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(plan_id): Path<i64>,
) -> ApiResult<Json<Vec<TradePlanLevelOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows =
        plutus_storage::queries::trade_plans::list_levels_for_plan(&state.db, user_id, plan_id)
            .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn add_level(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(plan_id): Path<i64>,
    Json(input): Json<TradePlanLevelIn>,
) -> ApiResult<Json<TradePlanLevelOut>> {
    let user_id = require_user(&actor.0)?;
    validate_level_kind(&input.kind)?;
    let row = plutus_storage::queries::trade_plans::add_level(
        &state.db,
        user_id,
        plutus_storage::queries::trade_plans::NewLevel {
            plan_id,
            kind: &input.kind,
            price: input.price,
            quantity: input.quantity,
            fraction_pct: input.fraction_pct,
            notes: input.notes.as_deref(),
            sort_order: input.sort_order,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn update_level(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(patch): Json<TradePlanLevelPatch>,
) -> ApiResult<Json<TradePlanLevelOut>> {
    let user_id = require_user(&actor.0)?;
    if let Some(kind) = patch.kind.as_deref() {
        validate_level_kind(kind)?;
    }
    if let Some(status) = patch.status.as_deref() {
        validate_level_status(status)?;
    }
    let row = plutus_storage::queries::trade_plans::update_level(
        &state.db,
        user_id,
        id,
        plutus_storage::queries::trade_plans::LevelPatch {
            kind: patch.kind.as_deref(),
            price: patch.price,
            quantity: patch.quantity,
            fraction_pct: patch.fraction_pct,
            notes: patch.notes.as_ref().map(|o| o.as_deref()),
            sort_order: patch.sort_order,
            status: patch.status.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete_level(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let user_id = require_user(&actor.0)?;
    plutus_storage::queries::trade_plans::delete_level(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Validators ───────────────────────────────────────────────────────────

fn validate_plan_status(s: &str) -> ApiResult<()> {
    match s {
        "active" | "closed" => Ok(()),
        _ => Err(ApiError::BadRequest(format!(
            "plan status must be active|closed; got {s}"
        ))),
    }
}

fn validate_level_kind(s: &str) -> ApiResult<()> {
    match s {
        "buy" | "stop_loss" | "take_profit" | "trim" => Ok(()),
        _ => Err(ApiError::BadRequest(format!(
            "level kind must be buy|stop_loss|take_profit|trim; got {s}"
        ))),
    }
}

fn validate_level_status(s: &str) -> ApiResult<()> {
    match s {
        "active" | "triggered" | "cancelled" => Ok(()),
        _ => Err(ApiError::BadRequest(format!(
            "level status must be active|triggered|cancelled; got {s}"
        ))),
    }
}
