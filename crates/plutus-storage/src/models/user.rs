//! End-user accounts. The admin account does NOT live here — admin credentials
//! come from `PLUTUS_ADMIN_USERNAME` / `PLUTUS_ADMIN_PASSWORD` env vars and
//! authenticate against the env directly, never the DB.
//!
//! `password_reset_required` is set when the admin resets a user's password;
//! the next login forces the user through `/change-password` before any
//! data-bearing route works.
//!
//! `allowed_countries` is a CSV of two-letter country codes (e.g. `"US,HK"`)
//! that scopes which market tabs the user sees in the web UI. Toasty 0.6
//! doesn't speak PostgreSQL arrays, and JSONB would be overkill for a
//! bounded 3-element set, so we store the canonical join with `,` and the
//! `country_codes()` helper splits on read. Validation lives at the API
//! DTO boundary — by the time a value reaches storage it's already been
//! checked against the `{US, HK, CN}` set.

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
    /// Canonical CSV form, e.g. `"US,HK,CN"`. Empty string is never valid
    /// at this layer — the DTO enforces ≥1 country before write.
    pub allowed_countries: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

impl User {
    /// Parse the stored CSV into a `Vec<String>`. Empty / whitespace
    /// entries are dropped, so a malformed cell (e.g. `"US,"`) still
    /// yields a sane list rather than a fake "" country.
    pub fn country_codes(&self) -> Vec<String> {
        self.allowed_countries
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
