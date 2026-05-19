use crate::db::{Db, DbError, Result};
use crate::models::Broker;

pub async fn list(db: &Db) -> Result<Vec<Broker>> {
    db.with(async |d| Broker::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, id: i64) -> Result<Broker> {
    db.with(async |d| Broker::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub async fn get_by_code(db: &Db, code: &str) -> Result<Option<Broker>> {
    let code = code.to_string();
    db.with(async |d| Broker::filter_by_code(code).first().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn create(db: &Db, code: &str, name: &str) -> Result<Broker> {
    let code_owned = code.to_string();
    let name_owned = name.to_string();
    let row = db
        .with(async |d| {
            toasty::create!(Broker {
                code: code_owned,
                name: name_owned,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn update_name(db: &Db, id: i64, name: &str) -> Result<Broker> {
    let mut row = get(db, id).await?;
    let name_owned = name.to_string();
    db.with(async |d| row.update().name(name_owned).exec(d).await).await?;
    get(db, id).await
}

/// Delete a broker. Refuses with `Conflict` if any account still
/// references it — transactions and other downstream rows hang off
/// accounts, so the account FK is the canonical dependency.
pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = get(db, id).await?;
    let client = db.raw_client().await?;
    let account_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM accounts WHERE broker_id = $1",
            &[&id],
        )
        .await
        .map_err(DbError::from)?
        .get(0);
    if account_count > 0 {
        return Err(DbError::Conflict(format!(
            "{account_count} account(s) still reference this broker"
        )));
    }
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
