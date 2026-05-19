use axum::extract::{Path, State};
use axum::Json;

use plutus_core::audit::{Actor, ActorKind};

use crate::dto::token::{TokenCreatedOut, TokenIn, TokenOut};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Web-only: prevent agents from spawning more tokens.
fn require_web(actor: &Actor) -> ApiResult<()> {
    match actor.kind {
        ActorKind::Web => Ok(()),
        ActorKind::Anonymous if cfg!(debug_assertions) => Ok(()),
        _ => Err(ApiError::Forbidden),
    }
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> ApiResult<Json<Vec<TokenOut>>> {
    require_web(&actor.0)?;
    let rows = plutus_storage::queries::tokens::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<TokenIn>,
) -> ApiResult<Json<TokenCreatedOut>> {
    require_web(&actor.0)?;
    let plain = crate::auth::token::generate();
    let row = plutus_storage::queries::tokens::create(&state.db, &input.label, &plain).await?;
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
    require_web(&actor.0)?;
    plutus_storage::queries::tokens::revoke(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
