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

/// Regular per-user tokens. Admin-grade tokens are excluded so a regular
/// user looking at their own list never sees (let alone could delete) an
/// admin token.
pub async fn list_for_user(db: &Db, user_id: i64) -> Result<Vec<ApiToken>> {
    let all = list(db).await?;
    Ok(all
        .into_iter()
        .filter(|t| !t.is_admin && t.user_id == user_id)
        .collect())
}

/// Every admin-grade token (regardless of `user_id`). Used by the admin
/// shell to surface and delete admin keys.
pub async fn list_admin(db: &Db) -> Result<Vec<ApiToken>> {
    let all = list(db).await?;
    Ok(all.into_iter().filter(|t| t.is_admin).collect())
}

pub async fn create(
    db: &Db,
    user_id: i64,
    is_admin: bool,
    label: &str,
    plain_token: &str,
) -> Result<ApiToken> {
    let label = label.to_string();
    let token_hash = hash_token(plain_token);
    // Store the literal plaintext so the UI list can show + copy it later
    // without having to regenerate. See model docs for the trade-off.
    let token_plain = plain_token.to_string();
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(ApiToken {
                user_id: user_id,
                is_admin: is_admin,
                label: label,
                token_hash: token_hash,
                token_plain: Some(token_plain),
                created_at: now,
                last_used_at: None::<jiff::Timestamp>,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

/// Hard delete — the row goes away, the hash no longer resolves, and any
/// bearer request still carrying the plaintext starts getting 401.
pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = db
        .with(async |d| ApiToken::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}

pub async fn find_active_by_plain(db: &Db, plain_token: &str) -> Result<Option<ApiToken>> {
    let hash = hash_token(plain_token);
    let row = db
        .with(async |d| ApiToken::filter_by_token_hash(hash).first().exec(d).await)
        .await?;
    Ok(row)
}
