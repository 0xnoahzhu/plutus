use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::ApiToken;

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenOut {
    pub id: i64,
    pub label: String,
    /// Plaintext token, returned so the UI list can render a masked
    /// preview + copy button. `None` for legacy tokens minted before the
    /// `token_plain` column existed — those render as "—" without a copy
    /// affordance. See `models/api_token.rs` for the trade-off rationale.
    pub token_plain: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

impl From<ApiToken> for TokenOut {
    fn from(t: ApiToken) -> Self {
        Self {
            id: t.id,
            label: t.label,
            token_plain: t.token_plain,
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
