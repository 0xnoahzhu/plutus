use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::ApiToken;

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenOut {
    pub id: i64,
    pub label: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

impl From<ApiToken> for TokenOut {
    fn from(t: ApiToken) -> Self {
        Self {
            id: t.id,
            label: t.label,
            created_at: t.created_at.to_string(),
            last_used_at: t.last_used_at.map(|t| t.to_string()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TokenIn {
    pub label: String,
}

/// Returned exactly once at creation. After this, only the prefix is visible.
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenCreatedOut {
    pub id: i64,
    pub label: String,
    pub token: String,
    pub created_at: String,
}
