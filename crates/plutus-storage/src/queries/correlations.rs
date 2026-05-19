use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::{CorrelationPair, CorrelationRun, UniverseDefinition};

// ── Universe definitions ──────────────────────────────────────────────────

pub async fn list_universes(db: &Db, user_id: i64) -> Result<Vec<UniverseDefinition>> {
    let rows = db
        .with(async |d| UniverseDefinition::all().exec(d).await)
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn get_universe(db: &Db, user_id: i64, id: i64) -> Result<UniverseDefinition> {
    let row = db
        .with(async |d| UniverseDefinition::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub async fn get_universe_by_name(
    db: &Db,
    user_id: i64,
    name: &str,
) -> Result<Option<UniverseDefinition>> {
    let name = name.to_string();
    let row = db
        .with(async |d| {
            UniverseDefinition::all()
                .filter(UniverseDefinition::fields().name().eq(&name))
                .exec(d)
                .await
        })
        .await?
        .into_iter()
        .find(|r| r.user_id == user_id);
    Ok(row)
}

pub async fn upsert_universe(
    db: &Db,
    user_id: i64,
    name: &str,
    description_md: Option<&str>,
    stock_ids_json: &str,
) -> Result<UniverseDefinition> {
    let existing = get_universe_by_name(db, user_id, name).await?;
    let description_md = description_md.map(str::to_string);
    let stock_ids = stock_ids_json.to_string();
    let now = jiff::Timestamp::now();

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| {
            row.update()
                .description_md(description_md)
                .stock_ids(stock_ids)
                .updated_at(now)
                .exec(d)
                .await
        })
        .await?;
        get_universe(db, user_id, id).await
    } else {
        let name = name.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(UniverseDefinition {
                    user_id: user_id,
                    name: name,
                    description_md: description_md,
                    stock_ids: stock_ids,
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

// ── Runs ──────────────────────────────────────────────────────────────────

pub async fn list_runs(db: &Db, user_id: i64) -> Result<Vec<CorrelationRun>> {
    let rows = db
        .with(async |d| CorrelationRun::all().exec(d).await)
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn get_run(db: &Db, user_id: i64, id: i64) -> Result<CorrelationRun> {
    let row = db
        .with(async |d| CorrelationRun::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewRun<'a> {
    pub user_id: i64,
    pub kind: &'a str,
    pub run_date: &'a str,
    pub universe_id: i64,
    pub lookback_days: i32,
    pub method: &'a str,
    pub summary_md: Option<&'a str>,
    pub metrics: Option<&'a str>,
    pub source: &'a str,
    pub translations: Option<&'a str>,
}

pub async fn create_run(db: &Db, input: NewRun<'_>) -> Result<CorrelationRun> {
    let user_id = input.user_id;
    let kind = input.kind.to_string();
    let run_date = input.run_date.to_string();
    let universe_id = input.universe_id;
    let lookback_days = input.lookback_days;
    let method = input.method.to_string();
    let summary_md = input.summary_md.map(str::to_string);
    let metrics = input.metrics.map(str::to_string);
    let source = input.source.to_string();
    let translations = input.translations.map(str::to_string);
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(CorrelationRun {
                user_id: user_id,
                kind: kind,
                run_date: run_date,
                universe_id: universe_id,
                lookback_days: lookback_days,
                method: method,
                summary_md: summary_md,
                metrics: metrics,
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

// ── Pairs ─────────────────────────────────────────────────────────────────

pub async fn list_pairs(db: &Db, user_id: i64, run_id: i64) -> Result<Vec<CorrelationPair>> {
    let rows = db
        .with(async |d| {
            CorrelationPair::all()
                .filter(CorrelationPair::fields().run_id().eq(run_id))
                .exec(d)
                .await
        })
        .await?;
    Ok(rows.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub async fn list_pairs_for_stock(
    db: &Db,
    user_id: i64,
    stock_id: i64,
) -> Result<Vec<CorrelationPair>> {
    let from_a: Vec<CorrelationPair> = db
        .with(async |d| {
            CorrelationPair::all()
                .filter(CorrelationPair::fields().stock_a_id().eq(stock_id))
                .exec(d)
                .await
        })
        .await?;
    let from_b: Vec<CorrelationPair> = db
        .with(async |d| {
            CorrelationPair::all()
                .filter(CorrelationPair::fields().stock_b_id().eq(stock_id))
                .exec(d)
                .await
        })
        .await?;
    let mut all = from_a;
    all.extend(from_b);
    Ok(all.into_iter().filter(|r| r.user_id == user_id).collect())
}

pub struct NewPair {
    pub user_id: i64,
    pub run_id: i64,
    pub stock_a_id: i64,
    pub stock_b_id: i64,
    pub correlation: Decimal,
}

pub async fn insert_pair(db: &Db, input: NewPair) -> Result<CorrelationPair> {
    let (a, b) = if input.stock_a_id <= input.stock_b_id {
        (input.stock_a_id, input.stock_b_id)
    } else {
        (input.stock_b_id, input.stock_a_id)
    };
    let user_id = input.user_id;
    let run_id = input.run_id;
    let correlation = input.correlation;
    let row = db
        .with(async |d| {
            toasty::create!(CorrelationPair {
                user_id: user_id,
                run_id: run_id,
                stock_a_id: a,
                stock_b_id: b,
                correlation: correlation,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
