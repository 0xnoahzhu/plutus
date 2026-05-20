use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
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
    // Single-row upsert via raw SQL so the natural-key conflict
    // (stock_id, trade_date) refreshes the existing bar instead of
    // erroring. Mirrors `batch_upsert` below for consistency.
    let rows = batch_upsert(db, &[input]).await?;
    rows.into_iter().next().ok_or(DbError::NotFound)
}

/// All-or-nothing upsert of N bars. Wrapped in a single Postgres
/// transaction so a validation error on any row rolls the whole batch
/// back. Per-row conflict on the `ohlcv_daily_natural_key`
/// (stock_id, trade_date) refreshes the existing bar — agents can
/// re-run their backfill loaders without producing duplicates.
pub async fn batch_upsert(db: &Db, items: &[NewOhlcv<'_>]) -> Result<Vec<OhlcvDaily>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let mut client = db.raw_client().await?;
    let tx = client.transaction().await.map_err(DbError::from)?;
    let mut out = Vec::with_capacity(items.len());
    let now = jiff::Timestamp::now();
    let sql = r#"
        INSERT INTO ohlcv_daily
            (stock_id, trade_date, open, high, low, close,
             adjusted_close, volume, source, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        -- Column-list form: `ohlcv_daily_natural_key` is a UNIQUE INDEX,
        -- not a UNIQUE CONSTRAINT — ON CONFLICT ON CONSTRAINT requires the
        -- latter, so we target the columns directly.
        ON CONFLICT (stock_id, trade_date) DO UPDATE SET
            open           = EXCLUDED.open,
            high           = EXCLUDED.high,
            low            = EXCLUDED.low,
            close          = EXCLUDED.close,
            adjusted_close = EXCLUDED.adjusted_close,
            volume         = EXCLUDED.volume,
            source         = EXCLUDED.source
        RETURNING id, stock_id, trade_date, open, high, low, close,
                  adjusted_close, volume, source, created_at
    "#;
    for item in items {
        let trade_date_owned = item.trade_date.to_string();
        let source_owned = item.source.to_string();
        let row = tx
            .query_one(
                sql,
                &[
                    &item.stock_id,
                    &trade_date_owned,
                    &item.open,
                    &item.high,
                    &item.low,
                    &item.close,
                    &item.adjusted_close,
                    &item.volume,
                    &source_owned,
                    &now,
                ],
            )
            .await
            .map_err(DbError::from)?;
        out.push(OhlcvDaily {
            id: row.get("id"),
            stock_id: row.get("stock_id"),
            trade_date: row.get("trade_date"),
            open: row.get("open"),
            high: row.get("high"),
            low: row.get("low"),
            close: row.get("close"),
            adjusted_close: row.get("adjusted_close"),
            volume: row.get("volume"),
            source: row.get("source"),
            created_at: row.get("created_at"),
        });
    }
    tx.commit().await.map_err(DbError::from)?;
    Ok(out)
}
