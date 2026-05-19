use crate::db::{Db, Result};
use crate::models::Broker;

pub async fn list(db: &Db) -> Result<Vec<Broker>> {
    db.with(async |d| Broker::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get_by_code(db: &Db, code: &str) -> Result<Option<Broker>> {
    let code = code.to_string();
    db.with(async |d| Broker::filter_by_code(code).first().exec(d).await)
        .await
        .map_err(Into::into)
}
