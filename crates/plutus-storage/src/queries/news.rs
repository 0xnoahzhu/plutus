//! News queries. All translatable text (title / summary / content_md /
//! agent_summary_md) lives in a single `content JSONB` column shaped as
//!   `{ "<locale>": { "title": ..., "summary": ..., "content_md": ...,
//!                     "agent_summary_md": ... } }`
//! Reads pick the right locale at SELECT time via JSON operators (with a
//! fallback to `en`); writes accept the JSON blob directly. The link tables
//! (`news_stock_links` / `news_sector_links` / `news_macro_links` /
//! `news_country_links`) are non-translatable join tables and continue to
//! be toasty-managed unchanged.

use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::{NewsCountryLink, NewsMacroLink, NewsSectorLink, NewsStockLink};

// ── News items ────────────────────────────────────────────────────────────

/// One news item with translatable fields already projected for the
/// caller's locale by the storage layer. Title / summary / content_md /
/// agent_summary_md may be `None` when neither the requested locale nor
/// `en` has the field populated.
#[derive(Debug, Clone)]
pub struct LocalizedNewsItem {
    pub id: i64,
    pub external_id: Option<String>,
    pub url: String,
    pub canonical_url: Option<String>,
    pub archive_url: Option<String>,
    pub url_status: Option<i32>,
    pub last_verified_at: Option<jiff::Timestamp>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub content_md: Option<String>,
    pub agent_summary_md: Option<String>,
    pub source: String,
    pub source_kind: String,
    pub category: String,
    pub region: String,
    pub published_at: jiff::Timestamp,
    pub fetched_at: jiff::Timestamp,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<Decimal>,
    pub importance: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

/// SQL fragment that pulls a translatable field for the requested locale,
/// falling back to `en` if the locale-specific value is missing. `$1` is
/// the requested locale.
const PROJECTION: &str = r#"
    id,
    external_id,
    url,
    canonical_url,
    archive_url,
    url_status,
    last_verified_at,
    COALESCE(content -> $1 ->> 'title',            content -> 'en' ->> 'title')            AS title,
    COALESCE(content -> $1 ->> 'summary',          content -> 'en' ->> 'summary')          AS summary,
    COALESCE(content -> $1 ->> 'content_md',       content -> 'en' ->> 'content_md')       AS content_md,
    COALESCE(content -> $1 ->> 'agent_summary_md', content -> 'en' ->> 'agent_summary_md') AS agent_summary_md,
    source,
    source_kind,
    category,
    region,
    published_at,
    fetched_at,
    sentiment,
    sentiment_score,
    importance,
    created_at,
    updated_at
"#;

fn row_to_localized(row: &tokio_postgres::Row) -> LocalizedNewsItem {
    LocalizedNewsItem {
        id: row.get("id"),
        external_id: row.get("external_id"),
        url: row.get("url"),
        canonical_url: row.get("canonical_url"),
        archive_url: row.get("archive_url"),
        url_status: row.get("url_status"),
        last_verified_at: row.get("last_verified_at"),
        title: row.get("title"),
        summary: row.get("summary"),
        content_md: row.get("content_md"),
        agent_summary_md: row.get("agent_summary_md"),
        source: row.get("source"),
        source_kind: row.get("source_kind"),
        category: row.get("category"),
        region: row.get("region"),
        published_at: row.get("published_at"),
        fetched_at: row.get("fetched_at"),
        sentiment: row.get("sentiment"),
        sentiment_score: row.get("sentiment_score"),
        importance: row.get("importance"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list(db: &Db, locale: &str) -> Result<Vec<LocalizedNewsItem>> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM news_items ORDER BY published_at DESC",
        projection = PROJECTION,
    );
    let rows = client.query(&sql, &[&locale]).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn get(db: &Db, locale: &str, id: i64) -> Result<LocalizedNewsItem> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM news_items WHERE id = $2",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_localized).ok_or(DbError::NotFound)
}

pub async fn find_by_url(db: &Db, locale: &str, url: &str) -> Result<Option<LocalizedNewsItem>> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM news_items WHERE url = $2",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &url])
        .await
        .map_err(DbError::from)?;
    Ok(row.as_ref().map(row_to_localized))
}

pub struct NewNewsItem<'a> {
    pub external_id: Option<&'a str>,
    pub url: &'a str,
    pub canonical_url: Option<&'a str>,
    pub archive_url: Option<&'a str>,
    pub url_status: Option<i32>,
    pub last_verified_at: Option<jiff::Timestamp>,
    pub source: &'a str,
    pub source_kind: &'a str,
    pub category: &'a str,
    pub region: &'a str,
    pub published_at: jiff::Timestamp,
    pub fetched_at: Option<jiff::Timestamp>,
    pub sentiment: Option<&'a str>,
    pub sentiment_score: Option<Decimal>,
    pub importance: &'a str,
    /// Full multi-locale content blob. Shape is
    /// `{ "<locale>": { "title": ..., "summary": ..., "content_md": ...,
    ///                   "agent_summary_md": ... } }`.
    pub content: serde_json::Value,
}

pub async fn create(db: &Db, input: NewNewsItem<'_>) -> Result<LocalizedNewsItem> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let fetched_at = input.fetched_at.unwrap_or(now);
    let content = &input.content;
    let sql = r#"
        INSERT INTO news_items
            (external_id, url, canonical_url, archive_url, url_status,
             last_verified_at, source, source_kind, category, region,
             published_at, fetched_at, sentiment, sentiment_score, importance,
             content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $17)
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.external_id,
                &input.url,
                &input.canonical_url,
                &input.archive_url,
                &input.url_status,
                &input.last_verified_at,
                &input.source,
                &input.source_kind,
                &input.category,
                &input.region,
                &input.published_at,
                &fetched_at,
                &input.sentiment,
                &input.sentiment_score,
                &input.importance,
                &content,
                &now,
            ],
        )
        .await
        .map_err(DbError::from)?;
    let id: i64 = row.get(0);
    get(db, "en", id).await
}

/// PATCH-style partial update. Every field is `Option<…>`; only the
/// `Some(…)` fields make it into the UPDATE. Always bumps `updated_at`.
///
/// `content` is merged into the existing JSONB with the `||` operator
/// so e.g. `{ "zh-CN": { "title": "…" } }` adds/replaces just the
/// zh-CN locale and leaves `en` untouched — the common case is fixing
/// or translating one locale on an existing row. To wipe a locale,
/// caller can send a full `content` via DELETE + re-create instead.
pub struct NewsPatch<'a> {
    pub external_id: Option<&'a str>,
    pub url: Option<&'a str>,
    pub canonical_url: Option<&'a str>,
    pub archive_url: Option<&'a str>,
    pub url_status: Option<i32>,
    pub last_verified_at: Option<jiff::Timestamp>,
    pub source: Option<&'a str>,
    pub source_kind: Option<&'a str>,
    pub category: Option<&'a str>,
    pub region: Option<&'a str>,
    pub published_at: Option<jiff::Timestamp>,
    pub fetched_at: Option<jiff::Timestamp>,
    pub sentiment: Option<&'a str>,
    pub sentiment_score: Option<Decimal>,
    pub importance: Option<&'a str>,
    pub content: Option<serde_json::Value>,
}

pub async fn update(db: &Db, id: i64, patch: NewsPatch<'_>) -> Result<LocalizedNewsItem> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();

    // Build SET clause dynamically. `updated_at` always present so the
    // row's mtime moves even on a no-other-fields patch. Each `Some(_)`
    // field appends one fragment + binds one param; the trailing
    // `WHERE id = $N` slot is reserved after all fields are placed.
    let mut sets: Vec<String> = vec!["updated_at = $1".to_string()];
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&now];

    if let Some(v) = patch.external_id.as_ref() {
        sets.push(format!("external_id = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.url.as_ref() {
        sets.push(format!("url = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.canonical_url.as_ref() {
        sets.push(format!("canonical_url = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.archive_url.as_ref() {
        sets.push(format!("archive_url = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.url_status.as_ref() {
        sets.push(format!("url_status = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.last_verified_at.as_ref() {
        sets.push(format!("last_verified_at = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.source.as_ref() {
        sets.push(format!("source = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.source_kind.as_ref() {
        sets.push(format!("source_kind = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.category.as_ref() {
        sets.push(format!("category = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.region.as_ref() {
        sets.push(format!("region = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.published_at.as_ref() {
        sets.push(format!("published_at = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.fetched_at.as_ref() {
        sets.push(format!("fetched_at = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.sentiment.as_ref() {
        sets.push(format!("sentiment = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.sentiment_score.as_ref() {
        sets.push(format!("sentiment_score = ${}", params.len() + 1));
        params.push(v);
    }
    if let Some(v) = patch.importance.as_ref() {
        sets.push(format!("importance = ${}", params.len() + 1));
        params.push(v);
    }
    // Content uses JSONB merge so a partial blob (one locale) doesn't
    // wipe the others. Full-replace requires DELETE + create.
    if let Some(v) = patch.content.as_ref() {
        sets.push(format!("content = content || ${}::jsonb", params.len() + 1));
        params.push(v);
    }

    let sql = format!(
        "UPDATE news_items SET {} WHERE id = ${}",
        sets.join(", "),
        params.len() + 1,
    );
    params.push(&id);

    let affected = client.execute(&sql, &params).await.map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    get(db, "en", id).await
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let client = db.raw_client().await?;
    let affected = client
        .execute("DELETE FROM news_items WHERE id = $1", &[&id])
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

// ── Stock links ───────────────────────────────────────────────────────────

pub async fn list_stock_links(db: &Db, news_id: i64) -> Result<Vec<NewsStockLink>> {
    db.with(async |d| {
        NewsStockLink::all()
            .filter(NewsStockLink::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn list_news_for_stock(db: &Db, stock_id: i64) -> Result<Vec<NewsStockLink>> {
    db.with(async |d| {
        NewsStockLink::all()
            .filter(NewsStockLink::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn add_stock_link(
    db: &Db,
    news_id: i64,
    stock_id: i64,
    relation: &str,
    relevance: Option<Decimal>,
) -> Result<NewsStockLink> {
    let relation = relation.to_string();
    let row = db
        .with(async |d| {
            toasty::create!(NewsStockLink {
                news_id: news_id,
                stock_id: stock_id,
                relation: relation,
                relevance: relevance,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

// ── Sector / macro / country links ────────────────────────────────────────

pub async fn add_sector_link(db: &Db, news_id: i64, sector_code: &str) -> Result<NewsSectorLink> {
    let sector_code = sector_code.to_string();
    db.with(async |d| {
        toasty::create!(NewsSectorLink { news_id: news_id, sector_code: sector_code })
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn add_macro_link(
    db: &Db,
    news_id: i64,
    indicator_code: &str,
) -> Result<NewsMacroLink> {
    let indicator_code = indicator_code.to_string();
    db.with(async |d| {
        toasty::create!(NewsMacroLink { news_id: news_id, indicator_code: indicator_code })
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn add_country_link(db: &Db, news_id: i64, country: &str) -> Result<NewsCountryLink> {
    let country = country.to_string();
    db.with(async |d| {
        toasty::create!(NewsCountryLink { news_id: news_id, country: country })
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn list_sector_links(db: &Db, news_id: i64) -> Result<Vec<NewsSectorLink>> {
    db.with(async |d| {
        NewsSectorLink::all()
            .filter(NewsSectorLink::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn list_macro_links(db: &Db, news_id: i64) -> Result<Vec<NewsMacroLink>> {
    db.with(async |d| {
        NewsMacroLink::all()
            .filter(NewsMacroLink::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn list_country_links(db: &Db, news_id: i64) -> Result<Vec<NewsCountryLink>> {
    db.with(async |d| {
        NewsCountryLink::all()
            .filter(NewsCountryLink::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}
