//! Watchlist queries. The watchlist is a single flat list of stocks; the
//! group concept has been retired.

use crate::db::{Db, DbError, Result};
use crate::models::WatchlistItem;

pub async fn list_items(db: &Db) -> Result<Vec<WatchlistItem>> {
    db.with(async |d| WatchlistItem::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get_item(db: &Db, id: i64) -> Result<WatchlistItem> {
    db.with(async |d| WatchlistItem::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

/// Idempotent add: if `stock_id` is already on the watchlist the existing
/// row is returned with notes updated.
pub async fn add_item(
    db: &Db,
    stock_id: i64,
    notes: Option<&str>,
) -> Result<WatchlistItem> {
    let existing = db
        .with(async |d| {
            WatchlistItem::all()
                .filter(WatchlistItem::fields().stock_id().eq(stock_id))
                .first()
                .exec(d)
                .await
        })
        .await?;
    let notes_owned = notes.map(str::to_string);

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| row.update().notes(notes_owned).exec(d).await)
            .await?;
        return get_item(db, id).await;
    }

    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(WatchlistItem {
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

pub async fn remove_item(db: &Db, stock_id: i64) -> Result<()> {
    let row = db
        .with(async |d| {
            WatchlistItem::all()
                .filter(WatchlistItem::fields().stock_id().eq(stock_id))
                .first()
                .exec(d)
                .await
        })
        .await?;
    if let Some(item) = row {
        db.with(async |d| item.delete().exec(d).await).await?;
    }
    Ok(())
}
