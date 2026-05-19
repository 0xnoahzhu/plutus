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
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginOut {
    pub ok: bool,
}

pub async fn login(
    State(state): State<AppState>,
    cookies: CookieJar,
    Json(input): Json<LoginIn>,
) -> ApiResult<(CookieJar, Json<LoginOut>)> {
    if state.master_password_hash.is_empty() {
        return Err(ApiError::Internal(
            "PLUTUS_MASTER_PASSWORD_HASH is not configured".into(),
        ));
    }
    if !password::verify(&input.password, &state.master_password_hash) {
        return Err(ApiError::Unauthorized);
    }
    let signed = session::sign(&state.cookie_secret, "ok");
    let cookie = Cookie::build((session::COOKIE_NAME, signed))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    Ok((cookies.add(cookie), Json(LoginOut { ok: true })))
}

pub async fn logout(cookies: CookieJar) -> (CookieJar, Json<LoginOut>) {
    let cookie = Cookie::build((session::COOKIE_NAME, ""))
        .path("/")
        .build();
    (cookies.remove(cookie), Json(LoginOut { ok: true }))
}

#[derive(Serialize)]
pub struct MeOut {
    pub kind: String,
    pub label: String,
    pub token_id: Option<i64>,
}

pub async fn me(actor: axum::extract::Extension<Actor>) -> Json<MeOut> {
    let actor = actor.0;
    let kind = match actor.kind {
        plutus_core::audit::ActorKind::Web => "web",
        plutus_core::audit::ActorKind::ApiToken => "api_token",
        plutus_core::audit::ActorKind::Anonymous => "anonymous",
        plutus_core::audit::ActorKind::System => "system",
    }
    .to_string();
    Json(MeOut {
        kind,
        label: actor.label,
        token_id: actor.token_id,
    })
}
