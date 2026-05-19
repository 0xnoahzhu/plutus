//! End-user accounts. The admin account does NOT live here — admin credentials
//! come from `PLUTUS_ADMIN_USERNAME` / `PLUTUS_ADMIN_PASSWORD` env vars and
//! authenticate against the env directly, never the DB.
//!
//! `password_reset_required` is set when the admin resets a user's password;
//! the next login forces the user through `/change-password` before any
//! data-bearing route works.

#[derive(Debug, toasty::Model)]
#[table = "users"]
pub struct User {
    #[key]
    #[auto]
    pub id: i64,
    #[unique]
    pub username: String,
    /// Argon2id hash. When `password_reset_required` is true the existing
    /// hash is still valid for the *first* login that lands on
    /// `/change-password`, but other endpoints reject the actor.
    pub password_hash: String,
    pub password_reset_required: bool,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
