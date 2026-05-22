use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::{AnalystEstimate, AnalystRating};

// ── Estimates ────────────────────────────────────────────────────────────

pub async fn list_estimates_for_stock(
    db: &Db,
    stock_id: i64,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<AnalystEstimate>> {
    let l = limit.unwrap_or(i32::MAX as usize);
    let o = offset.unwrap_or(0);
    db.with(async |d| {
        AnalystEstimate::all()
            .filter(AnalystEstimate::fields().stock_id().eq(stock_id))
            .order_by((
                AnalystEstimate::fields().as_of_date().desc(),
                AnalystEstimate::fields().id().desc(),
            ))
            .limit(l)
            .offset(o)
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn get_estimate(db: &Db, id: i64) -> Result<AnalystEstimate> {
    db.with(async |d| AnalystEstimate::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub async fn delete_estimate(db: &Db, id: i64) -> Result<()> {
    let row = get_estimate(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}

pub async fn count_estimates_for_stock(db: &Db, stock_id: i64) -> Result<i64> {
    let client = db.raw_client().await?;
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM analyst_estimates WHERE stock_id = $1",
            &[&stock_id],
        )
        .await
        .map_err(DbError::from)?;
    Ok(row.get::<_, i64>(0))
}

pub struct NewEstimate<'a> {
    pub stock_id: i64,
    pub metric: &'a str,
    pub fiscal_year: i32,
    pub fiscal_period: &'a str,
    pub consensus_mean: Option<Decimal>,
    pub consensus_median: Option<Decimal>,
    pub estimate_low: Option<Decimal>,
    pub estimate_high: Option<Decimal>,
    pub num_analysts: Option<i32>,
    pub as_of_date: &'a str,
    pub source: &'a str,
}

pub async fn insert_estimate(db: &Db, input: NewEstimate<'_>) -> Result<AnalystEstimate> {
    let stock_id = input.stock_id;
    let metric = input.metric.to_string();
    let fiscal_year = input.fiscal_year;
    let fiscal_period = input.fiscal_period.to_string();
    let consensus_mean = input.consensus_mean;
    let consensus_median = input.consensus_median;
    let estimate_low = input.estimate_low;
    let estimate_high = input.estimate_high;
    let num_analysts = input.num_analysts;
    let as_of_date = input.as_of_date.to_string();
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();

    let row = db
        .with(async |d| {
            toasty::create!(AnalystEstimate {
                stock_id: stock_id,
                metric: metric,
                fiscal_year: fiscal_year,
                fiscal_period: fiscal_period,
                consensus_mean: consensus_mean,
                consensus_median: consensus_median,
                estimate_low: estimate_low,
                estimate_high: estimate_high,
                num_analysts: num_analysts,
                as_of_date: as_of_date,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

// ── Ratings ──────────────────────────────────────────────────────────────

pub async fn list_ratings_for_stock(
    db: &Db,
    stock_id: i64,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<AnalystRating>> {
    let l = limit.unwrap_or(i32::MAX as usize);
    let o = offset.unwrap_or(0);
    db.with(async |d| {
        AnalystRating::all()
            .filter(AnalystRating::fields().stock_id().eq(stock_id))
            .order_by((
                AnalystRating::fields().rated_at().desc(),
                AnalystRating::fields().id().desc(),
            ))
            .limit(l)
            .offset(o)
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub struct NewRating<'a> {
    pub stock_id: i64,
    pub firm: &'a str,
    pub analyst_name: Option<&'a str>,
    pub rating: &'a str,
    pub rating_action: &'a str,
    pub previous_rating: Option<&'a str>,
    pub target_price: Option<Decimal>,
    pub target_currency: Option<&'a str>,
    pub previous_target: Option<Decimal>,
    pub rated_at: &'a str,
    pub source: &'a str,
}

pub async fn insert_rating(db: &Db, input: NewRating<'_>) -> Result<AnalystRating> {
    let stock_id = input.stock_id;
    let firm = input.firm.to_string();
    let analyst_name = input.analyst_name.map(str::to_string);
    let rating = input.rating.to_string();
    let rating_action = input.rating_action.to_string();
    let previous_rating = input.previous_rating.map(str::to_string);
    let target_price = input.target_price;
    let target_currency = input.target_currency.map(str::to_string);
    let previous_target = input.previous_target;
    let rated_at = input.rated_at.to_string();
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();

    let row = db
        .with(async |d| {
            toasty::create!(AnalystRating {
                stock_id: stock_id,
                firm: firm,
                analyst_name: analyst_name,
                rating: rating,
                rating_action: rating_action,
                previous_rating: previous_rating,
                target_price: target_price,
                target_currency: target_currency,
                previous_target: previous_target,
                rated_at: rated_at,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn get_rating(db: &Db, id: i64) -> Result<AnalystRating> {
    db.with(async |d| AnalystRating::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub async fn delete_rating(db: &Db, id: i64) -> Result<()> {
    let row = get_rating(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}

pub async fn count_ratings_for_stock(db: &Db, stock_id: i64) -> Result<i64> {
    let client = db.raw_client().await?;
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM analyst_ratings WHERE stock_id = $1",
            &[&stock_id],
        )
        .await
        .map_err(DbError::from)?;
    Ok(row.get::<_, i64>(0))
}

/// All-or-nothing batch insert of N estimates. Wrapped in a single
/// Postgres transaction so a single bad row rolls everything back.
/// Used by the agent's nightly consensus refresh sweeps — re-running
/// `insert_estimate` per row would be N round trips and N implicit
/// transactions.
pub async fn batch_insert_estimates(
    db: &Db,
    items: &[NewEstimate<'_>],
) -> Result<Vec<AnalystEstimate>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let mut client = db.raw_client().await?;
    let tx = client.transaction().await.map_err(DbError::from)?;
    let mut out = Vec::with_capacity(items.len());
    let now = jiff::Timestamp::now();
    let sql = r#"
        INSERT INTO analyst_estimates
            (stock_id, metric, fiscal_year, fiscal_period,
             consensus_mean, consensus_median, estimate_low, estimate_high,
             num_analysts, as_of_date, source, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, stock_id, metric, fiscal_year, fiscal_period,
                  consensus_mean, consensus_median, estimate_low, estimate_high,
                  num_analysts, as_of_date, source, created_at
    "#;
    for item in items {
        let metric_owned = item.metric.to_string();
        let fiscal_period_owned = item.fiscal_period.to_string();
        let as_of_owned = item.as_of_date.to_string();
        let source_owned = item.source.to_string();
        let row = tx
            .query_one(
                sql,
                &[
                    &item.stock_id,
                    &metric_owned,
                    &item.fiscal_year,
                    &fiscal_period_owned,
                    &item.consensus_mean,
                    &item.consensus_median,
                    &item.estimate_low,
                    &item.estimate_high,
                    &item.num_analysts,
                    &as_of_owned,
                    &source_owned,
                    &now,
                ],
            )
            .await
            .map_err(DbError::from)?;
        out.push(AnalystEstimate {
            id: row.get("id"),
            stock_id: row.get("stock_id"),
            metric: row.get("metric"),
            fiscal_year: row.get("fiscal_year"),
            fiscal_period: row.get("fiscal_period"),
            consensus_mean: row.get("consensus_mean"),
            consensus_median: row.get("consensus_median"),
            estimate_low: row.get("estimate_low"),
            estimate_high: row.get("estimate_high"),
            num_analysts: row.get("num_analysts"),
            as_of_date: row.get("as_of_date"),
            source: row.get("source"),
            created_at: row.get("created_at"),
        });
    }
    tx.commit().await.map_err(DbError::from)?;
    Ok(out)
}

/// All-or-nothing batch insert of N rating actions.
pub async fn batch_insert_ratings(
    db: &Db,
    items: &[NewRating<'_>],
) -> Result<Vec<AnalystRating>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let mut client = db.raw_client().await?;
    let tx = client.transaction().await.map_err(DbError::from)?;
    let mut out = Vec::with_capacity(items.len());
    let now = jiff::Timestamp::now();
    let sql = r#"
        INSERT INTO analyst_ratings
            (stock_id, firm, analyst_name, rating, rating_action,
             previous_rating, target_price, target_currency,
             previous_target, rated_at, source, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, stock_id, firm, analyst_name, rating, rating_action,
                  previous_rating, target_price, target_currency,
                  previous_target, rated_at, source, created_at
    "#;
    for item in items {
        let firm_owned = item.firm.to_string();
        let analyst_owned = item.analyst_name.map(str::to_string);
        let rating_owned = item.rating.to_string();
        let action_owned = item.rating_action.to_string();
        let prev_rating_owned = item.previous_rating.map(str::to_string);
        let target_ccy_owned = item.target_currency.map(str::to_string);
        let rated_at_owned = item.rated_at.to_string();
        let source_owned = item.source.to_string();
        let row = tx
            .query_one(
                sql,
                &[
                    &item.stock_id,
                    &firm_owned,
                    &analyst_owned,
                    &rating_owned,
                    &action_owned,
                    &prev_rating_owned,
                    &item.target_price,
                    &target_ccy_owned,
                    &item.previous_target,
                    &rated_at_owned,
                    &source_owned,
                    &now,
                ],
            )
            .await
            .map_err(DbError::from)?;
        out.push(AnalystRating {
            id: row.get("id"),
            stock_id: row.get("stock_id"),
            firm: row.get("firm"),
            analyst_name: row.get("analyst_name"),
            rating: row.get("rating"),
            rating_action: row.get("rating_action"),
            previous_rating: row.get("previous_rating"),
            target_price: row.get("target_price"),
            target_currency: row.get("target_currency"),
            previous_target: row.get("previous_target"),
            rated_at: row.get("rated_at"),
            source: row.get("source"),
            created_at: row.get("created_at"),
        });
    }
    tx.commit().await.map_err(DbError::from)?;
    Ok(out)
}
