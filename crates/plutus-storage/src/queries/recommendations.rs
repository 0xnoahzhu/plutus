use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::Recommendation;

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub status: Option<&'a str>,
    pub from: Option<&'a str>, // YYYY-MM-DD on issued_at
    pub to: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<Recommendation>> {
    let rows = match (filter.stock_id, filter.status) {
        (Some(s), Some(st)) => {
            let st = st.to_string();
            db.with(async |d| {
                Recommendation::all()
                    .filter(Recommendation::fields().stock_id().eq(Some(s)))
                    .filter(Recommendation::fields().status().eq(&st))
                    .exec(d)
                    .await
            })
            .await?
        }
        (Some(s), None) => {
            db.with(async |d| {
                Recommendation::all()
                    .filter(Recommendation::fields().stock_id().eq(Some(s)))
                    .exec(d)
                    .await
            })
            .await?
        }
        (None, Some(st)) => {
            let st = st.to_string();
            db.with(async |d| {
                Recommendation::all()
                    .filter(Recommendation::fields().status().eq(&st))
                    .exec(d)
                    .await
            })
            .await?
        }
        (None, None) => db.with(async |d| Recommendation::all().exec(d).await).await?,
    };
    let user_id = filter.user_id;
    Ok(rows
        .into_iter()
        .filter(|r| r.user_id == user_id)
        .filter(|r| {
            filter.from.map_or(true, |f| r.issued_at.to_string().as_str() >= f)
        })
        .filter(|r| {
            filter.to.map_or(true, |t| r.issued_at.to_string().as_str() <= t)
        })
        .collect())
}

pub async fn get(db: &Db, user_id: i64, id: i64) -> Result<Recommendation> {
    let row = db
        .with(async |d| Recommendation::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewRecommendation<'a> {
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<&'a str>,
    pub action: &'a str,
    pub confidence: Option<Decimal>,
    pub rationale_md: &'a str,
    pub target_price: Option<Decimal>,
    pub target_currency: Option<&'a str>,
    pub target_horizon: &'a str,
    pub issued_at: jiff::Timestamp,
    pub language: &'a str,
    pub source: &'a str,
    pub translations: Option<&'a str>,
}

pub async fn create(db: &Db, input: NewRecommendation<'_>) -> Result<Recommendation> {
    let user_id = input.user_id;
    let stock_id = input.stock_id;
    let sector_code = input.sector_code.map(str::to_string);
    let action = input.action.to_string();
    let confidence = input.confidence;
    let rationale_md = input.rationale_md.to_string();
    let target_price = input.target_price;
    let target_currency = input.target_currency.map(str::to_string);
    let target_horizon = input.target_horizon.to_string();
    let issued_at = input.issued_at;
    let language = input.language.to_string();
    let source = input.source.to_string();
    let translations = input.translations.map(str::to_string);
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(Recommendation {
                user_id: user_id,
                stock_id: stock_id,
                sector_code: sector_code,
                action: action,
                confidence: confidence,
                rationale_md: rationale_md,
                target_price: target_price,
                target_currency: target_currency,
                target_horizon: target_horizon,
                issued_at: issued_at,
                status: "open".to_string(),
                outcome_md: None::<String>,
                pnl_pct: None::<Decimal>,
                closed_at: None::<jiff::Timestamp>,
                language: language,
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

pub struct ClosePatch<'a> {
    pub status: &'a str, // "closed_correct" / "closed_wrong" / "closed_neutral" / "expired"
    pub outcome_md: Option<&'a str>,
    pub pnl_pct: Option<Decimal>,
    pub closed_at: jiff::Timestamp,
}

pub async fn close(db: &Db, user_id: i64, id: i64, patch: ClosePatch<'_>) -> Result<Recommendation> {
    let mut row = get(db, user_id, id).await?;
    let status = patch.status.to_string();
    let outcome_md = patch.outcome_md.map(str::to_string);
    let pnl_pct = patch.pnl_pct;
    let closed_at = patch.closed_at;
    let now = jiff::Timestamp::now();
    db.with(async |d| {
        row.update()
            .status(status)
            .outcome_md(outcome_md)
            .pnl_pct(pnl_pct)
            .closed_at(Some(closed_at))
            .updated_at(now)
            .exec(d)
            .await
    })
    .await?;
    get(db, user_id, id).await
}

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let row = get(db, user_id, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
