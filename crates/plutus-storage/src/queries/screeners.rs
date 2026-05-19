use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::{ScreenerHit, ScreenerRun};

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub name: Option<&'a str>,
    pub kind: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

pub async fn list_runs(db: &Db, filter: ListFilter<'_>) -> Result<Vec<ScreenerRun>> {
    let rows = match (filter.name, filter.kind) {
        (Some(n), Some(k)) => {
            let n = n.to_string();
            let k = k.to_string();
            db.with(async |d| {
                ScreenerRun::all()
                    .filter(ScreenerRun::fields().name().eq(&n))
                    .filter(ScreenerRun::fields().kind().eq(&k))
                    .exec(d)
                    .await
            })
            .await?
        }
        (Some(n), None) => {
            let n = n.to_string();
            db.with(async |d| {
                ScreenerRun::all()
                    .filter(ScreenerRun::fields().name().eq(&n))
                    .exec(d)
                    .await
            })
            .await?
        }
        (None, Some(k)) => {
            let k = k.to_string();
            db.with(async |d| {
                ScreenerRun::all()
                    .filter(ScreenerRun::fields().kind().eq(&k))
                    .exec(d)
                    .await
            })
            .await?
        }
        (None, None) => db.with(async |d| ScreenerRun::all().exec(d).await).await?,
    };
    let user_id = filter.user_id;
    let from = filter.from.map(str::to_string);
    let to = filter.to.map(str::to_string);
    Ok(rows
        .into_iter()
        .filter(|r| r.user_id == user_id)
        .filter(|r| from.as_deref().map_or(true, |f| r.run_date.as_str() >= f))
        .filter(|r| to.as_deref().map_or(true, |t| r.run_date.as_str() <= t))
        .collect())
}

pub async fn get_run(db: &Db, user_id: i64, id: i64) -> Result<ScreenerRun> {
    let row = db
        .with(async |d| ScreenerRun::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewRun<'a> {
    pub user_id: i64,
    pub name: &'a str,
    pub kind: &'a str,
    pub run_date: &'a str,
    pub universe: &'a str,
    pub universe_size: Option<i32>,
    pub criteria: Option<&'a str>,
    pub description_md: Option<&'a str>,
    pub summary_md: Option<&'a str>,
    pub sentiment: Option<&'a str>,
    pub language: &'a str,
    pub source: &'a str,
    pub translations: Option<&'a str>,
}

pub async fn upsert_run(db: &Db, input: NewRun<'_>) -> Result<ScreenerRun> {
    let user_id = input.user_id;
    let name_owned = input.name.to_string();
    let kind_owned = input.kind.to_string();
    let date_owned = input.run_date.to_string();
    let existing = db
        .with(async |d| {
            ScreenerRun::all()
                .filter(ScreenerRun::fields().name().eq(&name_owned))
                .filter(ScreenerRun::fields().kind().eq(&kind_owned))
                .filter(ScreenerRun::fields().run_date().eq(&date_owned))
                .exec(d)
                .await
        })
        .await?
        .into_iter()
        .find(|r| r.user_id == user_id);
    let universe = input.universe.to_string();
    let universe_size = input.universe_size;
    let criteria = input.criteria.map(str::to_string);
    let description_md = input.description_md.map(str::to_string);
    let summary_md = input.summary_md.map(str::to_string);
    let sentiment = input.sentiment.map(str::to_string);
    let language = input.language.to_string();
    let source = input.source.to_string();
    let translations = input.translations.map(str::to_string);
    let now = jiff::Timestamp::now();

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| {
            row.update()
                .universe(universe)
                .universe_size(universe_size)
                .criteria(criteria)
                .description_md(description_md)
                .summary_md(summary_md)
                .sentiment(sentiment)
                .language(language)
                .source(source)
                .translations(translations)
                .updated_at(now)
                .exec(d)
                .await
        })
        .await?;
        get_run(db, user_id, id).await
    } else {
        let name = input.name.to_string();
        let kind = input.kind.to_string();
        let run_date = input.run_date.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(ScreenerRun {
                    user_id: user_id,
                    name: name,
                    kind: kind,
                    run_date: run_date,
                    universe: universe,
                    universe_size: universe_size,
                    criteria: criteria,
                    description_md: description_md,
                    summary_md: summary_md,
                    sentiment: sentiment,
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
}

// ── Hits ──────────────────────────────────────────────────────────────────

pub async fn list_hits(db: &Db, user_id: i64, run_id: i64) -> Result<Vec<ScreenerHit>> {
    let rows = db
        .with(async |d| {
            ScreenerHit::all()
                .filter(ScreenerHit::fields().run_id().eq(run_id))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn list_hits_for_stock(db: &Db, user_id: i64, stock_id: i64) -> Result<Vec<ScreenerHit>> {
    let rows = db
        .with(async |d| {
            ScreenerHit::all()
                .filter(ScreenerHit::fields().stock_id().eq(stock_id))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub struct NewHit<'a> {
    pub user_id: i64,
    pub run_id: i64,
    pub stock_id: i64,
    pub rank: Option<i32>,
    pub score: Option<Decimal>,
    pub rationale_md: Option<&'a str>,
    pub metrics: Option<&'a str>,
    pub translations: Option<&'a str>,
}

pub async fn insert_hit(db: &Db, input: NewHit<'_>) -> Result<ScreenerHit> {
    let rationale_md = input.rationale_md.map(str::to_string);
    let metrics = input.metrics.map(str::to_string);
    let translations = input.translations.map(str::to_string);
    let now = jiff::Timestamp::now();
    let user_id = input.user_id;
    let run_id = input.run_id;
    let stock_id = input.stock_id;
    let rank = input.rank;
    let score = input.score;
    let row = db
        .with(async |d| {
            toasty::create!(ScreenerHit {
                user_id: user_id,
                run_id: run_id,
                stock_id: stock_id,
                rank: rank,
                score: score,
                rationale_md: rationale_md,
                metrics: metrics,
                translations: translations,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
