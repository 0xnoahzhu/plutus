use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::Transaction;

/// All transactions sorted newest first. `id desc` is the deterministic
/// tie-breaker so transactions executed at the same instant (rare but
/// possible in bulk imports) keep a stable relative order across
/// refreshes — important once pagination lands.
pub async fn list(db: &Db, user_id: i64) -> Result<Vec<Transaction>> {
    let rows = db
        .with(async |d| {
            Transaction::all()
                .order_by((
                    Transaction::fields().executed_at().desc(),
                    Transaction::fields().id().desc(),
                ))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn list_for_account(db: &Db, user_id: i64, account_id: i64) -> Result<Vec<Transaction>> {
    let rows = db
        .with(async |d| {
            Transaction::all()
                .filter(Transaction::fields().account_id().eq(account_id))
                .order_by((
                    Transaction::fields().executed_at().desc(),
                    Transaction::fields().id().desc(),
                ))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn list_for_stock(db: &Db, user_id: i64, stock_id: i64) -> Result<Vec<Transaction>> {
    let rows = db
        .with(async |d| {
            Transaction::all()
                .filter(Transaction::fields().stock_id().eq(Some(stock_id)))
                .order_by((
                    Transaction::fields().executed_at().desc(),
                    Transaction::fields().id().desc(),
                ))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn get(db: &Db, user_id: i64, id: i64) -> Result<Transaction> {
    let row = db
        .with(async |d| Transaction::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

#[derive(Debug, Clone)]
pub struct NewTransaction<'a> {
    pub user_id: i64,
    pub account_id: i64,
    pub stock_id: Option<i64>,
    pub kind: &'a str,
    pub executed_at: jiff::Timestamp,
    pub quantity: Decimal,
    pub price: Decimal,
    pub trade_currency: &'a str,
    pub commission: Decimal,
    pub commission_currency: &'a str,
    pub tax: Decimal,
    pub tax_currency: &'a str,
    pub fx_rate_to_base: Decimal,
    pub external_ref: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub source: &'a str,
    pub source_metadata: Option<&'a str>,
}

pub async fn create(db: &Db, input: NewTransaction<'_>) -> Result<Transaction> {
    let now = jiff::Timestamp::now();
    let user_id = input.user_id;
    let account_id = input.account_id;
    let stock_id = input.stock_id;
    let kind = input.kind.to_string();
    let executed_at = input.executed_at;
    let quantity = input.quantity;
    let price = input.price;
    let trade_currency = input.trade_currency.to_string();
    let commission = input.commission;
    let commission_currency = input.commission_currency.to_string();
    let tax = input.tax;
    let tax_currency = input.tax_currency.to_string();
    let fx_rate_to_base = input.fx_rate_to_base;
    let external_ref = input.external_ref.map(str::to_string);
    let notes = input.notes.map(str::to_string);
    let source = input.source.to_string();
    let source_metadata = input.source_metadata.map(str::to_string);

    let row = db
        .with(async |d| {
            toasty::create!(Transaction {
                user_id: user_id,
                account_id: account_id,
                stock_id: stock_id,
                kind: kind,
                executed_at: executed_at,
                quantity: quantity,
                price: price,
                trade_currency: trade_currency,
                commission: commission,
                commission_currency: commission_currency,
                tax: tax,
                tax_currency: tax_currency,
                fx_rate_to_base: fx_rate_to_base,
                external_ref: external_ref,
                notes: notes,
                source: source,
                source_metadata: source_metadata,
                created_at: now,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let row = get(db, user_id, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
