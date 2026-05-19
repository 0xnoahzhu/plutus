use crate::db::{Db, DbError, Result};
use crate::models::Filing;

pub async fn list_for_stock(db: &Db, stock_id: i64) -> Result<Vec<Filing>> {
    db.with(async |d| {
        Filing::all()
            .filter(Filing::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn get(db: &Db, id: i64) -> Result<Filing> {
    db.with(async |d| Filing::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub struct NewFiling<'a> {
    pub stock_id: i64,
    pub filing_type: &'a str,
    pub fiscal_year: Option<i32>,
    pub fiscal_period: Option<&'a str>,
    pub period_end: Option<&'a str>,
    pub filed_at: jiff::Timestamp,
    pub url: &'a str,
    pub title: &'a str,
    pub content_md: Option<&'a str>,
    pub source: &'a str,
}

pub async fn create(db: &Db, input: NewFiling<'_>) -> Result<Filing> {
    let stock_id = input.stock_id;
    let filing_type = input.filing_type.to_string();
    let fiscal_year = input.fiscal_year;
    let fiscal_period = input.fiscal_period.map(str::to_string);
    let period_end = input.period_end.map(str::to_string);
    let filed_at = input.filed_at;
    let url = input.url.to_string();
    let title = input.title.to_string();
    let content_md = input.content_md.map(str::to_string);
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(Filing {
                stock_id: stock_id,
                filing_type: filing_type,
                fiscal_year: fiscal_year,
                fiscal_period: fiscal_period,
                period_end: period_end,
                filed_at: filed_at,
                url: url,
                title: title,
                content_md: content_md,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
