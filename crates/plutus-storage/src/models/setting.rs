//! Key/value singleton settings. Replaces a `users` table since plutus is
//! single-user; user-level prefs live as rows here.

#[derive(Debug, toasty::Model)]
#[table = "settings"]
pub struct Setting {
    #[key]
    pub key: String,
    pub value: String, // JSON-encoded; agent and web both parse client-side
    pub updated_at: jiff::Timestamp,
}
