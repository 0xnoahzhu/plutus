use crate::db::{Db, Result};
use crate::models::IdempotencyKey;

pub struct StoredResponse {
    pub status: i32,
    pub body: String,
}

pub async fn lookup(db: &Db, key: &str, request_hash: &str) -> Result<Option<StoredResponse>> {
    let key = key.to_string();
    let row = db
        .with(async |d| IdempotencyKey::filter_by_key(key).first().exec(d).await)
        .await?;
    Ok(row.filter(|r| r.request_hash == request_hash).map(|r| StoredResponse {
        status: r.response_status,
        body: r.response_body,
    }))
}

pub async fn store(
    db: &Db,
    key: &str,
    method: &str,
    request_path: &str,
    request_hash: &str,
    status: i32,
    body: &str,
    ttl_hours: i64,
) -> Result<()> {
    let now = jiff::Timestamp::now();
    let expires_at = now + jiff::Span::new().hours(ttl_hours);
    let key = key.to_string();
    let method = method.to_string();
    let request_path = request_path.to_string();
    let request_hash = request_hash.to_string();
    let response_body = body.to_string();
    db.with(async |d| {
        toasty::create!(IdempotencyKey {
            key: key,
            method: method,
            request_path: request_path,
            request_hash: request_hash,
            response_status: status,
            response_body: response_body,
            created_at: now,
            expires_at: expires_at,
        })
        .exec(d)
        .await
    })
    .await?;
    Ok(())
}
