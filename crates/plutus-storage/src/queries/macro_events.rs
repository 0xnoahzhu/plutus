//! Macro event queries. Shared table — no per-user filtering. Translatable
//! text (`title`, `summary_md`) lives in a `content JSONB` column.

use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};

#[derive(Debug)]
pub struct LocalizedMacroEvent {
    pub id: i64,
    pub indicator_code: String,
    pub event_date: String,
    pub event_kind: String,
    pub decision: Option<String>,
    pub decision_bps: Option<i32>,
    pub new_value: Option<Decimal>,
    pub consensus_estimate: Option<Decimal>,
    pub surprise: Option<Decimal>,
    pub previous_value: Option<Decimal>,
    pub vote: Option<String>,
    pub dot_plot: Option<String>,
    pub url: Option<String>,
    pub source: String,
    pub title: Option<String>,
    pub summary_md: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

pub struct ListFilter<'a> {
    pub locale: &'a str,
    pub indicator_code: Option<&'a str>,
    pub event_kind: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

const PROJECTION: &str = r#"
    id,
    indicator_code,
    event_date,
    event_kind,
    decision,
    decision_bps,
    new_value,
    consensus_estimate,
    surprise,
    previous_value,
    vote,
    dot_plot,
    url,
    source,
    COALESCE(content -> $1 ->> 'title',      content -> 'en' ->> 'title')      AS title,
    COALESCE(content -> $1 ->> 'summary_md', content -> 'en' ->> 'summary_md') AS summary_md,
    created_at,
    updated_at
"#;

fn row_to_localized(row: &tokio_postgres::Row) -> LocalizedMacroEvent {
    LocalizedMacroEvent {
        id: row.get("id"),
        indicator_code: row.get("indicator_code"),
        event_date: row.get("event_date"),
        event_kind: row.get("event_kind"),
        decision: row.get("decision"),
        decision_bps: row.get("decision_bps"),
        new_value: row.get("new_value"),
        consensus_estimate: row.get("consensus_estimate"),
        surprise: row.get("surprise"),
        previous_value: row.get("previous_value"),
        vote: row.get("vote"),
        dot_plot: row.get("dot_plot"),
        url: row.get("url"),
        source: row.get("source"),
        title: row.get("title"),
        summary_md: row.get("summary_md"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<LocalizedMacroEvent>> {
    let client = db.raw_client().await?;
    let mut sql = format!(
        "SELECT {projection} FROM macro_events WHERE 1=1",
        projection = PROJECTION,
    );
    let mut args: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&filter.locale];
    let indicator_owned;
    if let Some(i) = filter.indicator_code {
        indicator_owned = i.to_string();
        sql.push_str(&format!(" AND indicator_code = ${}", args.len() + 1));
        args.push(&indicator_owned);
    }
    let kind_owned;
    if let Some(k) = filter.event_kind {
        kind_owned = k.to_string();
        sql.push_str(&format!(" AND event_kind = ${}", args.len() + 1));
        args.push(&kind_owned);
    }
    let from_owned;
    if let Some(f) = filter.from {
        from_owned = f.to_string();
        sql.push_str(&format!(" AND event_date >= ${}", args.len() + 1));
        args.push(&from_owned);
    }
    let to_owned;
    if let Some(t) = filter.to {
        to_owned = t.to_string();
        sql.push_str(&format!(" AND event_date <= ${}", args.len() + 1));
        args.push(&to_owned);
    }
    sql.push_str(" ORDER BY event_date ASC, indicator_code ASC");

    let rows = client.query(&sql, &args).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn get(db: &Db, locale: &str, id: i64) -> Result<LocalizedMacroEvent> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM macro_events WHERE id = $2",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_localized).ok_or(DbError::NotFound)
}

pub struct NewMacroEvent<'a> {
    pub indicator_code: &'a str,
    pub event_date: &'a str,
    pub event_kind: &'a str,
    pub decision: Option<&'a str>,
    pub decision_bps: Option<i32>,
    pub new_value: Option<Decimal>,
    pub consensus_estimate: Option<Decimal>,
    pub surprise: Option<Decimal>,
    pub previous_value: Option<Decimal>,
    pub vote: Option<&'a str>,
    pub dot_plot: Option<&'a str>,
    pub url: Option<&'a str>,
    pub source: &'a str,
    pub content: serde_json::Value,
}

/// Upsert by (indicator_code, event_date). Re-POST as scheduled → released
/// → revised arrives.
pub async fn upsert(db: &Db, input: NewMacroEvent<'_>) -> Result<LocalizedMacroEvent> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    // If agent didn't compute the surprise, do it here.
    let surprise = input.surprise.or_else(|| match (input.new_value, input.consensus_estimate) {
        (Some(a), Some(c)) => Some(a - c),
        _ => None,
    });
    let sql = r#"
        INSERT INTO macro_events
            (indicator_code, event_date, event_kind, decision, decision_bps,
             new_value, consensus_estimate, surprise, previous_value, vote,
             dot_plot, url, source, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $15)
        ON CONFLICT (indicator_code, event_date) DO UPDATE SET
            event_kind         = EXCLUDED.event_kind,
            decision           = EXCLUDED.decision,
            decision_bps       = EXCLUDED.decision_bps,
            new_value          = EXCLUDED.new_value,
            consensus_estimate = EXCLUDED.consensus_estimate,
            surprise           = EXCLUDED.surprise,
            previous_value     = EXCLUDED.previous_value,
            vote               = EXCLUDED.vote,
            dot_plot           = EXCLUDED.dot_plot,
            url                = EXCLUDED.url,
            source             = EXCLUDED.source,
            content            = EXCLUDED.content,
            updated_at         = EXCLUDED.updated_at
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.indicator_code,
                &input.event_date,
                &input.event_kind,
                &input.decision,
                &input.decision_bps,
                &input.new_value,
                &input.consensus_estimate,
                &surprise,
                &input.previous_value,
                &input.vote,
                &input.dot_plot,
                &input.url,
                &input.source,
                &content,
                &now,
            ],
        )
        .await
        .map_err(DbError::from)?;
    let id: i64 = row.get(0);
    get(db, "en", id).await
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let client = db.raw_client().await?;
    let affected = client
        .execute("DELETE FROM macro_events WHERE id = $1", &[&id])
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
