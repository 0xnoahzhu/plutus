//! Catalyst queries. All translatable text lives in a single
//! `content JSONB` column shaped as
//!   `{ "<locale>": { "title": ..., "summary_md": ..., "bull_case_md": ...,
//!                     "bear_case_md": ..., "notes": ... } }`

use crate::db::{Db, DbError, Result};

#[derive(Debug)]
pub struct LocalizedCatalyst {
    pub id: i64,
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<String>,
    pub country: Option<String>,
    pub catalyst_kind: String,
    pub catalyst_date: String,
    pub date_confidence: String,
    pub impact_level: String,
    pub status: String,
    pub url: Option<String>,
    pub source: String,
    pub title: Option<String>,
    pub summary_md: Option<String>,
    pub bull_case_md: Option<String>,
    pub bear_case_md: Option<String>,
    pub notes: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub locale: &'a str,
    pub stock_id: Option<i64>,
    pub sector_code: Option<&'a str>,
    pub country: Option<&'a str>,
    pub catalyst_kind: Option<&'a str>,
    pub status: Option<&'a str>,
    pub impact_level: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

const PROJECTION: &str = r#"
    id,
    user_id,
    stock_id,
    sector_code,
    country,
    catalyst_kind,
    catalyst_date,
    date_confidence,
    impact_level,
    status,
    url,
    source,
    COALESCE(content -> $1 ->> 'title',        content -> 'en' ->> 'title')        AS title,
    COALESCE(content -> $1 ->> 'summary_md',   content -> 'en' ->> 'summary_md')   AS summary_md,
    COALESCE(content -> $1 ->> 'bull_case_md', content -> 'en' ->> 'bull_case_md') AS bull_case_md,
    COALESCE(content -> $1 ->> 'bear_case_md', content -> 'en' ->> 'bear_case_md') AS bear_case_md,
    COALESCE(content -> $1 ->> 'notes',        content -> 'en' ->> 'notes')        AS notes,
    created_at,
    updated_at
"#;

fn row_to_localized(row: &tokio_postgres::Row) -> LocalizedCatalyst {
    LocalizedCatalyst {
        id: row.get("id"),
        user_id: row.get("user_id"),
        stock_id: row.get("stock_id"),
        sector_code: row.get("sector_code"),
        country: row.get("country"),
        catalyst_kind: row.get("catalyst_kind"),
        catalyst_date: row.get("catalyst_date"),
        date_confidence: row.get("date_confidence"),
        impact_level: row.get("impact_level"),
        status: row.get("status"),
        url: row.get("url"),
        source: row.get("source"),
        title: row.get("title"),
        summary_md: row.get("summary_md"),
        bull_case_md: row.get("bull_case_md"),
        bear_case_md: row.get("bear_case_md"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<LocalizedCatalyst>> {
    let client = db.raw_client().await?;
    let mut sql = format!(
        "SELECT {projection} FROM catalysts WHERE user_id = $2",
        projection = PROJECTION,
    );
    let mut args: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        vec![&filter.locale, &filter.user_id];
    if let Some(s) = filter.stock_id.as_ref() {
        sql.push_str(&format!(" AND stock_id = ${}", args.len() + 1));
        args.push(s);
    }
    let sector_owned;
    if let Some(sc) = filter.sector_code {
        sector_owned = sc.to_string();
        sql.push_str(&format!(" AND sector_code = ${}", args.len() + 1));
        args.push(&sector_owned);
    }
    let country_owned;
    if let Some(c) = filter.country {
        country_owned = c.to_string();
        sql.push_str(&format!(" AND country = ${}", args.len() + 1));
        args.push(&country_owned);
    }
    let kind_owned;
    if let Some(k) = filter.catalyst_kind {
        kind_owned = k.to_string();
        sql.push_str(&format!(" AND catalyst_kind = ${}", args.len() + 1));
        args.push(&kind_owned);
    }
    let status_owned;
    if let Some(st) = filter.status {
        status_owned = st.to_string();
        sql.push_str(&format!(" AND status = ${}", args.len() + 1));
        args.push(&status_owned);
    }
    let impact_owned;
    if let Some(il) = filter.impact_level {
        impact_owned = il.to_string();
        sql.push_str(&format!(" AND impact_level = ${}", args.len() + 1));
        args.push(&impact_owned);
    }
    let from_owned;
    if let Some(f) = filter.from {
        from_owned = f.to_string();
        sql.push_str(&format!(" AND catalyst_date >= ${}", args.len() + 1));
        args.push(&from_owned);
    }
    let to_owned;
    if let Some(t) = filter.to {
        to_owned = t.to_string();
        sql.push_str(&format!(" AND catalyst_date <= ${}", args.len() + 1));
        args.push(&to_owned);
    }
    sql.push_str(" ORDER BY catalyst_date ASC");

    let rows = client.query(&sql, &args).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn list_for_stock(
    db: &Db,
    user_id: i64,
    locale: &str,
    stock_id: i64,
) -> Result<Vec<LocalizedCatalyst>> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM catalysts WHERE user_id = $2 AND stock_id = $3 ORDER BY catalyst_date ASC",
        projection = PROJECTION,
    );
    let rows = client
        .query(&sql, &[&locale, &user_id, &stock_id])
        .await
        .map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn get(db: &Db, user_id: i64, locale: &str, id: i64) -> Result<LocalizedCatalyst> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM catalysts WHERE id = $2 AND user_id = $3",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id, &user_id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_localized).ok_or(DbError::NotFound)
}

pub struct NewCatalyst<'a> {
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<&'a str>,
    pub country: Option<&'a str>,
    pub catalyst_kind: &'a str,
    pub catalyst_date: &'a str,
    pub date_confidence: &'a str,
    pub impact_level: &'a str,
    pub status: &'a str,
    pub url: Option<&'a str>,
    pub source: &'a str,
    pub content: serde_json::Value,
}

/// All-or-nothing batch upsert. Each row goes through the same
/// natural-key conflict as `create`, so an agent re-running a calendar
/// update against the same source refreshes existing rows instead of
/// duplicating. Wrapped in one PG transaction.
pub async fn batch_create(
    db: &Db,
    items: Vec<NewCatalyst<'_>>,
) -> Result<Vec<LocalizedCatalyst>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let mut client = db.raw_client().await?;
    let tx = client.transaction().await.map_err(DbError::from)?;
    let now = jiff::Timestamp::now();
    let sql = r#"
        INSERT INTO catalysts
            (user_id, stock_id, sector_code, country, catalyst_kind, catalyst_date,
             date_confidence, impact_level, status, url, source, content,
             created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
        -- Column-list form (not `ON CONSTRAINT <name>`): the
        -- `catalysts_natural_key` index is declared as a plain UNIQUE INDEX,
        -- not a UNIQUE CONSTRAINT, so postgres only recognizes it via the
        -- column list. Both forms honor NULLS NOT DISTINCT once the right
        -- index is picked.
        ON CONFLICT (user_id, catalyst_kind, catalyst_date, stock_id, sector_code, country, source) DO UPDATE SET
            date_confidence = EXCLUDED.date_confidence,
            impact_level    = EXCLUDED.impact_level,
            status          = EXCLUDED.status,
            url             = EXCLUDED.url,
            content         = EXCLUDED.content,
            updated_at      = EXCLUDED.updated_at
        RETURNING id, user_id
    "#;
    let mut ids: Vec<(i64, i64)> = Vec::with_capacity(items.len());
    for item in &items {
        let row = tx
            .query_one(
                sql,
                &[
                    &item.user_id,
                    &item.stock_id,
                    &item.sector_code,
                    &item.country,
                    &item.catalyst_kind,
                    &item.catalyst_date,
                    &item.date_confidence,
                    &item.impact_level,
                    &item.status,
                    &item.url,
                    &item.source,
                    &item.content,
                    &now,
                ],
            )
            .await
            .map_err(DbError::from)?;
        ids.push((row.get(0), row.get(1)));
    }
    tx.commit().await.map_err(DbError::from)?;

    // Re-fetch each row through the locale-aware `get` so the response
    // shape matches `create` exactly. "en" is the safe default; callers
    // that want a localized projection should call list() afterwards.
    let mut out = Vec::with_capacity(ids.len());
    for (id, user_id) in ids {
        out.push(get(db, user_id, "en", id).await?);
    }
    Ok(out)
}

pub async fn create(db: &Db, input: NewCatalyst<'_>) -> Result<LocalizedCatalyst> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    // Upsert against the `catalysts_natural_key` unique index. Same
    // (user, kind, date, stock, sector, country, source) → update the
    // mutable fields (impact, status, url, content) and bump
    // updated_at. Different source for the same nominal event → fresh
    // row (provenance-discriminated). The natural key uses NULLS NOT
    // DISTINCT so two country-level catalysts (stock_id IS NULL) still
    // collide on the rest of the key.
    let sql = r#"
        INSERT INTO catalysts
            (user_id, stock_id, sector_code, country, catalyst_kind, catalyst_date,
             date_confidence, impact_level, status, url, source, content,
             created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
        -- See batch_create above: the column-list form is required because
        -- `catalysts_natural_key` is a UNIQUE INDEX, not a CONSTRAINT.
        ON CONFLICT (user_id, catalyst_kind, catalyst_date, stock_id, sector_code, country, source) DO UPDATE SET
            date_confidence = EXCLUDED.date_confidence,
            impact_level    = EXCLUDED.impact_level,
            status          = EXCLUDED.status,
            url             = EXCLUDED.url,
            content         = EXCLUDED.content,
            updated_at      = EXCLUDED.updated_at
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.user_id,
                &input.stock_id,
                &input.sector_code,
                &input.country,
                &input.catalyst_kind,
                &input.catalyst_date,
                &input.date_confidence,
                &input.impact_level,
                &input.status,
                &input.url,
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
            "DELETE FROM catalysts WHERE id = $1 AND user_id = $2",
            &[&id, &user_id],
        )
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
