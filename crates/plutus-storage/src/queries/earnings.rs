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

pub async fn list_for_stock(db: &Db, stock_id: i64) -> Result<Vec<EarningsEvent>> {
    db.with(async |d| {
        EarningsEvent::all()
            .filter(EarningsEvent::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
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
