//! Auth middleware. Inspects each request and either:
//! - identifies the caller (cookie session → DB row → user/admin actor;
//!   bearer token → api_token row → user actor), or
//! - allows the request through as `Anonymous` when `require_auth=false`, or
//! - rejects with 401 when `require_auth=true` and no credential is present.
//!
//! When the resolved actor is a regular user whose row has
//! `password_reset_required=true`, every route except a small unlock-list
//! returns 403 with `error: "password_reset_required"`. The frontend uses
//! that signal to route the user into `/change-password`.

use axum::extract::Request;
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;

use plutus_core::audit::{Actor, ActorKind};

use crate::auth::{session, token};
use crate::state::AppState;

/// Path suffixes (matched against the unstripped request URI) that remain
/// reachable when the actor must change their password before anything else.
/// All other routes return 403 in that state.
const RESET_UNLOCKED_PATHS: &[&str] = &[
    "/api/v1/auth/me",
    "/api/v1/auth/logout",
    "/api/v1/auth/change-password",
    "/api/v1/healthz",
    "/api/v1/openapi.json",
    "/api/v1/docs",
];

pub async fn extract_actor_inner(state: AppState, mut req: Request, next: Next) -> Response {
    let actor_opt = identify(&state, req.headers()).await;
    let Some(actor) = actor_opt else {
        if state.require_auth {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        req.extensions_mut().insert(Actor::anonymous());
        return next.run(req).await;
    };

    // Password-reset gate: if this is a regular user with the flag set, only a
    // tiny set of routes is reachable. The lookup mirrors what the handler
    // would do anyway, so we don't burn a query unless the actor is a user.
    if matches!(actor.kind, ActorKind::Web | ActorKind::ApiToken) {
        if let Some(uid) = actor.user_id {
            if let Ok(user) = plutus_storage::queries::users::get(&state.db, uid).await {
                if user.password_reset_required
                    && !is_unlocked_during_reset(req.uri().path())
                {
                    return password_reset_required_response();
                }
            }
        }
    }

    req.extensions_mut().insert(actor);
    next.run(req).await
}

async fn identify(state: &AppState, headers: &HeaderMap) -> Option<Actor> {
    let cookies = CookieJar::from_headers(headers);
    let session_id = cookies
        .get(session::COOKIE_NAME)
        .map(|c| c.value().to_string());
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|v| token::parse_bearer(v).map(str::to_string));

    if let Some(id) = session_id {
        if !id.is_empty() {
            if let Ok(Some(row)) =
                plutus_storage::queries::web_sessions::find_active(&state.db, &id).await
            {
                if row.is_admin {
                    return Some(Actor::admin(row.username));
                }
                return Some(Actor::user(row.user_id, row.username));
            }
        }
    }
    if let Some(plain) = bearer {
        if let Ok(Some(row)) =
            plutus_storage::queries::tokens::find_active_by_plain(&state.db, &plain).await
        {
            // Admin-grade tokens authenticate as the env-configured admin —
            // full `/admin/*` access, no per-user data scope. Regular tokens
            // authenticate as their owning user.
            if row.is_admin {
                return Some(Actor {
                    kind: plutus_core::audit::ActorKind::Admin,
                    user_id: None,
                    token_id: Some(row.id),
                    label: row.label,
                });
            }
            return Some(Actor::api_token(row.user_id, row.id, row.label));
        }
    }
    None
}

fn is_unlocked_during_reset(path: &str) -> bool {
    RESET_UNLOCKED_PATHS.iter().any(|p| *p == path)
}

fn password_reset_required_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": "password_reset_required",
            "message": "Password change required before further access.",
        })),
    )
        .into_response()
}
