//! Admin: manage end-user accounts. Create, list, force-reset password,
//! delete. Admin itself is NOT in the `users` table — its credentials come
//! from env vars and are not affected by anything here.

use axum::extract::{Path, State};
use axum::Json;

use plutus_core::audit::Actor;

use crate::auth::password;
use crate::dto::user::{
    AdminCreateUserIn, AdminResetPasswordIn, AdminUpdateCountriesIn, UserOut,
};
use crate::error::{ApiError, ApiResult};
use crate::handlers::admin::require_admin;
use crate::state::AppState;

/// Countries the system understands. Kept here (not in the DTO file) so
/// the validation message can stay close to the handler.
const SUPPORTED_COUNTRIES: &[&str] = &["US", "HK", "CN"];

/// Validate a country-allowlist payload. Returns the canonicalized list
/// (deduplicated, in input order) on success. Empty input → 400 because
/// a user with no countries can't see any market tab.
fn validate_countries(input: &[String]) -> ApiResult<Vec<String>> {
    if input.is_empty() {
        return Err(ApiError::BadRequest(
            "at least one country is required".into(),
        ));
    }
    let mut seen: Vec<String> = Vec::new();
    for raw in input {
        let code = raw.trim().to_uppercase();
        if !SUPPORTED_COUNTRIES.iter().any(|c| *c == code) {
            return Err(ApiError::BadRequest(format!(
                "country must be one of {}; got {raw:?}",
                SUPPORTED_COUNTRIES.join(", "),
            )));
        }
        if !seen.iter().any(|s| s == &code) {
            seen.push(code);
        }
    }
    Ok(seen)
}

/// Default allowlist used when admin omits the field on create.
fn default_countries() -> Vec<String> {
    SUPPORTED_COUNTRIES.iter().map(|s| s.to_string()).collect()
}

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
    // Optional on the wire; default to all three countries when absent
    // so admin clients that don't know about this field don't accidentally
    // create users with an empty allowlist.
    let allowed_countries = match input.allowed_countries.as_deref() {
        Some(list) => validate_countries(list)?,
        None => default_countries(),
    };
    let hash = password::hash(&input.password)
        .map_err(|e| ApiError::Internal(format!("hash failed: {e}")))?;
    let row = plutus_storage::queries::users::create(
        &state.db,
        username,
        &hash,
        &allowed_countries,
    )
    .await?;
    Ok(Json(row.into()))
}

/// Admin sets the user's market-country allowlist. Replaces (not merges)
/// the current list. Subsequent `/auth/me` calls for that user reflect
/// the new scope on the next request.
pub async fn update_countries(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(input): Json<AdminUpdateCountriesIn>,
) -> ApiResult<Json<UserOut>> {
    require_admin(&actor.0)?;
    let validated = validate_countries(&input.allowed_countries)?;
    let row = plutus_storage::queries::users::set_countries(&state.db, id, &validated).await?;
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
