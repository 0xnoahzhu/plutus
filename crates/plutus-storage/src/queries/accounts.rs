use crate::db::{Db, DbError, Result};
use crate::models::Account;

pub async fn list(db: &Db) -> Result<Vec<Account>> {
    db.with(async |d| Account::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, id: i64) -> Result<Account> {
    let row = db
        .with(async |d| Account::filter_by_id(id).first().exec(d).await)
        .await?;
    row.ok_or(DbError::NotFound)
}

pub struct NewAccount<'a> {
    pub broker_id: i64,
    pub name: &'a str,
    pub account_number: Option<&'a str>,
    pub base_currency: &'a str,
}

pub async fn create(db: &Db, input: NewAccount<'_>) -> Result<Account> {
    let now = jiff::Timestamp::now();
    let broker_id = input.broker_id;
    let name = input.name.to_string();
    let account_number = input.account_number.map(str::to_string);
    let base_currency = input.base_currency.to_string();
    let row = db
        .with(async |d| {
            toasty::create!(Account {
                broker_id: broker_id,
                name: name,
                account_number: account_number,
                base_currency: base_currency,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
