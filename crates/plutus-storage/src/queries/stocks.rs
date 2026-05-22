//! Stock queries. All translatable text lives in a single `content JSONB`
//! column shaped as
//!   `{ "<locale>": { "name": ..., "description_md": ... } }`
//! Reads pick the right locale at SELECT time via JSON operators (with a
//! fallback to `en`); writes accept the JSON blob directly. There are no
//! per-locale base columns to keep in sync.

use crate::db::{Db, DbError, Result};

/// One stock row with translatable fields already projected for the
/// caller's locale by the storage layer. `name` and `description_md` may
/// be `None` when neither the requested locale nor `en` has the field
/// populated (e.g. legacy rows mid-migration).
#[derive(Debug, Clone)]
pub struct LocalizedStock {
    pub id: i64,
    pub market_code: String,
    pub symbol: String,
    pub isin: Option<String>,
    pub figi: Option<String>,
    pub currency: String,
    pub lot_size: Option<i32>,
    pub asset_class: String,
    pub sector_code: Option<String>,
    pub name: Option<String>,
    pub description_md: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

/// SQL fragment that pulls a translatable field for the requested locale,
/// falling back to `en` if the locale-specific value is missing. `$1` is
/// the requested locale.
const PROJECTION: &str = r#"
    id,
    market_code,
    symbol,
    isin,
    figi,
    currency,
    lot_size,
    asset_class,
    sector_code,
    COALESCE(content -> $1 ->> 'name',           content -> 'en' ->> 'name')           AS name,
    COALESCE(content -> $1 ->> 'description_md', content -> 'en' ->> 'description_md') AS description_md,
    created_at,
    updated_at
"#;

fn row_to_localized(row: &tokio_postgres::Row) -> LocalizedStock {
    LocalizedStock {
        id: row.get("id"),
        market_code: row.get("market_code"),
        symbol: row.get("symbol"),
        isin: row.get("isin"),
        figi: row.get("figi"),
        currency: row.get("currency"),
        lot_size: row.get("lot_size"),
        asset_class: row.get("asset_class"),
        sector_code: row.get("sector_code"),
        name: row.get("name"),
        description_md: row.get("description_md"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// Filter for `list`. Every field is optional; `None` = no filter.
///
/// `symbol` is case-insensitive equality on the ticker. Returns 0 or 1 row.
/// `q` is a case-insensitive substring on the ticker AND the localized
/// `name` from the `content` JSONB (with fallback to `en`). Designed for
/// "user types `appl`, gets AAPL via symbol OR Apple Inc via name".
/// Both compose with each other and with caller-side `country` filtering
/// (which lives in the API handler — the DB layer doesn't know about
/// country → MIC mapping).
///
/// `limit` caps the result count. Callers should pass a sane value; the
/// handler enforces an upper bound so a runaway `q=a` doesn't dump every
/// ticker. `None` means "no DB-level cap" — handler still applies one.
pub struct ListFilter<'a> {
    pub symbol: Option<&'a str>,
    pub q: Option<&'a str>,
    /// Optional precise-fetch list. When set, returns exactly the rows
    /// whose id is in this slice; `limit` is ignored. Used by consumer
    /// pages (holdings / watchlists / orders / transactions) that
    /// already know which stocks they need to display and would
    /// otherwise be capped by the global LIMIT.
    pub ids: Option<&'a [i64]>,
    pub limit: Option<i64>,
}

pub async fn list(
    db: &Db,
    locale: &str,
    filter: ListFilter<'_>,
) -> Result<Vec<LocalizedStock>> {
    let client = db.raw_client().await?;

    // Build WHERE clause incrementally; param positions stay stable by
    // collecting borrowed-string holders for the optional values.
    let mut wheres: Vec<String> = Vec::new();
    // $1 is always the locale (used by the PROJECTION fallback). Filter
    // params start at $2.
    let mut next_pos = 2usize;
    let symbol_owned;
    let q_pattern_owned;
    let limit_owned;
    // Hoist the id slice to function scope so `params.push(&ids_owned)`
    // inside the if-let below has a borrow that outlives `params` itself.
    let ids_owned: &[i64] = filter.ids.unwrap_or(&[]);
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&locale];

    if let Some(sym) = filter.symbol {
        symbol_owned = sym.to_string();
        params.push(&symbol_owned);
        wheres.push(format!("UPPER(symbol) = UPPER(${next_pos})"));
        next_pos += 1;
    }
    if let Some(q) = filter.q {
        // ILIKE pattern: `%foo%`. Search symbol OR localized name (with
        // `en` fallback) so `?q=apple` hits AAPL via the name match.
        q_pattern_owned = format!("%{}%", q);
        params.push(&q_pattern_owned);
        wheres.push(format!(
            "(symbol ILIKE ${pos} OR COALESCE(content -> $1 ->> 'name', content -> 'en' ->> 'name') ILIKE ${pos})",
            pos = next_pos,
        ));
        next_pos += 1;
    }
    if filter.ids.is_some() {
        // `id = ANY($N)` with a BIGINT[] parameter. Empty slice still
        // produces a valid (always-false) clause so callers don't need
        // to special-case `&[]`.
        params.push(&ids_owned);
        wheres.push(format!("id = ANY(${next_pos})"));
        next_pos += 1;
    }
    let where_clause = if wheres.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", wheres.join(" AND "))
    };

    // When the caller supplied an explicit id list, ignore the limit:
    // the result set is already bounded by the ids vector. Without
    // this, a holdings page with >limit positions would silently lose
    // rows past the cap.
    let limit_clause = if filter.ids.is_some() {
        String::new()
    } else if let Some(n) = filter.limit {
        limit_owned = n;
        params.push(&limit_owned);
        format!(" LIMIT ${next_pos}")
    } else {
        String::new()
    };

    let sql = format!(
        "SELECT {projection} FROM stocks{where_clause} ORDER BY market_code ASC, symbol ASC{limit_clause}",
        projection = PROJECTION,
    );
    let rows = client.query(&sql, &params[..]).await.map_err(DbError::from)?;
    Ok(rows.iter().map(row_to_localized).collect())
}

pub async fn get(db: &Db, locale: &str, id: i64) -> Result<LocalizedStock> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM stocks WHERE id = $2",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &id])
        .await
        .map_err(DbError::from)?;
    row.as_ref().map(row_to_localized).ok_or(DbError::NotFound)
}

pub async fn find_by_market_symbol(
    db: &Db,
    locale: &str,
    market_code: &str,
    symbol: &str,
) -> Result<Option<LocalizedStock>> {
    let client = db.raw_client().await?;
    let sql = format!(
        "SELECT {projection} FROM stocks WHERE market_code = $2 AND symbol = $3",
        projection = PROJECTION,
    );
    let row = client
        .query_opt(&sql, &[&locale, &market_code, &symbol])
        .await
        .map_err(DbError::from)?;
    Ok(row.as_ref().map(row_to_localized))
}

pub struct NewStock<'a> {
    pub market_code: &'a str,
    pub symbol: &'a str,
    pub isin: Option<&'a str>,
    pub figi: Option<&'a str>,
    pub currency: &'a str,
    pub lot_size: Option<i32>,
    pub asset_class: &'a str,
    pub sector_code: Option<&'a str>,
    /// Full multi-locale content blob. Shape is
    /// `{ "<locale>": { "name": ..., "description_md": ... } }`. Callers
    /// can omit fields per locale; the SELECT-time COALESCE handles
    /// fallbacks to `en`.
    pub content: serde_json::Value,
}

pub async fn create(db: &Db, input: NewStock<'_>) -> Result<LocalizedStock> {
    if find_by_market_symbol(db, "en", input.market_code, input.symbol)
        .await?
        .is_some()
    {
        return Err(DbError::Conflict(format!(
            "stock {}:{} already exists",
            input.market_code, input.symbol
        )));
    }
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let content = &input.content;
    let sql = r#"
        INSERT INTO stocks
            (market_code, symbol, isin, figi, currency, lot_size, asset_class,
             sector_code, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
        RETURNING id
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.market_code,
                &input.symbol,
                &input.isin,
                &input.figi,
                &input.currency,
                &input.lot_size,
                &input.asset_class,
                &input.sector_code,
                &content,
                &now,
            ],
        )
        .await
        .map_err(DbError::from)?;
    let id: i64 = row.get(0);
    get(db, "en", id).await
}

/// Patch the `content` JSONB blob in-place (full replacement). Returns the
/// updated localized row read back through the locale projection.
pub async fn update_content(
    db: &Db,
    locale: &str,
    id: i64,
    content: &serde_json::Value,
) -> Result<LocalizedStock> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let affected = client
        .execute(
            "UPDATE stocks SET content = $1, updated_at = $2 WHERE id = $3",
            &[content, &now, &id],
        )
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    get(db, locale, id).await
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let client = db.raw_client().await?;
    let affected = client
        .execute("DELETE FROM stocks WHERE id = $1", &[&id])
        .await
        .map_err(DbError::from)?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
