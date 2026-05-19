use crate::db::{Db, DbError, Result};
use crate::models::SelfExam;

pub struct ListFilter<'a> {
    pub kind: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<SelfExam>> {
    let rows = if let Some(k) = filter.kind {
        let k = k.to_string();
        db.with(async |d| {
            SelfExam::all()
                .filter(SelfExam::fields().kind().eq(&k))
                .exec(d)
                .await
        })
        .await?
    } else {
        db.with(async |d| SelfExam::all().exec(d).await).await?
    };
    let from = filter.from.map(str::to_string);
    let to = filter.to.map(str::to_string);
    Ok(rows
        .into_iter()
        .filter(|r| from.as_deref().map_or(true, |f| r.period_start.as_str() >= f))
        .filter(|r| to.as_deref().map_or(true, |t| r.period_start.as_str() <= t))
        .collect())
}

pub async fn get(db: &Db, id: i64) -> Result<SelfExam> {
    db.with(async |d| SelfExam::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub struct NewExam<'a> {
    pub kind: &'a str,
    pub period_start: &'a str,
    pub period_end: &'a str,
    pub headline: &'a str,
    pub content_md: Option<&'a str>,
    pub metrics: Option<&'a str>,
    pub recommendation_ids: Option<&'a str>, // JSON array
    pub notes: Option<&'a str>,
    pub language: &'a str,
    pub source: &'a str,
    pub translations: Option<&'a str>,
}

pub async fn upsert(db: &Db, input: NewExam<'_>) -> Result<SelfExam> {
    let kind_owned = input.kind.to_string();
    let start_owned = input.period_start.to_string();
    let existing = db
        .with(async |d| {
            SelfExam::all()
                .filter(SelfExam::fields().kind().eq(&kind_owned))
                .filter(SelfExam::fields().period_start().eq(&start_owned))
                .first()
                .exec(d)
                .await
        })
        .await?;

    let period_end = input.period_end.to_string();
    let headline = input.headline.to_string();
    let content_md = input.content_md.map(str::to_string);
    let metrics = input.metrics.map(str::to_string);
    let recommendation_ids = input.recommendation_ids.map(str::to_string);
    let notes = input.notes.map(str::to_string);
    let language = input.language.to_string();
    let source = input.source.to_string();
    let translations = input.translations.map(str::to_string);
    let now = jiff::Timestamp::now();

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| {
            row.update()
                .period_end(period_end)
                .headline(headline)
                .content_md(content_md)
                .metrics(metrics)
                .recommendation_ids(recommendation_ids)
                .notes(notes)
                .language(language)
                .source(source)
                .translations(translations)
                .updated_at(now)
                .exec(d)
                .await
        })
        .await?;
        db.with(async |d| SelfExam::filter_by_id(id).first().exec(d).await)
            .await?
            .ok_or(DbError::NotFound)
    } else {
        let kind = input.kind.to_string();
        let period_start = input.period_start.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(SelfExam {
                    kind: kind,
                    period_start: period_start,
                    period_end: period_end,
                    headline: headline,
                    content_md: content_md,
                    metrics: metrics,
                    recommendation_ids: recommendation_ids,
                    notes: notes,
                    language: language,
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
