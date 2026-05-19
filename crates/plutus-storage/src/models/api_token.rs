//! API bearer tokens. Stored as sha256 hashes (token text only returned once,
//! at creation). Phase 0 gives every token full access; scopes can be added
//! later.

#[derive(Debug, toasty::Model)]
#[table = "api_tokens"]
pub struct ApiToken {
    #[key]
    #[auto]
    pub id: i64,
    pub label: String,
    #[unique]
    pub token_hash: String,
    pub created_at: jiff::Timestamp,
    pub last_used_at: Option<jiff::Timestamp>,
    pub revoked_at: Option<jiff::Timestamp>,
}
