use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::InsiderTransaction;

pub async fn list_for_stock(
    db: &Db,
    stock_id: i64,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<InsiderTransaction>> {
    let l = limit.unwrap_or(i32::MAX as usize);
    let o = offset.unwrap_or(0);
    db.with(async |d| {
        InsiderTransaction::all()
            .filter(InsiderTransaction::fields().stock_id().eq(stock_id))
            .order_by((
                InsiderTransaction::fields().executed_at().desc(),
                InsiderTransaction::fields().id().desc(),
            ))
            .limit(l)
            .offset(o)
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn get(db: &Db, id: i64) -> Result<InsiderTransaction> {
    db.with(async |d| InsiderTransaction::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = get(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}

pub async fn count_for_stock(db: &Db, stock_id: i64) -> Result<i64> {
    let client = db.raw_client().await?;
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM insider_transactions WHERE stock_id = $1",
            &[&stock_id],
        )
        .await
        .map_err(DbError::from)?;
    Ok(row.get::<_, i64>(0))
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

/// All-or-nothing batch insert of N insider transactions. Single
/// transaction; a bad row rolls everything back.
pub async fn batch_insert(
    db: &Db,
    items: &[NewInsiderTxn<'_>],
) -> Result<Vec<InsiderTransaction>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let mut client = db.raw_client().await?;
    let tx = client.transaction().await.map_err(DbError::from)?;
    let mut out = Vec::with_capacity(items.len());
    let now = jiff::Timestamp::now();
    let sql = r#"
        INSERT INTO insider_transactions
            (stock_id, person_name, role, txn_kind, shares, price,
             currency, executed_at, filed_at, source, source_url, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, stock_id, person_name, role, txn_kind, shares, price,
                  currency, executed_at, filed_at, source, source_url, created_at
    "#;
    for item in items {
        let person_owned = item.person_name.to_string();
        let role_owned = item.role.map(str::to_string);
        let kind_owned = item.txn_kind.to_string();
        let currency_owned = item.currency.map(str::to_string);
        let executed_owned = item.executed_at.to_string();
        let source_owned = item.source.to_string();
        let url_owned = item.source_url.map(str::to_string);
        let row = tx
            .query_one(
                sql,
                &[
                    &item.stock_id,
                    &person_owned,
                    &role_owned,
                    &kind_owned,
                    &item.shares,
                    &item.price,
                    &currency_owned,
                    &executed_owned,
                    &item.filed_at,
                    &source_owned,
                    &url_owned,
                    &now,
                ],
            )
            .await
            .map_err(DbError::from)?;
        out.push(InsiderTransaction {
            id: row.get("id"),
            stock_id: row.get("stock_id"),
            person_name: row.get("person_name"),
            role: row.get("role"),
            txn_kind: row.get("txn_kind"),
            shares: row.get("shares"),
            price: row.get("price"),
            currency: row.get("currency"),
            executed_at: row.get("executed_at"),
            filed_at: row.get("filed_at"),
            source: row.get("source"),
            source_url: row.get("source_url"),
            created_at: row.get("created_at"),
        });
    }
    tx.commit().await.map_err(DbError::from)?;
    Ok(out)
}
