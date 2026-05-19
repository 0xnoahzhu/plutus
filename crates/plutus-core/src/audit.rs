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
    /// Authenticated browser session (master password verified).
    Web,
    /// Bearer-token-authenticated API caller.
    ApiToken,
    /// Anonymous request, allowed when `PLUTUS_API_REQUIRE_AUTH=false`.
    Anonymous,
    /// Internal system operation (e.g. background job, migration).
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub kind: ActorKind,
    /// API token id, when `kind == ApiToken`. None otherwise.
    pub token_id: Option<i64>,
    /// Human-readable label for the audit row (e.g. token label, "web", "system").
    pub label: String,
}

impl Actor {
    #[must_use]
    pub fn anonymous() -> Self {
        Self {
            kind: ActorKind::Anonymous,
            token_id: None,
            label: "anonymous".into(),
        }
    }

    #[must_use]
    pub fn web() -> Self {
        Self {
            kind: ActorKind::Web,
            token_id: None,
            label: "web".into(),
        }
    }

    #[must_use]
    pub fn api_token(id: i64, label: impl Into<String>) -> Self {
        Self {
            kind: ActorKind::ApiToken,
            token_id: Some(id),
            label: label.into(),
        }
    }

    #[must_use]
    pub fn system() -> Self {
        Self {
            kind: ActorKind::System,
            token_id: None,
            label: "system".into(),
        }
    }
}
