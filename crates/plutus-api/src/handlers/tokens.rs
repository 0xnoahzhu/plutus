use axum::extract::{Path, State};
use axum::Json;

use plutus_core::audit::{Actor, ActorKind};

use crate::dto::token::{TokenCreatedOut, TokenIn, TokenOut};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Tokens are minted by regular users for themselves via the web UI. Bearer
/// auth deliberately can't mint more tokens (no token escalation). Admin has
/// no per-user data of its own, so we don't let admin mint tokens either —
/// admin's job is account management, not data access.
fn require_user_session(actor: &Actor) -> ApiResult<i64> {
    match actor.kind {
        ActorKind::Web => actor.user_id.ok_or(ApiError::Forbidden),
        ActorKind::Anonymous if cfg!(debug_assertions) => Ok(0),
        _ => Err(ApiError::Forbidden),
    }
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> ApiResult<Json<Vec<TokenOut>>> {
    let user_id = require_user_session(&actor.0)?;
    let rows = plutus_storage::queries::tokens::list_for_user(&state.db, user_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<TokenIn>,
) -> ApiResult<Json<TokenCreatedOut>> {
    let user_id = require_user_session(&actor.0)?;
    let plain = crate::auth::token::generate();
    let row = plutus_storage::queries::tokens::create(&state.db, user_id, &input.label, &plain)
        .await?;
    Ok(Json(TokenCreatedOut {
        id: row.id,
        label: row.label,
        token: plain,
        created_at: row.created_at.to_string(),
    }))
}

pub async fn revoke(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let _ = require_user_session(&actor.0)?;
    plutus_storage::queries::tokens::revoke(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
