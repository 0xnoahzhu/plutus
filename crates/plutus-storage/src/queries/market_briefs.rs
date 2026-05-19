use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::MarketBrief;

pub struct ListFilter<'a> {
    pub country: Option<&'a str>,
    pub kind: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<MarketBrief>> {
    // toasty can express AND chains of `eq`, but for "from <= date <= to" we
    // fall through to filtering in Rust after pulling the row set down. Per
    // resource these stay small (< 365 rows per country/year) so this is
    // fine for Phase 0.
    let rows = if let Some(c) = filter.country {
        let c = c.to_string();
        if let Some(k) = filter.kind {
            let k = k.to_string();
            db.with(async |d| {
                MarketBrief::all()
                    .filter(MarketBrief::fields().country().eq(&c))
                    .filter(MarketBrief::fields().kind().eq(&k))
                    .exec(d)
                    .await
            })
            .await?
        } else {
            db.with(async |d| {
                MarketBrief::all()
                    .filter(MarketBrief::fields().country().eq(&c))
                    .exec(d)
                    .await
            })
            .await?
        }
    } else if let Some(k) = filter.kind {
        let k = k.to_string();
        db.with(async |d| {
            MarketBrief::all()
                .filter(MarketBrief::fields().kind().eq(&k))
                .exec(d)
                .await
        })
        .await?
    } else {
        db.with(async |d| MarketBrief::all().exec(d).await).await?
    };
    let from = filter.from.map(str::to_string);
    let to = filter.to.map(str::to_string);
    Ok(rows
        .into_iter()
        .filter(|r| from.as_deref().map_or(true, |f| r.trade_date.as_str() >= f))
        .filter(|r| to.as_deref().map_or(true, |t| r.trade_date.as_str() <= t))
        .collect())
}

pub async fn get(db: &Db, id: i64) -> Result<MarketBrief> {
    db.with(async |d| MarketBrief::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub struct NewBrief<'a> {
    pub country: &'a str,
    pub kind: &'a str,
    pub trade_date: &'a str,
    pub headline: &'a str,
    pub content_md: Option<&'a str>,
    pub sentiment: Option<&'a str>,
    pub sentiment_score: Option<Decimal>,
    pub source: &'a str,
    pub language: &'a str,
    pub translations: Option<&'a str>,
}

/// Upsert by natural key (country, kind, trade_date). Replaces fields when a
/// row already exists for that day; inserts otherwise.
pub async fn upsert(db: &Db, input: NewBrief<'_>) -> Result<MarketBrief> {
    let country_owned = input.country.to_string();
    let kind_owned = input.kind.to_string();
    let date_owned = input.trade_date.to_string();

    let existing = db
        .with(async |d| {
            MarketBrief::all()
                .filter(MarketBrief::fields().country().eq(&country_owned))
                .filter(MarketBrief::fields().kind().eq(&kind_owned))
                .filter(MarketBrief::fields().trade_date().eq(&date_owned))
                .first()
                .exec(d)
                .await
        })
        .await?;

    let headline = input.headline.to_string();
    let content_md = input.content_md.map(str::to_string);
    let sentiment = input.sentiment.map(str::to_string);
    let sentiment_score = input.sentiment_score;
    let source = input.source.to_string();
    let language = input.language.to_string();
    let translations = input.translations.map(str::to_string);
    let now = jiff::Timestamp::now();

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| {
            row.update()
                .headline(headline)
                .content_md(content_md)
                .sentiment(sentiment)
                .sentiment_score(sentiment_score)
                .source(source)
                .language(language)
                .translations(translations)
                .updated_at(now)
                .exec(d)
                .await
        })
        .await?;
        let updated = db
            .with(async |d| MarketBrief::filter_by_id(id).first().exec(d).await)
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(updated)
    } else {
        let country = input.country.to_string();
        let kind = input.kind.to_string();
        let trade_date = input.trade_date.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(MarketBrief {
                    country: country,
                    kind: kind,
                    trade_date: trade_date,
                    headline: headline,
                    content_md: content_md,
                    sentiment: sentiment,
                    sentiment_score: sentiment_score,
                    source: source,
                    language: language,
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
