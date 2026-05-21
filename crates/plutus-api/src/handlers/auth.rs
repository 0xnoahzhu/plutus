//! Auth endpoints. `/auth/login` accepts a username + password; depending on
//! whether the username matches `PLUTUS_ADMIN_USERNAME`, the request is
//! verified against the env-var plaintext (admin) or against the Argon2 hash
//! on the matching `users` row. Either way, success creates a session row in
//! `web_sessions` and returns a `plutus_session=<id>` cookie.

use axum::extract::State;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};

use plutus_core::audit::Actor;

use crate::auth::{password, session};
use crate::dto::user::ChangePasswordIn;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginIn {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginOut {
    pub ok: bool,
    /// True when the authenticated user must change their password before any
    /// data-bearing route works (admin reset flow). The frontend uses this to
    /// route immediately to `/change-password`.
    pub password_reset_required: bool,
    pub is_admin: bool,
    pub username: String,
}

pub async fn login(
    State(state): State<AppState>,
    cookies: CookieJar,
    Json(input): Json<LoginIn>,
) -> ApiResult<(CookieJar, Json<LoginOut>)> {
    let username = input.username.trim();
    if username.is_empty() || input.password.is_empty() {
        return Err(ApiError::BadRequest("username and password are required".into()));
    }

    // Admin path: match against env-var plaintext. Admin is NOT a row in
    // `users` — credentials are env-only. The web_session row that anchors
    // the admin's browser cookie uses `user_id = 0`, which points at the
    // sentinel `__admin` row inserted during migration so the FK to
    // `users(id)` is valid. See models/audit_log.rs for the full
    // convention.
    if state.is_admin_username(username) {
        if input.password != state.admin_password {
            return Err(ApiError::Unauthorized);
        }
        let sid = session::generate_id();
        plutus_storage::queries::web_sessions::create(
            &state.db,
            &sid,
            /* user_id */ 0,
            /* is_admin */ true,
            username,
        )
        .await?;
        return Ok((
            cookies.add(build_session_cookie(&sid)),
            Json(LoginOut {
                ok: true,
                password_reset_required: false,
                is_admin: true,
                username: username.to_string(),
            }),
        ));
    }

    // Regular user path: lookup + Argon2 verify.
    let Some(user) =
        plutus_storage::queries::users::find_by_username(&state.db, username).await?
    else {
        return Err(ApiError::Unauthorized);
    };
    if !password::verify(&input.password, &user.password_hash) {
        return Err(ApiError::Unauthorized);
    }
    let sid = session::generate_id();
    plutus_storage::queries::web_sessions::create(
        &state.db,
        &sid,
        user.id,
        false,
        &user.username,
    )
    .await?;
    Ok((
        cookies.add(build_session_cookie(&sid)),
        Json(LoginOut {
            ok: true,
            password_reset_required: user.password_reset_required,
            is_admin: false,
            username: user.username,
        }),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    cookies: CookieJar,
) -> ApiResult<(CookieJar, Json<LogoutOut>)> {
    if let Some(c) = cookies.get(session::COOKIE_NAME) {
        let _ = plutus_storage::queries::web_sessions::delete(&state.db, c.value()).await;
    }
    let clear = Cookie::build((session::COOKIE_NAME, ""))
        .path("/")
        .build();
    Ok((cookies.remove(clear), Json(LogoutOut { ok: true })))
}

#[derive(Serialize)]
pub struct LogoutOut {
    pub ok: bool,
}

#[derive(Serialize)]
pub struct MeOut {
    pub kind: String,
    pub label: String,
    pub user_id: Option<i64>,
    pub token_id: Option<i64>,
    pub is_admin: bool,
    /// Two-letter country codes the user is scoped to. For DB-backed
    /// users (Web + ApiToken) this is the list stored on the `users`
    /// row. For admin / anonymous / system actors there's no DB row, so
    /// the list is empty — callers should treat "no scope" as "see
    /// everything" rather than "nothing".
    pub allowed_countries: Vec<String>,
}

pub async fn me(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> Json<MeOut> {
    let actor = actor.0;
    let is_admin = actor.is_admin();
    let kind = match actor.kind {
        plutus_core::audit::ActorKind::Web => "web",
        plutus_core::audit::ActorKind::ApiToken => "api_token",
        plutus_core::audit::ActorKind::Admin => "admin",
        plutus_core::audit::ActorKind::Anonymous => "anonymous",
        plutus_core::audit::ActorKind::System => "system",
    }
    .to_string();
    // Look up the country scope for DB-backed actors. We swallow errors
    // and fall through to an empty list — the worst case is the web UI
    // briefly showing every country tab, which is harmless and recovers
    // on the next request.
    let allowed_countries = match actor.user_id {
        Some(uid) => plutus_storage::queries::users::get(&state.db, uid)
            .await
            .map(|u| u.country_codes())
            .unwrap_or_default(),
        None => Vec::new(),
    };
    Json(MeOut {
        kind,
        label: actor.label,
        user_id: actor.user_id,
        token_id: actor.token_id,
        is_admin,
        allowed_countries,
    })
}

fn build_session_cookie(sid: &str) -> Cookie<'static> {
    Cookie::build((session::COOKIE_NAME, sid.to_string()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

#[derive(Serialize)]
pub struct ChangePasswordOut {
    pub ok: bool,
}

/// Self-service password change. Verifies the current password (or accepts
/// any value when `password_reset_required=true` — the admin reset already
/// invalidated the previous password's authority), then writes the new hash
/// and clears the reset flag.
pub async fn change_password(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<ChangePasswordIn>,
) -> ApiResult<Json<ChangePasswordOut>> {
    let actor = actor.0;
    let user_id = match (actor.kind, actor.user_id) {
        (plutus_core::audit::ActorKind::Web, Some(id)) => id,
        _ => return Err(ApiError::Forbidden),
    };
    if input.new_password != input.new_password_confirm {
        return Err(ApiError::BadRequest("new passwords do not match".into()));
    }
    if input.new_password.is_empty() {
        return Err(ApiError::BadRequest("new password is required".into()));
    }
    let user = plutus_storage::queries::users::get(&state.db, user_id).await?;
    // When a reset is pending the existing hash is treated as "must be replaced",
    // so we accept the current_password value without verifying. Otherwise the
    // user must prove they know it.
    if !user.password_reset_required && !password::verify(&input.current_password, &user.password_hash) {
        return Err(ApiError::Unauthorized);
    }
    let new_hash = password::hash(&input.new_password)
        .map_err(|e| ApiError::Internal(format!("hash failed: {e}")))?;
    plutus_storage::queries::users::set_password(&state.db, user_id, &new_hash).await?;
    Ok(Json(ChangePasswordOut { ok: true }))
}
