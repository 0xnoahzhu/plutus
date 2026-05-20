//! API bearer tokens. Two flavors share this table:
//!
//! - Regular tokens (`is_admin = false`): authenticate as the owning
//!   `user_id`. Minted by the user via `POST /tokens` while logged in to
//!   the web UI. Bearer requests resolve to a `Web`-equivalent actor scoped
//!   to that user's data.
//! - Admin tokens (`is_admin = true`): authenticate as the env-configured
//!   admin. `user_id` is `0` (the orphan sentinel — admin has no row in
//!   `users`). Minted only by admin via `POST /admin/tokens`. Bearer
//!   requests resolve to an `Admin` actor with full `/admin/*` access.
//!
//! Tokens are hard-deleted when revoked (no soft `revoked_at` flag): the row
//! is gone, the hash no longer matches anything, the bearer immediately
//! starts getting 401.
//!
//! ## Plaintext storage trade-off
//!
//! `token_plain` carries the literal token string alongside `token_hash`.
//! The hash is what auth lookups use (a sha256 of the bearer); the plaintext
//! is for the UI so the user can re-copy a previously minted token from
//! the list view without having to regenerate. Authentication still goes
//! through the hash so a leaked plaintext column doesn't widen the attack
//! surface compared to having both: an attacker with DB read access could
//! already authenticate by submitting the hash directly via a forged path,
//! and a personal-app threat model accepts that DB compromise = all tokens
//! compromised either way.
//!
//! Legacy tokens minted before this column existed have `token_plain = NULL`
//! and render as "—" in the UI (no copy button) — the auth path still works
//! because the hash is intact, but the user has to mint a new token if they
//! want a copy-able one.

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
    /// SHA-256 of the plaintext. Used for auth lookup
    /// (`find_active_by_plain` hashes the bearer and matches here).
    #[unique]
    pub token_hash: String,
    /// Literal plaintext, kept so the list view can show + copy without
    /// regenerating. Nullable so pre-existing tokens (minted before this
    /// column existed) still load — they just render without a copy
    /// button. See the module-level "Plaintext storage trade-off".
    pub token_plain: Option<String>,
    pub created_at: jiff::Timestamp,
    pub last_used_at: Option<jiff::Timestamp>,
}
