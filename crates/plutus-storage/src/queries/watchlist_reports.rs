//! Watchlist report queries. All translatable text lives in a single
//! `content JSONB` column shaped as
//!   `{ "<locale>": { "headline": ..., "summary_md": ..., "content_md": ..., "notes": ... } }`
//! Reads pick the right locale at SELECT time via JSON operators (with a
//! fallback to `en`); writes accept the JSON blob directly. There are no
//! per-locale base columns to keep in sync.

use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};

/// One watchlist report, with the translatable fields already projected
/// for the caller's locale by the storage layer.
#[derive(Debug)]
pub struct LocalizedWatchlistReport {
    pub id: i64,
    pub user_id: i64,
    pub kind: String,
    pub period_start: String,
    pub period_end: String,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<Decimal>,
    pub metrics: Option<String>,
    pub source: String,
    pub headline: Option<String>,
    pub summary_md: Option<String>,
    pub content_md: Option<String>,
    pub notes: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub locale: &'a str,
    pub kind: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

/// SQL fragment that pulls a translatable field for the requested locale,
/// falling back to `en` if the locale-specific value is missing. `$1` is
/// the requested locale.
const PROJECTION: &str = r#"
    id,
    user_id,
    kind,
    period_start,
    period_end,
    sentiment,
    sentiment_score,
    metrics,
    source,
    COALESCE(content -> $1 ->> 'headline',   content -> 'en' ->> 'headline')   AS headline,
    COALESCE(content -> $1 ->> 'summary_md', content -> 'en' ->> 'summary_md') AS summary_md,
    COALESCE(content -> $1 ->> 'content_md', content -> 'en' ->> 'content_md') AS content_md,
    COALESCE(content -> $1 ->> 'notes',      content -> 'en' ->> 'notes')      AS notes,
    created_at,
    updated_at
"#;

fn row_to_localized(row: &tokio_postgres::Row) -> LocalizedWatchlistReport {
    LocalizedWatchlistReport {
        id: row.get("id"),
        user_id: row.get("user_id"),
        kind: row.get("kind"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        sentiment: row.get("sentiment"),
        sentiment_score: row.get("sentiment_score"),
        metrics: row.get("metrics"),
        source: row.get("source"),
        headline: row.get("headline"),
        summary_md: row.get("summary_md"),
        content_md: row.get("content_md"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<LocalizedWatchlistReport>> {
    let client = db.raw_client().await?;
    let mut sql = format!(
        "SELECT {projection} FROM watchlist_reports WHERE user_id = $2",
        projection = PROJECTION,
    );
    let mut args: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&filter.locale, &filter.user_id];
    let kind_owned;
    if let Some(k) = filter.kind {
        kind_owned = k.to_string();
        sql.push_str(" AND kind = $3");
        args.push(&kind_owned);
    }
    let from_owned;
    if let Some(f) = filter.from {
        from_owned = f.to_string();
        sql.push_str(&format!(" AND period_start >= ${}", args.len() + 1));
        args.push(&from_owned);
    }
    let to_owned;
    if let Some(t) = filter.to {
        to_owned = t.to_string();
        sql.push_str(&format!(" AND period_start <= ${}", args.len() + 1));
        args.push(&to_owned);
    }
    sql.push_str(" ORDER BY period_start DESC, kind ASC");

    let rows = client.query(&sql, &args).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn get(db: &Db, user_id: i64, locale: &str, id: i64) -> Result<LocalizedWatchlistReport> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM watchlist_reports WHERE id = $2 AND user_id = $3",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id, &user_id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_localized).ok_or(DbError::NotFound)
}

pub struct NewReport<'a> {
    pub user_id: i64,
    pub kind: &'a str,
    pub period_start: &'a str,
    pub period_end: &'a str,
    pub sentiment: Option<&'a str>,
    pub sentiment_score: Option<Decimal>,
    pub metrics: Option<&'a str>,
    pub source: &'a str,
    /// Full multi-locale content blob. Shape is
    /// `{ "<locale>": { "headline": ..., "summary_md": ..., "content_md": ..., "notes": ... } }`.
    /// Callers can omit fields per locale; the SELECT-time COALESCE handles
    /// fallbacks to `en`.
    pub content: serde_json::Value,
}

/// Upsert on the natural key `(user_id, kind, period_start)`. Writes the
/// whole `content` JSONB blob verbatim — partial-locale updates are the
/// caller's responsibility (merge in app code, then re-upsert).
pub async fn upsert(db: &Db, input: NewReport<'_>) -> Result<LocalizedWatchlistReport> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    let sql = r#"
        INSERT INTO watchlist_reports
            (user_id, kind, period_start, period_end, sentiment, sentiment_score,
             metrics, source, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
        ON CONFLICT (user_id, kind, period_start) DO UPDATE SET
            period_end      = EXCLUDED.period_end,
            sentiment       = EXCLUDED.sentiment,
            sentiment_score = EXCLUDED.sentiment_score,
            metrics         = EXCLUDED.metrics,
            source          = EXCLUDED.source,
            content         = EXCLUDED.content,
            updated_at      = EXCLUDED.updated_at
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.user_id,
                &input.kind,
                &input.period_start,
                &input.period_end,
                &input.sentiment,
                &input.sentiment_score,
                &input.metrics,
                &input.source,
                &content,
                &now,
            ],
        )
        .await
        .map_err(DbError::from)?;
    let id: i64 = row.get(0);
    // Read back through the locale projection so the caller gets a fully
    // populated `LocalizedWatchlistReport`. Default to `en` since the
    // upsert response is mostly used by the agent which writes English.
    get(db, input.user_id, "en", id).await
}

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let client = db.raw_client().await?;
    let affected = client
        .execute(
            "DELETE FROM watchlist_reports WHERE id = $1 AND user_id = $2",
            &[&id, &user_id],
        )
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
