use crate::db::{Db, DbError, Result};
use crate::models::{Watchlist, WatchlistItem};

pub async fn list(db: &Db) -> Result<Vec<Watchlist>> {
    db.with(async |d| Watchlist::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, id: i64) -> Result<Watchlist> {
    let row = db
        .with(async |d| Watchlist::filter_by_id(id).first().exec(d).await)
        .await?;
    row.ok_or(DbError::NotFound)
}

pub async fn create(
    db: &Db,
    name: &str,
    description: Option<&str>,
    sort_order: i32,
) -> Result<Watchlist> {
    let now = jiff::Timestamp::now();
    let name = name.to_string();
    let description = description.map(str::to_string);
    let row = db
        .with(async |d| {
            toasty::create!(Watchlist {
                name: name,
                description: description,
                sort_order: sort_order,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = get(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}

/// Partial update. Any `Some(...)` field is written; `None` is left untouched.
pub async fn update(
    db: &Db,
    id: i64,
    name: Option<&str>,
    description: Option<Option<&str>>,
    sort_order: Option<i32>,
) -> Result<Watchlist> {
    let mut row = db
        .with(async |d| Watchlist::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)?;

    // toasty's UpdateBuilder consumes self per setter, so we materialize the
    // selected fields into owned values before entering the lock guard.
    let name_owned = name.map(str::to_string);
    let description_owned: Option<Option<String>> =
        description.map(|inner| inner.map(str::to_string));

    db.with(async |d| {
        let mut upd = row.update();
        if let Some(n) = name_owned {
            upd = upd.name(n);
        }
        if let Some(desc) = description_owned {
            upd = upd.description(desc);
        }
        if let Some(o) = sort_order {
            upd = upd.sort_order(o);
        }
        upd.exec(d).await
    })
    .await?;

    // Re-fetch so we return the latest state (toasty's update doesn't return rows).
    let updated = db
        .with(async |d| Watchlist::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)?;
    Ok(updated)
}

pub async fn list_items(db: &Db, watchlist_id: i64) -> Result<Vec<WatchlistItem>> {
    db.with(async |d| {
        WatchlistItem::all()
            .filter(WatchlistItem::fields().watchlist_id().eq(watchlist_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn add_item(
    db: &Db,
    watchlist_id: i64,
    stock_id: i64,
    notes: Option<&str>,
) -> Result<WatchlistItem> {
    let existing = db
        .with(async |d| {
            WatchlistItem::all()
                .filter(WatchlistItem::fields().watchlist_id().eq(watchlist_id))
                .filter(WatchlistItem::fields().stock_id().eq(stock_id))
                .first()
                .exec(d)
                .await
        })
        .await?;
    if existing.is_some() {
        return Err(DbError::Conflict("stock already in watchlist".into()));
    }
    let now = jiff::Timestamp::now();
    let notes = notes.map(str::to_string);
    let row = db
        .with(async |d| {
            toasty::create!(WatchlistItem {
                watchlist_id: watchlist_id,
                stock_id: stock_id,
                added_at: now,
                notes: notes,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn remove_item(db: &Db, watchlist_id: i64, stock_id: i64) -> Result<()> {
    let row = db
        .with(async |d| {
            WatchlistItem::all()
                .filter(WatchlistItem::fields().watchlist_id().eq(watchlist_id))
                .filter(WatchlistItem::fields().stock_id().eq(stock_id))
                .first()
                .exec(d)
                .await
        })
        .await?
        .ok_or(DbError::NotFound)?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
