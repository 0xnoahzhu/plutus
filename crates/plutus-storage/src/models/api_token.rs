//! API bearer tokens. Stored as sha256 hashes (token text only returned once,
//! at creation). Each token belongs to one user and authorizes that user's
//! data. Tokens are minted per-actor via `POST /tokens` while logged in.

#[derive(Debug, toasty::Model)]
#[table = "api_tokens"]
pub struct ApiToken {
    #[key]
    #[auto]
    pub id: i64,
    /// Owner of this token. Tokens authenticate as their owner — bearer
    /// requests carry this user_id through to the handler.
    pub user_id: i64,
    pub label: String,
    #[unique]
    pub token_hash: String,
    pub created_at: jiff::Timestamp,
    pub last_used_at: Option<jiff::Timestamp>,
    pub revoked_at: Option<jiff::Timestamp>,
}
