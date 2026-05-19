//! Self-exam queries. Translatable text (`headline`, `content_md`, `notes`)
//! lives in a `content JSONB` column.

use crate::db::{Db, DbError, Result};

#[derive(Debug)]
pub struct LocalizedSelfExam {
    pub id: i64,
    pub user_id: i64,
    pub kind: String,
    pub period_start: String,
    pub period_end: String,
    pub metrics: Option<String>,
    pub recommendation_ids: Option<String>,
    pub source: String,
    pub headline: Option<String>,
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

const PROJECTION: &str = r#"
    id,
    user_id,
    kind,
    period_start,
    period_end,
    metrics,
    recommendation_ids,
    source,
    COALESCE(content -> $1 ->> 'headline',   content -> 'en' ->> 'headline')   AS headline,
    COALESCE(content -> $1 ->> 'content_md', content -> 'en' ->> 'content_md') AS content_md,
    COALESCE(content -> $1 ->> 'notes',      content -> 'en' ->> 'notes')      AS notes,
    created_at,
    updated_at
"#;

fn row_to_localized(row: &tokio_postgres::Row) -> LocalizedSelfExam {
    LocalizedSelfExam {
        id: row.get("id"),
        user_id: row.get("user_id"),
        kind: row.get("kind"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        metrics: row.get("metrics"),
        recommendation_ids: row.get("recommendation_ids"),
        source: row.get("source"),
        headline: row.get("headline"),
        content_md: row.get("content_md"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<LocalizedSelfExam>> {
    let client = db.raw_client().await?;
    let mut sql = format!(
        "SELECT {projection} FROM self_exams WHERE user_id = $2",
        projection = PROJECTION,
    );
    let mut args: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        vec![&filter.locale, &filter.user_id];
    let kind_owned;
    if let Some(k) = filter.kind {
        kind_owned = k.to_string();
        sql.push_str(&format!(" AND kind = ${}", args.len() + 1));
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
    sql.push_str(" ORDER BY period_start DESC");

    let rows = client.query(&sql, &args).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn get(db: &Db, user_id: i64, locale: &str, id: i64) -> Result<LocalizedSelfExam> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM self_exams WHERE id = $2 AND user_id = $3",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id, &user_id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_localized).ok_or(DbError::NotFound)
}

pub struct NewExam<'a> {
    pub user_id: i64,
    pub kind: &'a str,
    pub period_start: &'a str,
    pub period_end: &'a str,
    pub metrics: Option<&'a str>,
    pub recommendation_ids: Option<&'a str>, // JSON array
    pub source: &'a str,
    pub content: serde_json::Value,
}

pub async fn upsert(db: &Db, input: NewExam<'_>) -> Result<LocalizedSelfExam> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    let sql = r#"
        INSERT INTO self_exams
            (user_id, kind, period_start, period_end, metrics,
             recommendation_ids, source, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
        ON CONFLICT (user_id, kind, period_start) DO UPDATE SET
            period_end         = EXCLUDED.period_end,
            metrics            = EXCLUDED.metrics,
            recommendation_ids = EXCLUDED.recommendation_ids,
            source             = EXCLUDED.source,
            content            = EXCLUDED.content,
            updated_at         = EXCLUDED.updated_at
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
                &input.metrics,
                &input.recommendation_ids,
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
            "DELETE FROM self_exams WHERE id = $1 AND user_id = $2",
            &[&id, &user_id],
        )
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
