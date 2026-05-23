//! Per-user unread state endpoints. The sidebar badge calls
//! `GET /unread/counts`; individual items are marked read as a side
//! effect of their detail GET handlers (see each entity's `get`).

use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;
use plutus_storage::queries::unread::{self, EntityKind};

use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

/// `GET /unread/counts` — `{ "<entity_type>": <n>, ... }`. Always contains
/// the full set of kinds (zero-filled) so the frontend doesn't have to
/// special-case missing keys.
pub async fn counts(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> ApiResult<Json<BTreeMap<&'static str, i64>>> {
    let user_id = require_user(&actor.0)?;
    let map = unread::counts(&state.db, user_id).await?;
    let mut out: BTreeMap<&'static str, i64> = BTreeMap::new();
    for kind in EntityKind::ALL {
        out.insert(kind.as_str(), map.get(kind).copied().unwrap_or(0));
    }
    Ok(Json(out))
}

#[derive(Debug, Deserialize)]
pub struct UnmarkPath {
    pub kind: String,
    pub id: i64,
}

/// `DELETE /reads/:kind/:id` — flip an item back to unread. Returns 404
/// if `kind` is unrecognized; idempotent otherwise.
pub async fn unmark(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(UnmarkPath { kind, id }): Path<UnmarkPath>,
) -> ApiResult<StatusCode> {
    let user_id = require_user(&actor.0)?;
    let Some(kind) = EntityKind::from_str(&kind) else {
        return Err(ApiError::BadRequest(format!("unknown entity kind: {kind}")));
    };
    unread::unmark_read(&state.db, user_id, kind, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
