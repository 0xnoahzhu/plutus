//! Admin-only routes. The `/admin/*` prefix is gated by [`require_admin`];
//! reaching these handlers without an admin session returns 403.

pub mod brokers;
pub mod users;

use plutus_core::audit::Actor;

use crate::error::{ApiError, ApiResult};

/// Reject the request unless the actor is the env-configured admin.
pub fn require_admin(actor: &Actor) -> ApiResult<()> {
    if actor.is_admin() {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}
