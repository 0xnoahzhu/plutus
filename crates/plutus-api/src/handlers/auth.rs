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

    // Admin path: match against env-var plaintext. Admin is NOT a row in `users`.
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
}

pub async fn me(actor: axum::extract::Extension<Actor>) -> Json<MeOut> {
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
    Json(MeOut {
        kind,
        label: actor.label,
        user_id: actor.user_id,
        token_id: actor.token_id,
        is_admin,
    })
}

fn build_session_cookie(sid: &str) -> Cookie<'static> {
    Cookie::build((session::COOKIE_NAME, sid.to_string()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}
