use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::EarningsEvent;

pub struct ListFilter<'a> {
    pub stock_id: Option<i64>,
    pub status: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<EarningsEvent>> {
    let rows = match (filter.stock_id, filter.status) {
        (Some(s), Some(st)) => {
            let st_owned = st.to_string();
            db.with(async |d| {
                EarningsEvent::all()
                    .filter(EarningsEvent::fields().stock_id().eq(s))
                    .filter(EarningsEvent::fields().status().eq(&st_owned))
                    .exec(d)
                    .await
            })
            .await?
        }
        (Some(s), None) => {
            db.with(async |d| {
                EarningsEvent::all()
                    .filter(EarningsEvent::fields().stock_id().eq(s))
                    .exec(d)
                    .await
            })
            .await?
        }
        (None, Some(st)) => {
            let st_owned = st.to_string();
            db.with(async |d| {
                EarningsEvent::all()
                    .filter(EarningsEvent::fields().status().eq(&st_owned))
                    .exec(d)
                    .await
            })
            .await?
        }
        (None, None) => db.with(async |d| EarningsEvent::all().exec(d).await).await?,
    };
    let from = filter.from.map(str::to_string);
    let to = filter.to.map(str::to_string);
    Ok(rows
        .into_iter()
        .filter(|r| from.as_deref().map_or(true, |f| r.announce_date.as_str() >= f))
        .filter(|r| to.as_deref().map_or(true, |t| r.announce_date.as_str() <= t))
        .collect())
}

pub async fn list_for_stock(
    db: &Db,
    stock_id: i64,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<EarningsEvent>> {
    // Toasty's `.limit()` / `.offset()` always emit LIMIT/OFFSET in
    // the generated SQL — no way to conditionally skip them when the
    // caller wants "no cap". Pass `i32::MAX` as the sentinel when
    // `limit` is None (well under i64::MAX so Postgres accepts it
    // without overflow). Offset defaults to 0 (a no-op).
    let l = limit.unwrap_or(i32::MAX as usize);
    let o = offset.unwrap_or(0);
    db.with(async |d| {
        EarningsEvent::all()
            .filter(EarningsEvent::fields().stock_id().eq(stock_id))
            .order_by((
                EarningsEvent::fields().announce_date().desc(),
                EarningsEvent::fields().id().desc(),
            ))
            .limit(l)
            .offset(o)
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

/// Cheap COUNT for X-Total-Count headers on the paginated handler.
pub async fn count_for_stock(db: &Db, stock_id: i64) -> Result<i64> {
    let client = db.raw_client().await?;
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM earnings_events WHERE stock_id = $1",
            &[&stock_id],
        )
        .await
        .map_err(DbError::from)?;
    Ok(row.get::<_, i64>(0))
}

pub async fn get(db: &Db, id: i64) -> Result<EarningsEvent> {
    db.with(async |d| EarningsEvent::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub struct NewEarnings<'a> {
    pub stock_id: i64,
    pub fiscal_year: i32,
    pub fiscal_period: &'a str,
    pub announce_at: Option<jiff::Timestamp>,
    pub announce_date: &'a str,
    pub announce_timing: &'a str,
    pub status: &'a str,
    pub eps_estimate: Option<Decimal>,
    pub eps_actual: Option<Decimal>,
    pub revenue_estimate: Option<Decimal>,
    pub revenue_actual: Option<Decimal>,
    pub currency: Option<&'a str>,
    pub guidance_md: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub url: Option<&'a str>,
    pub source: &'a str,
}

/// All-or-nothing batch upsert via raw SQL so the conflict on the
/// `earnings_events_natural_key` (stock_id, fiscal_year, fiscal_period)
/// refreshes existing rows. One PG transaction; a bad row rolls
/// everything back.
pub async fn batch_upsert(
    db: &Db,
    items: Vec<NewEarnings<'_>>,
) -> Result<Vec<EarningsEvent>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let mut client = db.raw_client().await?;
    let tx = client.transaction().await.map_err(DbError::from)?;
    let now = jiff::Timestamp::now();
    let sql = r#"
        INSERT INTO earnings_events
            (stock_id, fiscal_year, fiscal_period, announce_at, announce_date,
             announce_timing, status, eps_estimate, eps_actual,
             revenue_estimate, revenue_actual, currency, guidance_md,
             notes, url, source, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $17)
        -- Column-list form: `earnings_events_natural_key` is a plain UNIQUE
        -- INDEX, not a UNIQUE CONSTRAINT — ON CONFLICT ON CONSTRAINT only
        -- accepts the latter, so the column form is required.
        ON CONFLICT (stock_id, fiscal_year, fiscal_period) DO UPDATE SET
            announce_at      = EXCLUDED.announce_at,
            announce_date    = EXCLUDED.announce_date,
            announce_timing  = EXCLUDED.announce_timing,
            status           = EXCLUDED.status,
            eps_estimate     = EXCLUDED.eps_estimate,
            eps_actual       = EXCLUDED.eps_actual,
            revenue_estimate = EXCLUDED.revenue_estimate,
            revenue_actual   = EXCLUDED.revenue_actual,
            currency         = EXCLUDED.currency,
            guidance_md      = EXCLUDED.guidance_md,
            notes            = EXCLUDED.notes,
            url              = EXCLUDED.url,
            source           = EXCLUDED.source,
            updated_at       = EXCLUDED.updated_at
        RETURNING id, stock_id, fiscal_year, fiscal_period, announce_at,
                  announce_date, announce_timing, status, eps_estimate,
                  eps_actual, revenue_estimate, revenue_actual, currency,
                  guidance_md, notes, url, source, created_at, updated_at
    "#;
    let mut out = Vec::with_capacity(items.len());
    for item in &items {
        let row = tx
            .query_one(
                sql,
                &[
                    &item.stock_id,
                    &item.fiscal_year,
                    &item.fiscal_period,
                    &item.announce_at,
                    &item.announce_date,
                    &item.announce_timing,
                    &item.status,
                    &item.eps_estimate,
                    &item.eps_actual,
                    &item.revenue_estimate,
                    &item.revenue_actual,
                    &item.currency,
                    &item.guidance_md,
                    &item.notes,
                    &item.url,
                    &item.source,
                    &now,
                ],
            )
            .await
            .map_err(DbError::from)?;
        out.push(EarningsEvent {
            id: row.get("id"),
            stock_id: row.get("stock_id"),
            fiscal_year: row.get("fiscal_year"),
            fiscal_period: row.get("fiscal_period"),
            announce_at: row.get("announce_at"),
            announce_date: row.get("announce_date"),
            announce_timing: row.get("announce_timing"),
            status: row.get("status"),
            eps_estimate: row.get("eps_estimate"),
            eps_actual: row.get("eps_actual"),
            revenue_estimate: row.get("revenue_estimate"),
            revenue_actual: row.get("revenue_actual"),
            currency: row.get("currency"),
            guidance_md: row.get("guidance_md"),
            notes: row.get("notes"),
            url: row.get("url"),
            source: row.get("source"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }
    tx.commit().await.map_err(DbError::from)?;
    Ok(out)
}

/// Upsert by (stock_id, fiscal_year, fiscal_period). Lets the agent post
/// the same event multiple times as new info lands (scheduled → confirmed →
/// released) without duplicating rows.
pub async fn upsert(db: &Db, input: NewEarnings<'_>) -> Result<EarningsEvent> {
    let period_owned = input.fiscal_period.to_string();
    let existing = db
        .with(async |d| {
            EarningsEvent::all()
                .filter(EarningsEvent::fields().stock_id().eq(input.stock_id))
                .filter(EarningsEvent::fields().fiscal_year().eq(input.fiscal_year))
                .filter(EarningsEvent::fields().fiscal_period().eq(&period_owned))
                .first()
                .exec(d)
                .await
        })
        .await?;

    let announce_at = input.announce_at;
    let announce_date = input.announce_date.to_string();
    let announce_timing = input.announce_timing.to_string();
    let status = input.status.to_string();
    let eps_estimate = input.eps_estimate;
    let eps_actual = input.eps_actual;
    let revenue_estimate = input.revenue_estimate;
    let revenue_actual = input.revenue_actual;
    let currency = input.currency.map(str::to_string);
    let guidance_md = input.guidance_md.map(str::to_string);
    let notes = input.notes.map(str::to_string);
    let url = input.url.map(str::to_string);
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| {
            row.update()
                .announce_at(announce_at)
                .announce_date(announce_date)
                .announce_timing(announce_timing)
                .status(status)
                .eps_estimate(eps_estimate)
                .eps_actual(eps_actual)
                .revenue_estimate(revenue_estimate)
                .revenue_actual(revenue_actual)
                .currency(currency)
                .guidance_md(guidance_md)
                .notes(notes)
                .url(url)
                .source(source)
                .updated_at(now)
                .exec(d)
                .await
        })
        .await?;
        let updated = db
            .with(async |d| EarningsEvent::filter_by_id(id).first().exec(d).await)
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(updated)
    } else {
        let stock_id = input.stock_id;
        let fiscal_year = input.fiscal_year;
        let fiscal_period = input.fiscal_period.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(EarningsEvent {
                    stock_id: stock_id,
                    fiscal_year: fiscal_year,
                    fiscal_period: fiscal_period,
                    announce_at: announce_at,
                    announce_date: announce_date,
                    announce_timing: announce_timing,
                    status: status,
                    eps_estimate: eps_estimate,
                    eps_actual: eps_actual,
                    revenue_estimate: revenue_estimate,
                    revenue_actual: revenue_actual,
                    currency: currency,
                    guidance_md: guidance_md,
                    notes: notes,
                    url: url,
                    source: source,
                    created_at: now,
                    updated_at: now,
                })
                .exec(d)
                .await
            })
            .await?;
        Ok(row)
    }
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = get(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
