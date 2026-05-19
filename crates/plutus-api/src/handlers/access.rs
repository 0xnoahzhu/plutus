//! Per-handler authorization helpers.
//!
//! Routes that serve per-user data go through [`require_user`] to extract
//! the calling actor's `user_id`. Admin actors (no `user_id`) get 403 —
//! admin's role is account management via `/admin/*`, not data access.
//!
//! In debug builds we let anonymous callers through with `user_id == 0`
//! (the "orphaned / pre-multi-user" sentinel) so local development with
//! `PLUTUS_API_REQUIRE_AUTH=false` keeps working.

use plutus_core::audit::{Actor, ActorKind};

use crate::error::{ApiError, ApiResult};

/// Sentinel user_id used for legacy single-user data and dev-mode anonymous
/// callers. Rows with this id remain visible only to the bucket itself.
pub const ORPHAN_USER_ID: i64 = 0;

/// Resolve the actor's data-owning user_id. Admin and other non-user actors
/// get 403.
pub fn require_user(actor: &Actor) -> ApiResult<i64> {
    match actor.kind {
        ActorKind::Web | ActorKind::ApiToken => {
            actor.user_id.ok_or(ApiError::Forbidden)
        }
        ActorKind::Anonymous if cfg!(debug_assertions) => Ok(ORPHAN_USER_ID),
        _ => Err(ApiError::Forbidden),
    }
}
