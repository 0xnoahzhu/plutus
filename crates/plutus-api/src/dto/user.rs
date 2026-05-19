use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::User;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserOut {
    pub id: i64,
    pub username: String,
    pub password_reset_required: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserOut {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            password_reset_required: u.password_reset_required,
            created_at: u.created_at.to_string(),
            updated_at: u.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminCreateUserIn {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminResetPasswordIn {
    /// Temporary password the admin will communicate to the user. On the next
    /// successful login the user is forced through `/auth/change-password`.
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordIn {
    pub current_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}
