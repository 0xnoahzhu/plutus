//! End-user account CRUD. The password column holds an Argon2 hash —
//! callers (the API layer) are responsible for hashing on the way in
//! and verifying on the way out.

use crate::db::{Db, DbError, Result};
use crate::models::User;

pub async fn list(db: &Db) -> Result<Vec<User>> {
    db.with(async |d| User::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, id: i64) -> Result<User> {
    db.with(async |d| User::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub async fn find_by_username(db: &Db, username: &str) -> Result<Option<User>> {
    let owned = username.to_string();
    db.with(async |d| User::filter_by_username(owned).first().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn create(
    db: &Db,
    username: &str,
    password_hash: &str,
    allowed_countries: &[String],
) -> Result<User> {
    let username = username.to_string();
    let password_hash = password_hash.to_string();
    let allowed = allowed_countries.join(",");
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(User {
                username: username,
                password_hash: password_hash,
                password_reset_required: false,
                allowed_countries: allowed,
                created_at: now,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

/// Replace a user's country allowlist with `codes`. Caller (API DTO
/// layer) has already validated that each entry is in `{US, HK, CN}`
/// and that the list is non-empty.
pub async fn set_countries(db: &Db, id: i64, codes: &[String]) -> Result<User> {
    let csv = codes.join(",");
    let mut row = get(db, id).await?;
    let now = jiff::Timestamp::now();
    db.with(async |d| {
        row.update()
            .allowed_countries(csv)
            .updated_at(now)
            .exec(d)
            .await
    })
    .await?;
    get(db, id).await
}

/// Apply a new hash directly. Used by the user-driven /change-password flow
/// once the actor has confirmed their current password and chosen a new one.
/// Clears the `password_reset_required` flag.
pub async fn set_password(db: &Db, id: i64, password_hash: &str) -> Result<User> {
    let password_hash = password_hash.to_string();
    let now = jiff::Timestamp::now();
    let mut row = get(db, id).await?;
    db.with(async |d| {
        row.update()
            .password_hash(password_hash)
            .password_reset_required(false)
            .updated_at(now)
            .exec(d)
            .await
    })
    .await?;
    get(db, id).await
}

/// Admin-initiated reset: writes a fresh temporary hash and flips the
/// `password_reset_required` flag so the next login is forced through
/// `/change-password`.
pub async fn admin_reset(db: &Db, id: i64, temp_hash: &str) -> Result<User> {
    let temp_hash = temp_hash.to_string();
    let now = jiff::Timestamp::now();
    let mut row = get(db, id).await?;
    db.with(async |d| {
        row.update()
            .password_hash(temp_hash)
            .password_reset_required(true)
            .updated_at(now)
            .exec(d)
            .await
    })
    .await?;
    get(db, id).await
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = get(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
