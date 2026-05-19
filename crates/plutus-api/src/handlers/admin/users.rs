//! Admin: manage end-user accounts. Create, list, force-reset password,
//! delete. Admin itself is NOT in the `users` table — its credentials come
//! from env vars and are not affected by anything here.

use axum::extract::{Path, State};
use axum::Json;

use plutus_core::audit::Actor;

use crate::auth::password;
use crate::dto::user::{AdminCreateUserIn, AdminResetPasswordIn, UserOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::admin::require_admin;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> ApiResult<Json<Vec<UserOut>>> {
    require_admin(&actor.0)?;
    let rows = plutus_storage::queries::users::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<AdminCreateUserIn>,
) -> ApiResult<Json<UserOut>> {
    require_admin(&actor.0)?;
    let username = input.username.trim();
    if username.is_empty() {
        return Err(ApiError::BadRequest("username is required".into()));
    }
    if input.password.is_empty() {
        return Err(ApiError::BadRequest("password is required".into()));
    }
    if state.is_admin_username(username) {
        return Err(ApiError::Conflict(
            "username conflicts with the admin account".into(),
        ));
    }
    if plutus_storage::queries::users::find_by_username(&state.db, username)
        .await?
        .is_some()
    {
        return Err(ApiError::Conflict("username already taken".into()));
    }
    let hash = password::hash(&input.password)
        .map_err(|e| ApiError::Internal(format!("hash failed: {e}")))?;
    let row = plutus_storage::queries::users::create(&state.db, username, &hash).await?;
    Ok(Json(row.into()))
}

/// Force-reset the user's password. Writes the new hash, flips
/// `password_reset_required=true` so the next login lands them on
/// `/change-password`, and invalidates all of the user's existing web
/// sessions. API tokens are NOT invalidated here — admin can revoke them
/// individually if compromise is suspected.
pub async fn reset_password(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(input): Json<AdminResetPasswordIn>,
) -> ApiResult<Json<UserOut>> {
    require_admin(&actor.0)?;
    if input.password.is_empty() {
        return Err(ApiError::BadRequest("password is required".into()));
    }
    let hash = password::hash(&input.password)
        .map_err(|e| ApiError::Internal(format!("hash failed: {e}")))?;
    let row = plutus_storage::queries::users::admin_reset(&state.db, id, &hash).await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    require_admin(&actor.0)?;
    plutus_storage::queries::users::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
