use rust_decimal::Decimal;

use crate::db::{Db, Result};
use crate::models::InsiderTransaction;

pub async fn list_for_stock(db: &Db, stock_id: i64) -> Result<Vec<InsiderTransaction>> {
    db.with(async |d| {
        InsiderTransaction::all()
            .filter(InsiderTransaction::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub struct NewInsiderTxn<'a> {
    pub stock_id: i64,
    pub person_name: &'a str,
    pub role: Option<&'a str>,
    pub txn_kind: &'a str,
    pub shares: Decimal,
    pub price: Option<Decimal>,
    pub currency: Option<&'a str>,
    pub executed_at: &'a str,
    pub filed_at: jiff::Timestamp,
    pub source: &'a str,
    pub source_url: Option<&'a str>,
}

pub async fn insert(db: &Db, input: NewInsiderTxn<'_>) -> Result<InsiderTransaction> {
    let stock_id = input.stock_id;
    let person_name = input.person_name.to_string();
    let role = input.role.map(str::to_string);
    let txn_kind = input.txn_kind.to_string();
    let shares = input.shares;
    let price = input.price;
    let currency = input.currency.map(str::to_string);
    let executed_at = input.executed_at.to_string();
    let filed_at = input.filed_at;
    let source = input.source.to_string();
    let source_url = input.source_url.map(str::to_string);
    let now = jiff::Timestamp::now();

    let row = db
        .with(async |d| {
            toasty::create!(InsiderTransaction {
                stock_id: stock_id,
                person_name: person_name,
                role: role,
                txn_kind: txn_kind,
                shares: shares,
                price: price,
                currency: currency,
                executed_at: executed_at,
                filed_at: filed_at,
                source: source,
                source_url: source_url,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
