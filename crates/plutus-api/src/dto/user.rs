use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::User;

/// A regular user account. Admin is env-only and NOT a row in this
/// table — admin's `UserOut` is synthesized at `/auth/me` time without
/// touching the DB.
#[derive(Debug, Serialize, ToSchema)]
pub struct UserOut {
    /// Primary key.
    pub id: i64,
    /// Login name. Case-sensitive. Unique.
    pub username: String,
    /// When `true`, every route except `/auth/me`, `/auth/logout`,
    /// `/auth/change-password`, `/healthz`, `/openapi.json`, `/docs`
    /// returns 403 until the user calls `/auth/change-password`. Set
    /// automatically by `POST /admin/users/{id}/reset-password`.
    pub password_reset_required: bool,
    /// Two-letter country codes the user is scoped to (e.g.
    /// `["US","HK"]`). Endpoints that list country-scoped data honor this
    /// allowlist. Empty list = admin actor (no scope; sees everything).
    /// Non-empty for every DB-backed user.
    pub allowed_countries: Vec<String>,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

impl From<User> for UserOut {
    fn from(u: User) -> Self {
        let allowed_countries = u.country_codes();
        Self {
            id: u.id,
            username: u.username,
            password_reset_required: u.password_reset_required,
            allowed_countries,
            created_at: u.created_at.to_string(),
            updated_at: u.updated_at.to_string(),
        }
    }
}

/// `POST /admin/users` body — admin-only.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminCreateUserIn {
    /// Login name. Case-sensitive. Must not equal `PLUTUS_ADMIN_USERNAME`
    /// (409 if it does) and must not already exist (409).
    pub username: String,
    /// Initial password. The new user is created with
    /// `password_reset_required=true`, forcing them to change it on first
    /// login.
    pub password: String,
    /// Optional country allowlist. Defaults server-side to all three
    /// supported countries (`US, HK, CN`) so admin clients that don't
    /// send the field don't accidentally create users with no scope.
    #[serde(default)]
    pub allowed_countries: Option<Vec<String>>,
}

/// `POST /admin/users/{id}/reset-password` body — admin-only.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminResetPasswordIn {
    /// Temporary password the admin will communicate to the user. Sets
    /// `password_reset_required=true` and invalidates all of the user's
    /// existing web sessions; API tokens stay valid (revoke separately if
    /// needed).
    pub password: String,
}

/// `POST /admin/users/{id}/countries` body — admin-only.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminUpdateCountriesIn {
    /// Non-empty list of two-letter country codes drawn from
    /// `{US, HK, CN}`. Replaces (not merges) the user's current
    /// `allowed_countries`. Empty list or unknown code → 400.
    pub allowed_countries: Vec<String>,
}

/// `POST /auth/change-password` body — self-service. While
/// `password_reset_required=true`, `current_password` can be any string
/// (the admin reset already invalidated the prior credential's authority).
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordIn {
    /// Current password. Ignored when `password_reset_required=true`.
    pub current_password: String,
    /// New password.
    pub new_password: String,
    /// Must equal `new_password` byte-for-byte.
    pub new_password_confirm: String,
}
