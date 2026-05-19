use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::MarketBrief;

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub country: Option<&'a str>,
    pub kind: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<MarketBrief>> {
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
    let user_id = filter.user_id;
    let from = filter.from.map(str::to_string);
    let to = filter.to.map(str::to_string);
    Ok(rows
        .into_iter()
        .filter(|r| r.user_id == user_id)
        .filter(|r| from.as_deref().map_or(true, |f| r.trade_date.as_str() >= f))
        .filter(|r| to.as_deref().map_or(true, |t| r.trade_date.as_str() <= t))
        .collect())
}

pub async fn get(db: &Db, user_id: i64, id: i64) -> Result<MarketBrief> {
    let row = db
        .with(async |d| MarketBrief::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewBrief<'a> {
    pub user_id: i64,
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

/// Upsert by natural key (user_id, country, kind, trade_date). Replaces fields
/// when a row already exists for that day; inserts otherwise.
pub async fn upsert(db: &Db, input: NewBrief<'_>) -> Result<MarketBrief> {
    let user_id = input.user_id;
    let country_owned = input.country.to_string();
    let kind_owned = input.kind.to_string();
    let date_owned = input.trade_date.to_string();

    let existing = db
        .with(async |d| {
            MarketBrief::all()
                .filter(MarketBrief::fields().country().eq(&country_owned))
                .filter(MarketBrief::fields().kind().eq(&kind_owned))
                .filter(MarketBrief::fields().trade_date().eq(&date_owned))
                .exec(d)
                .await
        })
        .await?
        .into_iter()
        .find(|r| r.user_id == user_id);

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
        get(db, user_id, id).await
    } else {
        let country = input.country.to_string();
        let kind = input.kind.to_string();
        let trade_date = input.trade_date.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(MarketBrief {
                    user_id: user_id,
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

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let row = get(db, user_id, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
