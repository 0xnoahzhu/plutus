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

pub async fn insert_observation(db: &Db, input: NewObservation<'_>) -> Result<MacroObservation> {
    let indicator_code = input.indicator_code.to_string();
    let obs_date = input.obs_date.to_string();
    let value = input.value;
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(MacroObservation {
                indicator_code: indicator_code,
                obs_date: obs_date,
                value: value,
                revised_at: None::<jiff::Timestamp>,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
