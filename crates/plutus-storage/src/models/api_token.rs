//! API bearer tokens. Stored as sha256 hashes (token text only returned once,
//! at creation). Two flavors share this table:
//!
//! - Regular tokens (`is_admin = false`): authenticate as the owning
//!   `user_id`. Minted by the user via `POST /tokens` while logged in to
//!   the web UI. Bearer requests resolve to a `Web`-equivalent actor scoped
//!   to that user's data.
//! - Admin tokens (`is_admin = true`): authenticate as the env-configured
//!   admin. `user_id` is `0` (the orphan sentinel — admin has no row in
//!   `users`). Minted only by admin via `POST /admin/tokens`. Bearer
//!   requests resolve to an `Admin` actor with full `/admin/*` access.

#[derive(Debug, toasty::Model)]
#[table = "api_tokens"]
pub struct ApiToken {
    #[key]
    #[auto]
    pub id: i64,
    /// Owner of the token. `0` for admin-grade tokens (admin isn't a row
    /// in `users`); otherwise the row id of a regular user.
    pub user_id: i64,
    /// When true, the bearer is granted the env-configured admin's
    /// authority — full `/admin/*` access, no per-user data scope.
    pub is_admin: bool,
    pub label: String,
    #[unique]
    pub token_hash: String,
    pub created_at: jiff::Timestamp,
    pub last_used_at: Option<jiff::Timestamp>,
    pub revoked_at: Option<jiff::Timestamp>,
}
