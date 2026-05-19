use plutus_storage::Db;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub require_auth: bool,
    /// Admin username, in plaintext, from `PLUTUS_ADMIN_USERNAME`. Empty means
    /// no admin account is configured — admin login is disabled.
    pub admin_username: String,
    /// Admin password, in plaintext, from `PLUTUS_ADMIN_PASSWORD`. The admin
    /// account intentionally does NOT live in the database; admin credentials
    /// authenticate against this env-var value directly.
    pub admin_password: String,
}

impl AppState {
    /// True when the request's submitted username matches the configured admin
    /// (and admin is configured at all).
    #[must_use]
    pub fn is_admin_username(&self, username: &str) -> bool {
        !self.admin_username.is_empty() && username == self.admin_username
    }
}
