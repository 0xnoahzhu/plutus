use crate::db::{Db, DbError, Result};
use crate::models::StockTranslation;

pub async fn list_for_stock(db: &Db, stock_id: i64) -> Result<Vec<StockTranslation>> {
    db.with(async |d| {
        StockTranslation::all()
            .filter(StockTranslation::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn upsert(
    db: &Db,
    stock_id: i64,
    locale: &str,
    name: &str,
    description_md: Option<&str>,
) -> Result<StockTranslation> {
    let now = jiff::Timestamp::now();
    let locale_owned = locale.to_string();
    let existing = db
        .with(async |d| {
            StockTranslation::all()
                .filter(StockTranslation::fields().stock_id().eq(stock_id))
                .filter(StockTranslation::fields().locale().eq(&locale_owned))
                .first()
                .exec(d)
                .await
        })
        .await?;
    let name = name.to_string();
    let description_md = description_md.map(str::to_string);
    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| {
            row.update()
                .name(name)
                .description_md(description_md)
                .updated_at(now)
                .exec(d)
                .await
        })
        .await?;
        let updated = db
            .with(async |d| StockTranslation::filter_by_id(id).first().exec(d).await)
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(updated)
    } else {
        let locale = locale.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(StockTranslation {
                    stock_id: stock_id,
                    locale: locale,
                    name: name,
                    description_md: description_md,
                    updated_at: now,
                })
                .exec(d)
                .await
            })
            .await?;
        Ok(row)
    }
}
