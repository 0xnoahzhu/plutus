//! Market brief queries. All translatable text lives in a single
//! `content JSONB` column shaped as
//!   `{ "<locale>": { "headline": ..., "content_md": ... } }`
//! Reads pick the right locale at SELECT time via JSON operators (with a
//! fallback to `en`); writes accept the JSON blob directly. There are no
//! per-locale base columns to keep in sync.

use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};

/// One market brief, with the translatable fields already projected for the
/// caller's locale by the storage layer.
#[derive(Debug)]
pub struct LocalizedMarketBrief {
    pub id: i64,
    pub user_id: i64,
    pub country: String,
    pub kind: String,
    pub trade_date: String,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<Decimal>,
    pub source: String,
    pub headline: Option<String>,
    pub content_md: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub locale: &'a str,
    pub country: Option<&'a str>,
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
    country,
    kind,
    trade_date,
    sentiment,
    sentiment_score,
    source,
    COALESCE(content -> $1 ->> 'headline',   content -> 'en' ->> 'headline')   AS headline,
    COALESCE(content -> $1 ->> 'content_md', content -> 'en' ->> 'content_md') AS content_md,
    created_at,
    updated_at
"#;

fn row_to_localized(row: &tokio_postgres::Row) -> LocalizedMarketBrief {
    LocalizedMarketBrief {
        id: row.get("id"),
        user_id: row.get("user_id"),
        country: row.get("country"),
        kind: row.get("kind"),
        trade_date: row.get("trade_date"),
        sentiment: row.get("sentiment"),
        sentiment_score: row.get("sentiment_score"),
        source: row.get("source"),
        headline: row.get("headline"),
        content_md: row.get("content_md"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<LocalizedMarketBrief>> {
    let client = db.raw_client().await?;
    let mut sql = format!(
        "SELECT {projection} FROM market_briefs WHERE user_id = $2",
        projection = PROJECTION,
    );
    let mut args: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        vec![&filter.locale, &filter.user_id];
    let country_owned;
    if let Some(c) = filter.country {
        country_owned = c.to_string();
        sql.push_str(&format!(" AND country = ${}", args.len() + 1));
        args.push(&country_owned);
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
        sql.push_str(&format!(" AND trade_date >= ${}", args.len() + 1));
        args.push(&from_owned);
    }
    let to_owned;
    if let Some(t) = filter.to {
        to_owned = t.to_string();
        sql.push_str(&format!(" AND trade_date <= ${}", args.len() + 1));
        args.push(&to_owned);
    }
    sql.push_str(" ORDER BY trade_date DESC, kind ASC, country ASC");

    let rows = client.query(&sql, &args).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn get(db: &Db, user_id: i64, locale: &str, id: i64) -> Result<LocalizedMarketBrief> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM market_briefs WHERE id = $2 AND user_id = $3",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id, &user_id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_localized).ok_or(DbError::NotFound)
}

pub struct NewBrief<'a> {
    pub user_id: i64,
    pub country: &'a str,
    pub kind: &'a str,
    pub trade_date: &'a str,
    pub sentiment: Option<&'a str>,
    pub sentiment_score: Option<Decimal>,
    pub source: &'a str,
    /// Full multi-locale content blob. Shape is
    /// `{ "<locale>": { "headline": ..., "content_md": ... } }`.
    pub content: serde_json::Value,
}

/// Upsert on the natural key `(user_id, country, kind, trade_date)`. Writes
/// the whole `content` JSONB blob verbatim — partial-locale updates are the
/// caller's responsibility.
pub async fn upsert(db: &Db, input: NewBrief<'_>) -> Result<LocalizedMarketBrief> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    let sql = r#"
        INSERT INTO market_briefs
            (user_id, country, kind, trade_date, sentiment, sentiment_score,
             source, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
        ON CONFLICT (user_id, country, kind, trade_date) DO UPDATE SET
            sentiment       = EXCLUDED.sentiment,
            sentiment_score = EXCLUDED.sentiment_score,
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
                &input.country,
                &input.kind,
                &input.trade_date,
                &input.sentiment,
                &input.sentiment_score,
                &input.source,
                &content,
                &now,
            ],
        )
        .await
        .map_err(DbError::from)?;
    let id: i64 = row.get(0);
    get(db, input.user_id, "en", id).await
}

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let client = db.raw_client().await?;
    let affected = client
        .execute(
            "DELETE FROM market_briefs WHERE id = $1 AND user_id = $2",
            &[&id, &user_id],
        )
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
