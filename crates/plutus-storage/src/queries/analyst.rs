use rust_decimal::Decimal;

use crate::db::{Db, Result};
use crate::models::{AnalystEstimate, AnalystRating};

// ── Estimates ────────────────────────────────────────────────────────────

pub async fn list_estimates_for_stock(db: &Db, stock_id: i64) -> Result<Vec<AnalystEstimate>> {
    db.with(async |d| {
        AnalystEstimate::all()
            .filter(AnalystEstimate::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
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

pub async fn list_ratings_for_stock(db: &Db, stock_id: i64) -> Result<Vec<AnalystRating>> {
    db.with(async |d| {
        AnalystRating::all()
            .filter(AnalystRating::fields().stock_id().eq(stock_id))
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
