//! Stock CRUD helpers.

use crate::db::{Db, DbError, Result};
use crate::models::Stock;

pub async fn list(db: &Db) -> Result<Vec<Stock>> {
    db.with(async |d| Stock::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, id: i64) -> Result<Stock> {
    let row = db
        .with(async |d| Stock::filter_by_id(id).first().exec(d).await)
        .await?;
    row.ok_or(DbError::NotFound)
}

pub async fn find_by_market_symbol(
    db: &Db,
    market_code: &str,
    symbol: &str,
) -> Result<Option<Stock>> {
    let market_code = market_code.to_string();
    let symbol = symbol.to_string();
    db.with(async |d| {
        Stock::all()
            .filter(Stock::fields().market_code().eq(&market_code))
            .filter(Stock::fields().symbol().eq(&symbol))
            .first()
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub struct NewStock<'a> {
    pub market_code: &'a str,
    pub symbol: &'a str,
    pub isin: Option<&'a str>,
    pub figi: Option<&'a str>,
    pub currency: &'a str,
    pub lot_size: Option<i32>,
    pub asset_class: &'a str,
    pub sector_code: Option<&'a str>,
}

pub async fn create(db: &Db, input: NewStock<'_>) -> Result<Stock> {
    if find_by_market_symbol(db, input.market_code, input.symbol)
        .await?
        .is_some()
    {
        return Err(DbError::Conflict(format!(
            "stock {}:{} already exists",
            input.market_code, input.symbol
        )));
    }
    let now = jiff::Timestamp::now();
    let market_code = input.market_code.to_string();
    let symbol = input.symbol.to_string();
    let isin = input.isin.map(str::to_string);
    let figi = input.figi.map(str::to_string);
    let currency = input.currency.to_string();
    let lot_size = input.lot_size;
    let asset_class = input.asset_class.to_string();
    let sector_code = input.sector_code.map(str::to_string);

    let row = db
        .with(async |d| {
            toasty::create!(Stock {
                market_code: market_code,
                symbol: symbol,
                isin: isin,
                figi: figi,
                currency: currency,
                lot_size: lot_size,
                asset_class: asset_class,
                sector_code: sector_code,
                created_at: now,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = get(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
