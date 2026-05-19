use crate::db::{Db, DbError, Result};
use crate::models::Catalyst;

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<&'a str>,
    pub catalyst_kind: Option<&'a str>,
    pub status: Option<&'a str>,
    pub impact_level: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<Catalyst>> {
    let mut rows: Vec<Catalyst> =
        db.with(async |d| Catalyst::all().exec(d).await).await?;
    rows.retain(|c| c.user_id == filter.user_id);
    if let Some(s) = filter.stock_id {
        rows.retain(|c| c.stock_id == Some(s));
    }
    if let Some(sc) = filter.sector_code {
        rows.retain(|c| c.sector_code.as_deref() == Some(sc));
    }
    if let Some(k) = filter.catalyst_kind {
        rows.retain(|c| c.catalyst_kind == k);
    }
    if let Some(st) = filter.status {
        rows.retain(|c| c.status == st);
    }
    if let Some(il) = filter.impact_level {
        rows.retain(|c| c.impact_level == il);
    }
    if let Some(f) = filter.from {
        rows.retain(|c| c.catalyst_date.as_str() >= f);
    }
    if let Some(t) = filter.to {
        rows.retain(|c| c.catalyst_date.as_str() <= t);
    }
    Ok(rows)
}

pub async fn list_for_stock(db: &Db, user_id: i64, stock_id: i64) -> Result<Vec<Catalyst>> {
    let rows = db
        .with(async |d| {
            Catalyst::all()
                .filter(Catalyst::fields().stock_id().eq(Some(stock_id)))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn get(db: &Db, user_id: i64, id: i64) -> Result<Catalyst> {
    let row = db
        .with(async |d| Catalyst::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewCatalyst<'a> {
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<&'a str>,
    pub country: Option<&'a str>,
    pub catalyst_kind: &'a str,
    pub title: &'a str,
    pub summary_md: Option<&'a str>,
    pub catalyst_date: &'a str,
    pub date_confidence: &'a str,
    pub impact_level: &'a str,
    pub bull_case_md: Option<&'a str>,
    pub bear_case_md: Option<&'a str>,
    pub status: &'a str,
    pub notes: Option<&'a str>,
    pub url: Option<&'a str>,
    pub source: &'a str,
    pub translations: Option<&'a str>,
}

pub async fn create(db: &Db, input: NewCatalyst<'_>) -> Result<Catalyst> {
    let user_id = input.user_id;
    let stock_id = input.stock_id;
    let sector_code = input.sector_code.map(str::to_string);
    let country = input.country.map(str::to_string);
    let catalyst_kind = input.catalyst_kind.to_string();
    let title = input.title.to_string();
    let summary_md = input.summary_md.map(str::to_string);
    let catalyst_date = input.catalyst_date.to_string();
    let date_confidence = input.date_confidence.to_string();
    let impact_level = input.impact_level.to_string();
    let bull_case_md = input.bull_case_md.map(str::to_string);
    let bear_case_md = input.bear_case_md.map(str::to_string);
    let status = input.status.to_string();
    let notes = input.notes.map(str::to_string);
    let url = input.url.map(str::to_string);
    let source = input.source.to_string();
    let translations = input.translations.map(str::to_string);
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(Catalyst {
                user_id: user_id,
                stock_id: stock_id,
                sector_code: sector_code,
                country: country,
                catalyst_kind: catalyst_kind,
                title: title,
                summary_md: summary_md,
                catalyst_date: catalyst_date,
                date_confidence: date_confidence,
                impact_level: impact_level,
                bull_case_md: bull_case_md,
                bear_case_md: bear_case_md,
                status: status,
                notes: notes,
                url: url,
                source: source,
                translations: translations,
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
