use rust_decimal::Decimal;

use crate::db::{Db, Result};
use crate::models::FxRateDaily;

pub async fn list(db: &Db) -> Result<Vec<FxRateDaily>> {
    db.with(async |d| FxRateDaily::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn insert(
    db: &Db,
    base: &str,
    quote: &str,
    date: &str,
    rate: Decimal,
    source: &str,
) -> Result<FxRateDaily> {
    let base_currency = base.to_string();
    let quote_currency = quote.to_string();
    let rate_date = date.to_string();
    let source = source.to_string();
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(FxRateDaily {
                base_currency: base_currency,
                quote_currency: quote_currency,
                rate_date: rate_date,
                rate: rate,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
