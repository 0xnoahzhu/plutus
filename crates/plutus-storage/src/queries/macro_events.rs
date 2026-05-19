use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::MacroEvent;

pub struct ListFilter<'a> {
    pub indicator_code: Option<&'a str>,
    pub event_kind: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<MacroEvent>> {
    let rows = match (filter.indicator_code, filter.event_kind) {
        (Some(i), Some(k)) => {
            let i_owned = i.to_string();
            let k_owned = k.to_string();
            db.with(async |d| {
                MacroEvent::all()
                    .filter(MacroEvent::fields().indicator_code().eq(&i_owned))
                    .filter(MacroEvent::fields().event_kind().eq(&k_owned))
                    .exec(d)
                    .await
            })
            .await?
        }
        (Some(i), None) => {
            let i_owned = i.to_string();
            db.with(async |d| {
                MacroEvent::all()
                    .filter(MacroEvent::fields().indicator_code().eq(&i_owned))
                    .exec(d)
                    .await
            })
            .await?
        }
        (None, Some(k)) => {
            let k_owned = k.to_string();
            db.with(async |d| {
                MacroEvent::all()
                    .filter(MacroEvent::fields().event_kind().eq(&k_owned))
                    .exec(d)
                    .await
            })
            .await?
        }
        (None, None) => db.with(async |d| MacroEvent::all().exec(d).await).await?,
    };
    let from = filter.from.map(str::to_string);
    let to = filter.to.map(str::to_string);
    Ok(rows
        .into_iter()
        .filter(|r| from.as_deref().map_or(true, |f| r.event_date.as_str() >= f))
        .filter(|r| to.as_deref().map_or(true, |t| r.event_date.as_str() <= t))
        .collect())
}

pub async fn get(db: &Db, id: i64) -> Result<MacroEvent> {
    db.with(async |d| MacroEvent::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub struct NewMacroEvent<'a> {
    pub indicator_code: &'a str,
    pub event_date: &'a str,
    pub event_kind: &'a str,
    pub title: &'a str,
    pub summary_md: Option<&'a str>,
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
    pub translations: Option<&'a str>,
}

/// Upsert by (indicator_code, event_date). Re-POST as scheduled → released
/// → revised arrives.
pub async fn upsert(db: &Db, input: NewMacroEvent<'_>) -> Result<MacroEvent> {
    let indicator_owned = input.indicator_code.to_string();
    let date_owned = input.event_date.to_string();
    let existing = db
        .with(async |d| {
            MacroEvent::all()
                .filter(MacroEvent::fields().indicator_code().eq(&indicator_owned))
                .filter(MacroEvent::fields().event_date().eq(&date_owned))
                .first()
                .exec(d)
                .await
        })
        .await?;

    let event_kind = input.event_kind.to_string();
    let title = input.title.to_string();
    let summary_md = input.summary_md.map(str::to_string);
    let decision = input.decision.map(str::to_string);
    let decision_bps = input.decision_bps;
    let new_value = input.new_value;
    let consensus_estimate = input.consensus_estimate;
    // If agent didn't compute the surprise, do it here.
    let surprise = input.surprise.or_else(|| match (new_value, consensus_estimate) {
        (Some(a), Some(c)) => Some(a - c),
        _ => None,
    });
    let previous_value = input.previous_value;
    let vote = input.vote.map(str::to_string);
    let dot_plot = input.dot_plot.map(str::to_string);
    let url = input.url.map(str::to_string);
    let source = input.source.to_string();
    let translations = input.translations.map(str::to_string);
    let now = jiff::Timestamp::now();

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| {
            row.update()
                .event_kind(event_kind)
                .title(title)
                .summary_md(summary_md)
                .decision(decision)
                .decision_bps(decision_bps)
                .new_value(new_value)
                .consensus_estimate(consensus_estimate)
                .surprise(surprise)
                .previous_value(previous_value)
                .vote(vote)
                .dot_plot(dot_plot)
                .url(url)
                .source(source)
                .translations(translations)
                .updated_at(now)
                .exec(d)
                .await
        })
        .await?;
        db.with(async |d| MacroEvent::filter_by_id(id).first().exec(d).await)
            .await?
            .ok_or(DbError::NotFound)
    } else {
        let indicator_code = input.indicator_code.to_string();
        let event_date = input.event_date.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(MacroEvent {
                    indicator_code: indicator_code,
                    event_date: event_date,
                    event_kind: event_kind,
                    title: title,
                    summary_md: summary_md,
                    decision: decision,
                    decision_bps: decision_bps,
                    new_value: new_value,
                    consensus_estimate: consensus_estimate,
                    surprise: surprise,
                    previous_value: previous_value,
                    vote: vote,
                    dot_plot: dot_plot,
                    url: url,
                    source: source,
                    translations: translations,
                    created_at: now,
                    updated_at: now,
                })
                .exec(d)
                .await
            })
            .await?;
        Ok(row)
    }
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = get(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
