//! Recommendation queries. Translatable text (`rationale_md`, `outcome_md`)
//! lives in a `content JSONB` column. Reads project for the requested locale
//! with fallback to `en`.

use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};

#[derive(Debug)]
pub struct LocalizedRecommendation {
    pub id: i64,
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<String>,
    pub action: String,
    pub confidence: Option<Decimal>,
    pub target_price: Option<Decimal>,
    pub target_currency: Option<String>,
    pub target_horizon: String,
    pub issued_at: jiff::Timestamp,
    pub status: String,
    pub pnl_pct: Option<Decimal>,
    pub closed_at: Option<jiff::Timestamp>,
    pub source: String,
    pub rationale_md: Option<String>,
    pub outcome_md: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub locale: &'a str,
    pub stock_id: Option<i64>,
    pub status: Option<&'a str>,
    pub from: Option<&'a str>, // YYYY-MM-DD on issued_at
    pub to: Option<&'a str>,
}

const PROJECTION: &str = r#"
    id,
    user_id,
    stock_id,
    sector_code,
    action,
    confidence,
    target_price,
    target_currency,
    target_horizon,
    issued_at,
    status,
    pnl_pct,
    closed_at,
    source,
    COALESCE(content -> $1 ->> 'rationale_md', content -> 'en' ->> 'rationale_md') AS rationale_md,
    COALESCE(content -> $1 ->> 'outcome_md',   content -> 'en' ->> 'outcome_md')   AS outcome_md,
    created_at,
    updated_at
"#;

fn row_to_localized(row: &tokio_postgres::Row) -> LocalizedRecommendation {
    LocalizedRecommendation {
        id: row.get("id"),
        user_id: row.get("user_id"),
        stock_id: row.get("stock_id"),
        sector_code: row.get("sector_code"),
        action: row.get("action"),
        confidence: row.get("confidence"),
        target_price: row.get("target_price"),
        target_currency: row.get("target_currency"),
        target_horizon: row.get("target_horizon"),
        issued_at: row.get("issued_at"),
        status: row.get("status"),
        pnl_pct: row.get("pnl_pct"),
        closed_at: row.get("closed_at"),
        source: row.get("source"),
        rationale_md: row.get("rationale_md"),
        outcome_md: row.get("outcome_md"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<LocalizedRecommendation>> {
    let client = db.raw_client().await?;
    let mut sql = format!(
        "SELECT {projection} FROM recommendations WHERE user_id = $2",
        projection = PROJECTION,
    );
    let mut args: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        vec![&filter.locale, &filter.user_id];
    if let Some(s) = filter.stock_id.as_ref() {
        sql.push_str(&format!(" AND stock_id = ${}", args.len() + 1));
        args.push(s);
    }
    let status_owned;
    if let Some(st) = filter.status {
        status_owned = st.to_string();
        sql.push_str(&format!(" AND status = ${}", args.len() + 1));
        args.push(&status_owned);
    }
    // Note: from/to filters on issued_at compare against the timestamp's text
    // representation, matching the previous in-memory behavior. Callers pass
    // ISO date prefixes (YYYY-MM-DD) and we lean on the lexicographic order
    // of RFC3339 timestamps.
    let from_owned;
    if let Some(f) = filter.from {
        from_owned = f.to_string();
        sql.push_str(&format!(" AND to_char(issued_at, 'YYYY-MM-DD') >= ${}", args.len() + 1));
        args.push(&from_owned);
    }
    let to_owned;
    if let Some(t) = filter.to {
        to_owned = t.to_string();
        sql.push_str(&format!(" AND to_char(issued_at, 'YYYY-MM-DD') <= ${}", args.len() + 1));
        args.push(&to_owned);
    }
    sql.push_str(" ORDER BY issued_at DESC");

    let rows = client.query(&sql, &args).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn get(
    db: &Db,
    user_id: i64,
    locale: &str,
    id: i64,
) -> Result<LocalizedRecommendation> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM recommendations WHERE id = $2 AND user_id = $3",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id, &user_id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_localized).ok_or(DbError::NotFound)
}

pub struct NewRecommendation<'a> {
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<&'a str>,
    pub action: &'a str,
    pub confidence: Option<Decimal>,
    pub target_price: Option<Decimal>,
    pub target_currency: Option<&'a str>,
    pub target_horizon: &'a str,
    pub issued_at: jiff::Timestamp,
    pub source: &'a str,
    pub content: serde_json::Value,
}

pub async fn create(db: &Db, input: NewRecommendation<'_>) -> Result<LocalizedRecommendation> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    let status = "open";
    let sql = r#"
        INSERT INTO recommendations
            (user_id, stock_id, sector_code, action, confidence, target_price,
             target_currency, target_horizon, issued_at, status, source, content,
             created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.user_id,
                &input.stock_id,
                &input.sector_code,
                &input.action,
                &input.confidence,
                &input.target_price,
                &input.target_currency,
                &input.target_horizon,
                &input.issued_at,
                &status,
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

pub struct ClosePatch<'a> {
    pub status: &'a str, // "closed_correct" / "closed_wrong" / "closed_neutral" / "expired"
    pub outcome_md: Option<&'a str>,
    pub pnl_pct: Option<Decimal>,
    pub closed_at: jiff::Timestamp,
}

/// Close out an open recommendation. Note: `outcome_md` is merged into the
/// existing `content` JSONB at the `en` locale (or replaces it there). For
/// multi-locale outcome text, callers should re-POST a full `content` blob
/// via a future update endpoint; this fast-path only handles the common
/// single-locale close.
pub async fn close(
    db: &Db,
    user_id: i64,
    id: i64,
    patch: ClosePatch<'_>,
) -> Result<LocalizedRecommendation> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let affected = if let Some(s) = patch.outcome_md {
        // Merge `outcome_md` into the existing `content` JSONB at the `en`
        // locale. For multi-locale outcome text, callers should send a full
        // upsert via a separate endpoint; this fast-path covers the common
        // single-locale close.
        let outcome_value = serde_json::json!({ "en": { "outcome_md": s } });
        client
            .execute(
                r#"
                    UPDATE recommendations
                    SET status     = $1,
                        pnl_pct    = $2,
                        closed_at  = $3,
                        updated_at = $4,
                        content    = content || $5::jsonb
                    WHERE id = $6 AND user_id = $7
                "#,
                &[
                    &patch.status,
                    &patch.pnl_pct,
                    &patch.closed_at,
                    &now,
                    &outcome_value,
                    &id,
                    &user_id,
                ],
            )
            .await
            .map_err(DbError::from)?
    } else {
        client
            .execute(
                r#"
                    UPDATE recommendations
                    SET status     = $1,
                        pnl_pct    = $2,
                        closed_at  = $3,
                        updated_at = $4
                    WHERE id = $5 AND user_id = $6
                "#,
                &[
                    &patch.status,
                    &patch.pnl_pct,
                    &patch.closed_at,
                    &now,
                    &id,
                    &user_id,
                ],
            )
            .await
            .map_err(DbError::from)?
    };
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    get(db, user_id, "en", id).await
}

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let client = db.raw_client().await?;
    let affected = client
        .execute(
            "DELETE FROM recommendations WHERE id = $1 AND user_id = $2",
            &[&id, &user_id],
        )
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
