use plutus_storage::Db;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub require_auth: bool,
    /// Master password in plaintext. Empty string means login is disabled
    /// (the API will refuse to mint a session cookie). Read from
    /// `PLUTUS_MASTER_PASSWORD` at server boot.
    pub master_password: String,
}
