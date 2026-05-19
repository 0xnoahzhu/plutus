//! Admin: manage the shared `brokers` reference table. Regular users
//! pick from this list when creating an account but can't modify it.
//! Delete refuses to drop a broker with any account or broker_symbol
//! row still pointing at it — the storage layer returns `Conflict` →
//! 409 in that case.

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;

use plutus_core::audit::Actor;

use crate::dto::broker::BrokerOut;
use crate::error::{ApiError, ApiResult};
use crate::handlers::admin::require_admin;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminCreateBrokerIn {
    /// Short identifier (e.g. "IBKR", "MOOMOO_US"). Must be unique.
    pub code: String,
    /// Display name (e.g. "Interactive Brokers").
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminUpdateBrokerIn {
    pub name: String,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> ApiResult<Json<Vec<BrokerOut>>> {
    require_admin(&actor.0)?;
    let rows = plutus_storage::queries::brokers::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<AdminCreateBrokerIn>,
) -> ApiResult<Json<BrokerOut>> {
    require_admin(&actor.0)?;
    let code = input.code.trim();
    let name = input.name.trim();
    if code.is_empty() || name.is_empty() {
        return Err(ApiError::BadRequest("code and name are required".into()));
    }
    if plutus_storage::queries::brokers::get_by_code(&state.db, code)
        .await?
        .is_some()
    {
        return Err(ApiError::Conflict(format!("broker code '{code}' already exists")));
    }
    let row = plutus_storage::queries::brokers::create(&state.db, code, name).await?;
    Ok(Json(row.into()))
}

pub async fn update(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(input): Json<AdminUpdateBrokerIn>,
) -> ApiResult<Json<BrokerOut>> {
    require_admin(&actor.0)?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let row = plutus_storage::queries::brokers::update_name(&state.db, id, name).await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    require_admin(&actor.0)?;
    plutus_storage::queries::brokers::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
