use sha2::{Digest, Sha256};

use crate::db::{Db, DbError, Result};
use crate::models::ApiToken;

pub fn hash_token(plain: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plain.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn list(db: &Db) -> Result<Vec<ApiToken>> {
    db.with(async |d| ApiToken::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn create(db: &Db, label: &str, plain_token: &str) -> Result<ApiToken> {
    let label = label.to_string();
    let token_hash = hash_token(plain_token);
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(ApiToken {
                label: label,
                token_hash: token_hash,
                created_at: now,
                last_used_at: None::<jiff::Timestamp>,
                revoked_at: None::<jiff::Timestamp>,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn revoke(db: &Db, id: i64) -> Result<()> {
    let mut row = db
        .with(async |d| ApiToken::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)?;
    db.with(async |d| {
        row.update()
            .revoked_at(Some(jiff::Timestamp::now()))
            .exec(d)
            .await
    })
    .await?;
    Ok(())
}

pub async fn find_active_by_plain(db: &Db, plain_token: &str) -> Result<Option<ApiToken>> {
    let hash = hash_token(plain_token);
    let row = db
        .with(async |d| ApiToken::filter_by_token_hash(hash).first().exec(d).await)
        .await?;
    Ok(row.filter(|t| t.revoked_at.is_none()))
}
