use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::{MacroIndicator, MacroObservation};

// ── Indicators ────────────────────────────────────────────────────────────

pub async fn list_indicators(db: &Db) -> Result<Vec<MacroIndicator>> {
    db.with(async |d| MacroIndicator::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get_indicator(db: &Db, code: &str) -> Result<Option<MacroIndicator>> {
    let code = code.to_string();
    db.with(async |d| MacroIndicator::filter_by_code(code).first().exec(d).await)
        .await
        .map_err(Into::into)
}

pub struct NewIndicator<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub country: &'a str,
    pub unit: &'a str,
    pub frequency: &'a str,
    pub source: &'a str,
    pub description: Option<&'a str>,
}

pub async fn upsert_indicator(db: &Db, input: NewIndicator<'_>) -> Result<MacroIndicator> {
    let code_owned = input.code.to_string();
    if let Some(existing) = db
        .with(async |d| MacroIndicator::filter_by_code(code_owned).first().exec(d).await)
        .await?
    {
        let name = input.name.to_string();
        let country = input.country.to_string();
        let unit = input.unit.to_string();
        let frequency = input.frequency.to_string();
        let source = input.source.to_string();
        let description = input.description.map(str::to_string);
        db.with(async |d| {
            let mut e = existing;
            e.update()
                .name(name)
                .country(country)
                .unit(unit)
                .frequency(frequency)
                .source(source)
                .description(description)
                .exec(d)
                .await
        })
        .await?;
    } else {
        let code = input.code.to_string();
        let name = input.name.to_string();
        let country = input.country.to_string();
        let unit = input.unit.to_string();
        let frequency = input.frequency.to_string();
        let source = input.source.to_string();
        let description = input.description.map(str::to_string);
        db.with(async |d| {
            toasty::create!(MacroIndicator {
                code: code,
                name: name,
                country: country,
                unit: unit,
                frequency: frequency,
                source: source,
                description: description,
            })
            .exec(d)
            .await
        })
        .await?;
    }
    let code = input.code.to_string();
    db.with(async |d| MacroIndicator::filter_by_code(code).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

// ── Observations ─────────────────────────────────────────────────────────

pub async fn list_observations(db: &Db, indicator_code: &str) -> Result<Vec<MacroObservation>> {
    let code = indicator_code.to_string();
    db.with(async |d| {
        MacroObservation::all()
            .filter(MacroObservation::fields().indicator_code().eq(&code))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub struct NewObservation<'a> {
    pub indicator_code: &'a str,
    pub obs_date: &'a str,
    pub value: Decimal,
    pub source: &'a str,
}

/// Upsert by (indicator_code, obs_date). A re-POST for the same code
/// and date is a revision: `value` and `source` are refreshed, and
/// `revised_at` is stamped with the current time so the row carries
/// both its original publish time (`created_at`) and the latest
/// revision time. First-time inserts leave `revised_at` NULL.
pub async fn upsert_observation(db: &Db, input: NewObservation<'_>) -> Result<MacroObservation> {
    let client = db.raw_client().await?;
    let now = jiff::Timestamp::now();
    let sql = r#"
        INSERT INTO macro_observations
            (indicator_code, obs_date, value, revised_at, source, created_at)
        VALUES ($1, $2, $3, NULL, $4, $5)
        ON CONFLICT (indicator_code, obs_date) DO UPDATE SET
            value      = EXCLUDED.value,
            source     = EXCLUDED.source,
            revised_at = EXCLUDED.created_at
        RETURNING id, indicator_code, obs_date, value, revised_at, source, created_at
    "#;
    let row = client
        .query_one(
            sql,
            &[
                &input.indicator_code,
                &input.obs_date,
                &input.value,
                &input.source,
                &now,
            ],
        )
        .await
        .map_err(DbError::from)?;
    Ok(MacroObservation {
        id: row.get(0),
        indicator_code: row.get(1),
        obs_date: row.get(2),
        value: row.get(3),
        revised_at: row.get(4),
        source: row.get(5),
        created_at: row.get(6),
    })
}
