//! Watchlist queries. The watchlist is a single flat list of stocks per user.
//! Unique on `(user_id, stock_id)` so the same ticker can't show up twice for
//! the same user, but different users can independently watch it.

use crate::db::{Db, DbError, Result};
use crate::models::WatchlistItem;

pub async fn list_items(db: &Db, user_id: i64) -> Result<Vec<WatchlistItem>> {
    let rows = db
        .with(async |d| {
            WatchlistItem::all()
                .order_by((
                    WatchlistItem::fields().added_at().desc(),
                    WatchlistItem::fields().id().desc(),
                ))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn get_item(db: &Db, user_id: i64, id: i64) -> Result<WatchlistItem> {
    let row = db
        .with(async |d| WatchlistItem::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

/// Idempotent add: if `(user_id, stock_id)` is already on the watchlist the
/// existing row is returned with notes updated.
pub async fn add_item(
    db: &Db,
    user_id: i64,
    stock_id: i64,
    notes: Option<&str>,
) -> Result<WatchlistItem> {
    let existing = db
        .with(async |d| {
            WatchlistItem::all()
                .filter(WatchlistItem::fields().stock_id().eq(stock_id))
                .exec(d)
                .await
        })
        .await?
        .into_iter()
        .find(|r| r.user_id == user_id);
    let notes_owned = notes.map(str::to_string);

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| row.update().notes(notes_owned).exec(d).await)
            .await?;
        return get_item(db, user_id, id).await;
    }

    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(WatchlistItem {
                user_id: user_id,
                stock_id: stock_id,
                added_at: now,
                notes: notes_owned,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn remove_item(db: &Db, user_id: i64, stock_id: i64) -> Result<()> {
    let row = db
        .with(async |d| {
            WatchlistItem::all()
                .filter(WatchlistItem::fields().stock_id().eq(stock_id))
                .exec(d)
                .await
        })
        .await?
        .into_iter()
        .find(|r| r.user_id == user_id);
    if let Some(item) = row {
        db.with(async |d| item.delete().exec(d).await).await?;
    }
    Ok(())
}
