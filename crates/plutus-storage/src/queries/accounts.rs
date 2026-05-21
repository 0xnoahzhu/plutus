use crate::db::{Db, DbError, Result};
use crate::models::Account;

pub async fn list(db: &Db, user_id: i64) -> Result<Vec<Account>> {
    let rows = db
        .with(async |d| {
            Account::all()
                .order_by((
                    Account::fields().created_at().desc(),
                    Account::fields().id().desc(),
                ))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn get(db: &Db, user_id: i64, id: i64) -> Result<Account> {
    let row = db
        .with(async |d| Account::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewAccount<'a> {
    pub user_id: i64,
    pub broker_id: i64,
    pub name: &'a str,
    pub account_number: Option<&'a str>,
    pub base_currency: &'a str,
}

pub async fn create(db: &Db, input: NewAccount<'_>) -> Result<Account> {
    // Pre-check for a duplicate on the natural key
    // `(user_id, broker_id, account_number)`. Backstop is the
    // UNIQUE NULLS NOT DISTINCT index on the table, but checking here
    // converts the postgres unique-violation 23505 into a clean 409
    // with a user-friendly message instead of a raw "db error" 500.
    let existing: Vec<Account> = db
        .with(async |d| {
            Account::all()
                .filter(Account::fields().broker_id().eq(input.broker_id))
                .exec(d)
                .await
        })
        .await?
        .into_iter()
        .filter(|a| {
            a.user_id == input.user_id
                && a.account_number.as_deref() == input.account_number
        })
        .collect();
    if let Some(dup) = existing.first() {
        return Err(DbError::Conflict(format!(
            "account already exists with same broker_id={} and account_number={:?} (existing id={})",
            input.broker_id, input.account_number, dup.id
        )));
    }

    let now = jiff::Timestamp::now();
    let user_id = input.user_id;
    let broker_id = input.broker_id;
    let name = input.name.to_string();
    let account_number = input.account_number.map(str::to_string);
    let base_currency = input.base_currency.to_string();
    let row = db
        .with(async |d| {
            toasty::create!(Account {
                user_id: user_id,
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

/// Delete a user-owned account. Refuses (`Conflict`) when any transaction
/// still references it — transactions are the per-trade ledger and must
/// not be silently orphaned.
pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let row = get(db, user_id, id).await?;
    let client = db.raw_client().await?;
    let tx_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM transactions WHERE account_id = $1",
            &[&id],
        )
        .await
        .map_err(DbError::from)?
        .get(0);
    if tx_count > 0 {
        return Err(DbError::Conflict(format!(
            "{tx_count} transaction(s) still reference this account"
        )));
    }
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
