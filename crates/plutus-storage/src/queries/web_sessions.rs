//! Web session rows. The cookie carries the random `id`; this row resolves
//! the id to the identity (user_id + is_admin + username) plus an expiry.

use crate::db::{Db, Result};
use crate::models::WebSession;

/// Default session lifetime. Long enough that users rarely need to re-login,
/// short enough that a forgotten browser tab eventually times out.
pub const DEFAULT_TTL_DAYS: i64 = 30;

/// Insert a new session row. `id` is the cookie value (callers generate a
/// fresh random token before calling).
pub async fn create(
    db: &Db,
    id: &str,
    user_id: i64,
    is_admin: bool,
    username: &str,
) -> Result<WebSession> {
    let id = id.to_string();
    let username = username.to_string();
    let now = jiff::Timestamp::now();
    let expires_at = now
        .checked_add(jiff::SignedDuration::from_hours(24 * DEFAULT_TTL_DAYS))
        .unwrap_or(now);
    let row = db
        .with(async |d| {
            toasty::create!(WebSession {
                id: id,
                user_id: user_id,
                is_admin: is_admin,
                username: username,
                created_at: now,
                expires_at: expires_at,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

/// Look up a session by cookie id. Returns `None` for missing or expired rows.
pub async fn find_active(db: &Db, id: &str) -> Result<Option<WebSession>> {
    let id = id.to_string();
    let row = db
        .with(async |d| WebSession::filter_by_id(id).first().exec(d).await)
        .await?;
    let Some(row) = row else { return Ok(None) };
    if row.expires_at <= jiff::Timestamp::now() {
        return Ok(None);
    }
    Ok(Some(row))
}

/// Best-effort delete. Missing rows are not an error.
pub async fn delete(db: &Db, id: &str) -> Result<()> {
    let id = id.to_string();
    let row = db
        .with(async |d| WebSession::filter_by_id(id).first().exec(d).await)
        .await?;
    if let Some(row) = row {
        db.with(async |d| row.delete().exec(d).await).await?;
    }
    Ok(())
}

