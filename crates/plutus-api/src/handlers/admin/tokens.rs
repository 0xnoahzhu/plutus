//! Admin: manage admin-grade API tokens. These authenticate as the
//! env-configured admin (full `/admin/*` access, no per-user data scope)
//! and let automation — e.g. the hermes agent doing user provisioning
//! or broker housekeeping — call admin endpoints without a cookie
//! session.
//!
//! Distinct from `handlers::tokens` which only mints regular per-user
//! tokens. Both flavors live in the same `api_tokens` table, separated
//! by the `is_admin` flag.

use axum::extract::{Path, State};
use axum::Json;

use plutus_core::audit::Actor;

use crate::dto::token::{TokenCreatedOut, TokenIn, TokenOut};
use crate::error::ApiResult;
use crate::handlers::admin::require_admin;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> ApiResult<Json<Vec<TokenOut>>> {
    require_admin(&actor.0)?;
    let rows = plutus_storage::queries::tokens::list_admin(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<TokenIn>,
) -> ApiResult<Json<TokenCreatedOut>> {
    require_admin(&actor.0)?;
    let plain = crate::auth::token::generate();
    // user_id = 0 because admin isn't a row in `users`. is_admin = true
    // is what makes the middleware resolve this token to the Admin actor.
    let row = plutus_storage::queries::tokens::create(&state.db, 0, true, &input.label, &plain)
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
    require_admin(&actor.0)?;
    plutus_storage::queries::tokens::revoke(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
