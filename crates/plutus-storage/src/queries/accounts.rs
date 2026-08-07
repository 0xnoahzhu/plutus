use rust_decimal::Decimal;

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
                // No anchor by default: a brand-new account starts empty,
                // so the ledger alone tells the truth about its cash.
                cash_balance: Decimal::ZERO,
                cash_as_of: None::<jiff::Timestamp>,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

/// Fields an account owner may change after creation. `broker_id`,
/// `base_currency` and `account_number` are deliberately absent —
/// changing them would silently re-denominate or re-key an account that
/// transactions already point at.
#[derive(Debug, Clone, Default)]
pub struct AccountPatch<'a> {
    pub name: Option<&'a str>,
    /// Known cash at `cash_as_of`, in the account's base currency.
    pub cash_balance: Option<Decimal>,
    /// Outer `None` leaves the anchor time alone; `Some(None)` clears it
    /// so the whole ledger counts again.
    pub cash_as_of: Option<Option<jiff::Timestamp>>,
}

pub async fn update(
    db: &Db,
    user_id: i64,
    id: i64,
    patch: AccountPatch<'_>,
) -> Result<Account> {
    let mut row = get(db, user_id, id).await?;
    db.with(async |d| {
        let mut q = row.update();
        if let Some(name) = patch.name {
            q = q.name(name.to_string());
        }
        if let Some(cash_balance) = patch.cash_balance {
            q = q.cash_balance(cash_balance);
        }
        if let Some(cash_as_of) = patch.cash_as_of {
            q = q.cash_as_of(cash_as_of);
        }
        q.exec(d).await
    })
    .await?;
    get(db, user_id, id).await
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
