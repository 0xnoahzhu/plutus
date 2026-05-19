//! Server-side session records for browser logins. The browser cookie holds
//! the random session id; this row holds the identity it resolves to and an
//! expiry. Sessions can be revoked by deleting the row.
//!
//! `is_admin == true` rows have `user_id == 0` and `username == PLUTUS_ADMIN_USERNAME`
//! at the moment the session was created — admin doesn't have a `users` row.

#[derive(Debug, toasty::Model)]
#[table = "web_sessions"]
pub struct WebSession {
    #[key]
    pub id: String, // 32-byte base64-url-no-pad random
    /// `0` when `is_admin == true`; otherwise the `users.id` row this session
    /// authenticates.
    pub user_id: i64,
    pub is_admin: bool,
    pub username: String,
    pub created_at: jiff::Timestamp,
    pub expires_at: jiff::Timestamp,
}
