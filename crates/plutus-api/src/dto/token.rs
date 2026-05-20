use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::ApiToken;

/// A long-lived API token bound to one user. Send as
/// `Authorization: Bearer <token_plain>`. Created via `POST /tokens`
/// (user-owned) or `POST /admin/tokens` (admin-grade — has no `user_id`
/// and authenticates as admin).
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenOut {
    /// Primary key.
    pub id: i64,
    /// User-chosen label (e.g. `daily-briefing-bot`).
    pub label: String,
    /// Plaintext token, stored alongside the hash so the listing UI can
    /// render a masked preview + copy button. `None` for legacy tokens
    /// minted before the `token_plain` column existed.
    pub token_plain: Option<String>,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp the token was last seen on a request.
    /// `null` until the first authenticated call.
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

/// `POST /tokens` body. Mints a new user-scoped token. Returns
/// [`TokenCreatedOut`] with the plaintext token in the response.
#[derive(Debug, Deserialize, ToSchema)]
pub struct TokenIn {
    /// User-chosen label.
    pub label: String,
}

/// `POST /tokens` response. Returns the plaintext `token` so the caller
/// can capture it once at creation. The plaintext is ALSO stored in
/// `api_tokens.token_plain` so the listing UI can show it later — see
/// [`TokenOut::token_plain`].
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenCreatedOut {
    /// Primary key.
    pub id: i64,
    /// User-chosen label.
    pub label: String,
    /// Plaintext token. Set this as `Authorization: Bearer <token>` on
    /// future requests.
    pub token: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
}
