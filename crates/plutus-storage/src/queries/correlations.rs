use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::{CorrelationPair, UniverseDefinition};

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

#[derive(Debug)]
pub struct LocalizedCorrelationRun {
    pub id: i64,
    pub user_id: i64,
    pub kind: String,
    pub run_date: String,
    pub universe_id: i64,
    pub lookback_days: i32,
    pub method: String,
    pub metrics: Option<String>,
    pub source: String,
    pub summary_md: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

const RUN_PROJECTION: &str = r#"
    id,
    user_id,
    kind,
    run_date,
    universe_id,
    lookback_days,
    method,
    metrics,
    source,
    COALESCE(content -> $1 ->> 'summary_md', content -> 'en' ->> 'summary_md') AS summary_md,
    created_at,
    updated_at
"#;

fn row_to_run(row: &tokio_postgres::Row) -> LocalizedCorrelationRun {
    LocalizedCorrelationRun {
        id: row.get("id"),
        user_id: row.get("user_id"),
        kind: row.get("kind"),
        run_date: row.get("run_date"),
        universe_id: row.get("universe_id"),
        lookback_days: row.get("lookback_days"),
        method: row.get("method"),
        metrics: row.get("metrics"),
        source: row.get("source"),
        summary_md: row.get("summary_md"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list_runs(
    db: &Db,
    user_id: i64,
    locale: &str,
) -> Result<Vec<LocalizedCorrelationRun>> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM correlation_runs WHERE user_id = $2 ORDER BY run_date DESC",
        projection = RUN_PROJECTION,
    );
    let rows = client
        .query(&sql, &[&locale, &user_id])
        .await
        .map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_run).collect())
}

pub async fn get_run(
    db: &Db,
    user_id: i64,
    locale: &str,
    id: i64,
) -> Result<LocalizedCorrelationRun> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM correlation_runs WHERE id = $2 AND user_id = $3",
        projection = RUN_PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id, &user_id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_run).ok_or(DbError::NotFound)
}

pub struct NewRun<'a> {
    pub user_id: i64,
    pub kind: &'a str,
    pub run_date: &'a str,
    pub universe_id: i64,
    pub lookback_days: i32,
    pub method: &'a str,
    pub metrics: Option<&'a str>,
    pub source: &'a str,
    pub content: serde_json::Value,
}

pub async fn create_run(db: &Db, input: NewRun<'_>) -> Result<LocalizedCorrelationRun> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    let sql = r#"
        INSERT INTO correlation_runs
            (user_id, kind, run_date, universe_id, lookback_days, method,
             metrics, source, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.user_id,
                &input.kind,
                &input.run_date,
                &input.universe_id,
                &input.lookback_days,
                &input.method,
                &input.metrics,
                &input.source,
                &content,
                &now,
            ],
        )
        .await
        .map_err(DbError::from)?;
    let id: i64 = row.get(0);
    get_run(db, input.user_id, "en", id).await
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

/// Delete a correlation_run plus its associated pairs in a single
/// transaction. `correlation_pairs.run_id` has no FK in the schema (toasty
/// 0.6 limitation), so the cascade is enforced here. Both deletes scope by
/// `user_id` to keep per-user isolation honest even if a caller passes a
/// `run_id` they don't own — they'll just get NotFound.
pub async fn delete_run(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let mut client = db.raw_client().await?;
    let tx = client.transaction().await.map_err(DbError::from)?;
    tx.execute(
        "DELETE FROM correlation_pairs WHERE run_id = $1 AND user_id = $2",
        &[&id, &user_id],
    )
    .await
    .map_err(DbError::from)?;
    let affected = tx
        .execute(
            "DELETE FROM correlation_runs WHERE id = $1 AND user_id = $2",
            &[&id, &user_id],
        )
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    tx.commit().await.map_err(DbError::from)?;
    Ok(())
}

/// Delete a universe definition. Returns `Conflict` if any correlation_run
/// still references it — agents must delete the runs first. We don't
/// cascade-delete runs from here because run data is expensive to recompute
/// and the caller probably didn't mean to nuke it.
pub async fn delete_universe(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let client = db.raw_client().await?;
    let in_use = client
        .query_one(
            "SELECT COUNT(*)::bigint FROM correlation_runs \
             WHERE universe_id = $1 AND user_id = $2",
            &[&id, &user_id],
        )
        .await
        .map_err(DbError::from)?;
    let count: i64 = in_use.get(0);
    if count > 0 {
        return Err(DbError::Conflict(format!(
            "universe {id} is referenced by {count} correlation run(s); delete those first"
        )));
    }
    let affected = client
        .execute(
            "DELETE FROM universe_definitions WHERE id = $1 AND user_id = $2",
            &[&id, &user_id],
        )
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
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
