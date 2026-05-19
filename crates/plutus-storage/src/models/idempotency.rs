//! Idempotency-Key replay table. Renamed `path` → `request_path` to dodge a
//! name collision with the methods toasty generates on every model.

#[derive(Debug, toasty::Model)]
#[table = "idempotency_keys"]
pub struct IdempotencyKey {
    #[key]
    pub key: String,
    pub method: String,
    pub request_path: String,
    pub request_hash: String,
    pub response_status: i32,
    pub response_body: String,
    pub created_at: jiff::Timestamp,
    pub expires_at: jiff::Timestamp,
}
