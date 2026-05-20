use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::User;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserOut {
    pub id: i64,
    pub username: String,
    pub password_reset_required: bool,
    /// Two-letter country codes the user is scoped to (e.g. `["US","HK"]`).
    /// Empty list = admin actor (no scope, sees everything); non-empty for
    /// every DB-backed user.
    pub allowed_countries: Vec<String>,
    pub created_at: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminCreateUserIn {
    pub username: String,
    pub password: String,
    /// Optional country allowlist. Defaults server-side to all three
    /// supported countries (US, HK, CN) so older admin clients that
    /// don't send the field keep the previous behavior.
    #[serde(default)]
    pub allowed_countries: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminResetPasswordIn {
    /// Temporary password the admin will communicate to the user. On the next
    /// successful login the user is forced through `/auth/change-password`.
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminUpdateCountriesIn {
    /// Non-empty list of two-letter country codes drawn from
    /// `{US, HK, CN}`. The server validates the membership; passing an
    /// empty list or any unknown code yields a 400.
    pub allowed_countries: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordIn {
    pub current_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}
