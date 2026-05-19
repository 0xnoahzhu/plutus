use crate::db::{Db, Result};
use crate::models::Setting;

pub async fn list(db: &Db) -> Result<Vec<Setting>> {
    db.with(async |d| Setting::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, key: &str) -> Result<Option<Setting>> {
    let key = key.to_string();
    db.with(async |d| Setting::filter_by_key(key).first().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn upsert(db: &Db, key: &str, value: &str) -> Result<()> {
    if let Some(mut existing) = get(db, key).await? {
        let value = value.to_string();
        let now = jiff::Timestamp::now();
        db.with(async |d| existing.update().value(value).updated_at(now).exec(d).await)
            .await?;
    } else {
        let key = key.to_string();
        let value = value.to_string();
        let now = jiff::Timestamp::now();
        db.with(async |d| {
            toasty::create!(Setting {
                key: key,
                value: value,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    }
    Ok(())
}
