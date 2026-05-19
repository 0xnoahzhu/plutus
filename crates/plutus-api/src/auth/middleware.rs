//! Auth middleware. Inspects each request and either:
//! - identifies the caller (cookie session → DB row → user/admin actor;
//!   bearer token → api_token row → user actor), or
//! - allows the request through as `Anonymous` when `require_auth=false`, or
//! - rejects with 401 when `require_auth=true` and no credential is present.

use axum::extract::Request;
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;

use plutus_core::audit::Actor;

use crate::auth::{session, token};
use crate::state::AppState;

pub async fn extract_actor_inner(state: AppState, mut req: Request, next: Next) -> Response {
    // Inspect headers first so we don't hold a &Request across an await.
    let actor_opt = identify(&state, req.headers()).await;
    if let Some(actor) = actor_opt {
        req.extensions_mut().insert(actor);
        return next.run(req).await;
    }
    if state.require_auth {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    req.extensions_mut().insert(Actor::anonymous());
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
            return Some(Actor::api_token(row.user_id, row.id, row.label));
        }
    }
    None
}
