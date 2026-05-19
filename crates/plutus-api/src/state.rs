use plutus_storage::Db;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub require_auth: bool,
    pub master_password_hash: String,
    pub cookie_secret: Vec<u8>,
}
