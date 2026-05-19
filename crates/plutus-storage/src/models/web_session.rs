//! Server-side session records for browser logins. Cookie carries the id; this
//! table holds expiry and any per-session data.

#[derive(Debug, toasty::Model)]
#[table = "web_sessions"]
pub struct WebSession {
    #[key]
    pub id: String, // 32-byte random hex
    pub created_at: jiff::Timestamp,
    pub expires_at: jiff::Timestamp,
}
