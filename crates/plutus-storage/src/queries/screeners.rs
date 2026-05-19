//! Screener queries — runs and hits. Translatable text on both lives in a
//! `content JSONB` column; reads pick the right locale at SELECT time, with
//! fallback to `en`. Writes accept a `serde_json::Value` blob.

use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};

// ── Runs ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LocalizedScreenerRun {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub kind: String,
    pub run_date: String,
    pub universe: String,
    pub universe_size: Option<i32>,
    pub criteria: Option<String>,
    pub sentiment: Option<String>,
    pub source: String,
    pub description_md: Option<String>,
    pub summary_md: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub locale: &'a str,
    pub name: Option<&'a str>,
    pub kind: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

const RUN_PROJECTION: &str = r#"
    id,
    user_id,
    name,
    kind,
    run_date,
    universe,
    universe_size,
    criteria,
    sentiment,
    source,
    COALESCE(content -> $1 ->> 'description_md', content -> 'en' ->> 'description_md') AS description_md,
    COALESCE(content -> $1 ->> 'summary_md',     content -> 'en' ->> 'summary_md')     AS summary_md,
    created_at,
    updated_at
"#;

fn row_to_run(row: &tokio_postgres::Row) -> LocalizedScreenerRun {
    LocalizedScreenerRun {
        id: row.get("id"),
        user_id: row.get("user_id"),
        name: row.get("name"),
        kind: row.get("kind"),
        run_date: row.get("run_date"),
        universe: row.get("universe"),
        universe_size: row.get("universe_size"),
        criteria: row.get("criteria"),
        sentiment: row.get("sentiment"),
        source: row.get("source"),
        description_md: row.get("description_md"),
        summary_md: row.get("summary_md"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list_runs(db: &Db, filter: ListFilter<'_>) -> Result<Vec<LocalizedScreenerRun>> {
    let client = db.raw_client().await?;
    let mut sql = format!(
        "SELECT {projection} FROM screener_runs WHERE user_id = $2",
        projection = RUN_PROJECTION,
    );
    let mut args: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        vec![&filter.locale, &filter.user_id];
    let name_owned;
    if let Some(n) = filter.name {
        name_owned = n.to_string();
        sql.push_str(&format!(" AND name = ${}", args.len() + 1));
        args.push(&name_owned);
    }
    let kind_owned;
    if let Some(k) = filter.kind {
        kind_owned = k.to_string();
        sql.push_str(&format!(" AND kind = ${}", args.len() + 1));
        args.push(&kind_owned);
    }
    let from_owned;
    if let Some(f) = filter.from {
        from_owned = f.to_string();
        sql.push_str(&format!(" AND run_date >= ${}", args.len() + 1));
        args.push(&from_owned);
    }
    let to_owned;
    if let Some(t) = filter.to {
        to_owned = t.to_string();
        sql.push_str(&format!(" AND run_date <= ${}", args.len() + 1));
        args.push(&to_owned);
    }
    sql.push_str(" ORDER BY run_date DESC, name ASC");

    let rows = client.query(&sql, &args).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_run).collect())
}

pub async fn get_run(db: &Db, user_id: i64, locale: &str, id: i64) -> Result<LocalizedScreenerRun> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM screener_runs WHERE id = $2 AND user_id = $3",
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
    pub name: &'a str,
    pub kind: &'a str,
    pub run_date: &'a str,
    pub universe: &'a str,
    pub universe_size: Option<i32>,
    pub criteria: Option<&'a str>,
    pub sentiment: Option<&'a str>,
    pub source: &'a str,
    pub content: serde_json::Value,
}

pub async fn upsert_run(db: &Db, input: NewRun<'_>) -> Result<LocalizedScreenerRun> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    let sql = r#"
        INSERT INTO screener_runs
            (user_id, name, kind, run_date, universe, universe_size, criteria,
             sentiment, source, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
        ON CONFLICT (user_id, name, kind, run_date) DO UPDATE SET
            universe      = EXCLUDED.universe,
            universe_size = EXCLUDED.universe_size,
            criteria      = EXCLUDED.criteria,
            sentiment     = EXCLUDED.sentiment,
            source        = EXCLUDED.source,
            content       = EXCLUDED.content,
            updated_at    = EXCLUDED.updated_at
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.user_id,
                &input.name,
                &input.kind,
                &input.run_date,
                &input.universe,
                &input.universe_size,
                &input.criteria,
                &input.sentiment,
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

// ── Hits ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LocalizedScreenerHit {
    pub id: i64,
    pub user_id: i64,
    pub run_id: i64,
    pub stock_id: i64,
    pub rank: Option<i32>,
    pub score: Option<Decimal>,
    pub metrics: Option<String>,
    pub rationale_md: Option<String>,
    pub created_at: jiff::Timestamp,
}

const HIT_PROJECTION: &str = r#"
    id,
    user_id,
    run_id,
    stock_id,
    rank,
    score,
    metrics,
    COALESCE(content -> $1 ->> 'rationale_md', content -> 'en' ->> 'rationale_md') AS rationale_md,
    created_at
"#;

fn row_to_hit(row: &tokio_postgres::Row) -> LocalizedScreenerHit {
    LocalizedScreenerHit {
        id: row.get("id"),
        user_id: row.get("user_id"),
        run_id: row.get("run_id"),
        stock_id: row.get("stock_id"),
        rank: row.get("rank"),
        score: row.get("score"),
        metrics: row.get("metrics"),
        rationale_md: row.get("rationale_md"),
        created_at: row.get("created_at"),
    }
}

pub async fn list_hits(
    db: &Db,
    user_id: i64,
    locale: &str,
    run_id: i64,
) -> Result<Vec<LocalizedScreenerHit>> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM screener_hits WHERE user_id = $2 AND run_id = $3 ORDER BY rank ASC NULLS LAST",
        projection = HIT_PROJECTION,
    );
    let rows = client
        .query(&sql, &[&locale, &user_id, &run_id])
        .await
        .map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_hit).collect())
}

pub async fn list_hits_for_stock(
    db: &Db,
    user_id: i64,
    locale: &str,
    stock_id: i64,
) -> Result<Vec<LocalizedScreenerHit>> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM screener_hits WHERE user_id = $2 AND stock_id = $3 ORDER BY created_at DESC",
        projection = HIT_PROJECTION,
    );
    let rows = client
        .query(&sql, &[&locale, &user_id, &stock_id])
        .await
        .map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_hit).collect())
}

pub struct NewHit<'a> {
    pub user_id: i64,
    pub run_id: i64,
    pub stock_id: i64,
    pub rank: Option<i32>,
    pub score: Option<Decimal>,
    pub metrics: Option<&'a str>,
    pub content: serde_json::Value,
}

pub async fn insert_hit(db: &Db, input: NewHit<'_>) -> Result<LocalizedScreenerHit> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    let sql = r#"
        INSERT INTO screener_hits
            (user_id, run_id, stock_id, rank, score, metrics, content, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.user_id,
                &input.run_id,
                &input.stock_id,
                &input.rank,
                &input.score,
                &input.metrics,
                &content,
                &now,
            ],
        )
        .await
        .map_err(DbError::from)?;
    let id: i64 = row.get(0);
    let sql = format!(
        "SELECT {projection} FROM screener_hits WHERE id = $2",
        projection = HIT_PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&"en", &id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_hit).ok_or(DbError::NotFound)
}
