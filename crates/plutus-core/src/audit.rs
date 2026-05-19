//! Audit primitives. Storage layer materializes these into the `audit_log` table.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    /// Authenticated browser session for a regular user.
    Web,
    /// Bearer-token-authenticated API caller. The token is scoped to one user.
    ApiToken,
    /// Admin session. Admin credentials live entirely in env vars
    /// (`PLUTUS_ADMIN_USERNAME` / `PLUTUS_ADMIN_PASSWORD`); admin is NOT a row
    /// in the `users` table and therefore has no `user_id`.
    Admin,
    /// Anonymous request, allowed when `PLUTUS_API_REQUIRE_AUTH=false`.
    Anonymous,
    /// Internal system operation (e.g. background job, migration).
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub kind: ActorKind,
    /// The end-user this request acts on behalf of. `Some` for `Web` and
    /// `ApiToken` actors, `None` for `Admin`, `Anonymous`, and `System`.
    pub user_id: Option<i64>,
    /// API token id, when `kind == ApiToken`. None otherwise.
    pub token_id: Option<i64>,
    /// Human-readable label for the audit row (username for Web/Admin, token
    /// label for ApiToken, "anonymous" / "system" otherwise).
    pub label: String,
}

impl Actor {
    #[must_use]
    pub fn anonymous() -> Self {
        Self {
            kind: ActorKind::Anonymous,
            user_id: None,
            token_id: None,
            label: "anonymous".into(),
        }
    }

    /// Web session for a regular user. `id` is the row id from the `users`
    /// table; `username` becomes the audit label.
    #[must_use]
    pub fn user(id: i64, username: impl Into<String>) -> Self {
        Self {
            kind: ActorKind::Web,
            user_id: Some(id),
            token_id: None,
            label: username.into(),
        }
    }

    /// Bearer-token-authenticated request on behalf of a user. `user_id` is
    /// the token's owner, `token_id` is the token row, `label` is the token
    /// label (used for audit display).
    #[must_use]
    pub fn api_token(user_id: i64, token_id: i64, label: impl Into<String>) -> Self {
        Self {
            kind: ActorKind::ApiToken,
            user_id: Some(user_id),
            token_id: Some(token_id),
            label: label.into(),
        }
    }

    /// Admin session. No user_id because admin is env-only, not a DB row.
    #[must_use]
    pub fn admin(username: impl Into<String>) -> Self {
        Self {
            kind: ActorKind::Admin,
            user_id: None,
            token_id: None,
            label: username.into(),
        }
    }

    #[must_use]
    pub fn system() -> Self {
        Self {
            kind: ActorKind::System,
            user_id: None,
            token_id: None,
            label: "system".into(),
        }
    }

    /// True when this actor is the env-configured admin.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        matches!(self.kind, ActorKind::Admin)
    }
}
