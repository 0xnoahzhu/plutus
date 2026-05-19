use rust_decimal::Decimal;

use crate::db::{Db, Result};
use crate::models::OhlcvDaily;

pub async fn list_for_stock(db: &Db, stock_id: i64) -> Result<Vec<OhlcvDaily>> {
    db.with(async |d| {
        OhlcvDaily::all()
            .filter(OhlcvDaily::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub struct NewOhlcv<'a> {
    pub stock_id: i64,
    pub trade_date: &'a str,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub adjusted_close: Option<Decimal>,
    pub volume: i64,
    pub source: &'a str,
}

pub async fn insert(db: &Db, input: NewOhlcv<'_>) -> Result<OhlcvDaily> {
    let stock_id = input.stock_id;
    let trade_date = input.trade_date.to_string();
    let open = input.open;
    let high = input.high;
    let low = input.low;
    let close = input.close;
    let adjusted_close = input.adjusted_close;
    let volume = input.volume;
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(OhlcvDaily {
                stock_id: stock_id,
                trade_date: trade_date,
                open: open,
                high: high,
                low: low,
                close: close,
                adjusted_close: adjusted_close,
                volume: volume,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
