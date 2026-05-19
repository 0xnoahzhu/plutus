use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::{
    NewsCountryLink, NewsItem, NewsMacroLink, NewsSectorLink, NewsStockLink, NewsTranslation,
};

// ── News items ────────────────────────────────────────────────────────────

pub async fn list(db: &Db) -> Result<Vec<NewsItem>> {
    db.with(async |d| NewsItem::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, id: i64) -> Result<NewsItem> {
    db.with(async |d| NewsItem::filter_by_id(id).first().exec(d).await)
        .await?
        .ok_or(DbError::NotFound)
}

pub async fn find_by_url(db: &Db, url: &str) -> Result<Option<NewsItem>> {
    let url = url.to_string();
    db.with(async |d| NewsItem::filter_by_url(url).first().exec(d).await)
        .await
        .map_err(Into::into)
}

pub struct NewNewsItem<'a> {
    pub external_id: Option<&'a str>,
    pub url: &'a str,
    pub canonical_url: Option<&'a str>,
    pub archive_url: Option<&'a str>,
    pub url_status: Option<i32>,
    pub last_verified_at: Option<jiff::Timestamp>,
    pub title: &'a str,
    pub summary: Option<&'a str>,
    pub content_md: Option<&'a str>,
    pub agent_summary_md: Option<&'a str>,
    pub language: &'a str,
    pub source: &'a str,
    pub source_kind: &'a str,
    pub category: &'a str,
    pub region: &'a str,
    pub published_at: jiff::Timestamp,
    pub fetched_at: Option<jiff::Timestamp>,
    pub sentiment: Option<&'a str>,
    pub sentiment_score: Option<Decimal>,
    pub importance: &'a str,
}

pub async fn create(db: &Db, input: NewNewsItem<'_>) -> Result<NewsItem> {
    let now = jiff::Timestamp::now();
    let external_id = input.external_id.map(str::to_string);
    let url = input.url.to_string();
    let canonical_url = input.canonical_url.map(str::to_string);
    let archive_url = input.archive_url.map(str::to_string);
    let url_status = input.url_status;
    let last_verified_at = input.last_verified_at;
    let title = input.title.to_string();
    let summary = input.summary.map(str::to_string);
    let content_md = input.content_md.map(str::to_string);
    let agent_summary_md = input.agent_summary_md.map(str::to_string);
    let language = input.language.to_string();
    let source = input.source.to_string();
    let source_kind = input.source_kind.to_string();
    let category = input.category.to_string();
    let region = input.region.to_string();
    let published_at = input.published_at;
    let fetched_at = input.fetched_at.unwrap_or(now);
    let sentiment = input.sentiment.map(str::to_string);
    let sentiment_score = input.sentiment_score;
    let importance = input.importance.to_string();

    let row = db
        .with(async |d| {
            toasty::create!(NewsItem {
                external_id: external_id,
                url: url,
                canonical_url: canonical_url,
                archive_url: archive_url,
                url_status: url_status,
                last_verified_at: last_verified_at,
                title: title,
                summary: summary,
                content_md: content_md,
                agent_summary_md: agent_summary_md,
                language: language,
                source: source,
                source_kind: source_kind,
                category: category,
                region: region,
                published_at: published_at,
                fetched_at: fetched_at,
                sentiment: sentiment,
                sentiment_score: sentiment_score,
                importance: importance,
                created_at: now,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let row = get(db, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}

// ── Stock links ───────────────────────────────────────────────────────────

pub async fn list_stock_links(db: &Db, news_id: i64) -> Result<Vec<NewsStockLink>> {
    db.with(async |d| {
        NewsStockLink::all()
            .filter(NewsStockLink::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn list_news_for_stock(db: &Db, stock_id: i64) -> Result<Vec<NewsStockLink>> {
    db.with(async |d| {
        NewsStockLink::all()
            .filter(NewsStockLink::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn add_stock_link(
    db: &Db,
    news_id: i64,
    stock_id: i64,
    relation: &str,
    relevance: Option<Decimal>,
) -> Result<NewsStockLink> {
    let relation = relation.to_string();
    let row = db
        .with(async |d| {
            toasty::create!(NewsStockLink {
                news_id: news_id,
                stock_id: stock_id,
                relation: relation,
                relevance: relevance,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

// ── Sector / macro / country links ────────────────────────────────────────

pub async fn add_sector_link(db: &Db, news_id: i64, sector_code: &str) -> Result<NewsSectorLink> {
    let sector_code = sector_code.to_string();
    db.with(async |d| {
        toasty::create!(NewsSectorLink { news_id: news_id, sector_code: sector_code })
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn add_macro_link(
    db: &Db,
    news_id: i64,
    indicator_code: &str,
) -> Result<NewsMacroLink> {
    let indicator_code = indicator_code.to_string();
    db.with(async |d| {
        toasty::create!(NewsMacroLink { news_id: news_id, indicator_code: indicator_code })
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn add_country_link(db: &Db, news_id: i64, country: &str) -> Result<NewsCountryLink> {
    let country = country.to_string();
    db.with(async |d| {
        toasty::create!(NewsCountryLink { news_id: news_id, country: country })
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn list_sector_links(db: &Db, news_id: i64) -> Result<Vec<NewsSectorLink>> {
    db.with(async |d| {
        NewsSectorLink::all()
            .filter(NewsSectorLink::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn list_macro_links(db: &Db, news_id: i64) -> Result<Vec<NewsMacroLink>> {
    db.with(async |d| {
        NewsMacroLink::all()
            .filter(NewsMacroLink::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn list_country_links(db: &Db, news_id: i64) -> Result<Vec<NewsCountryLink>> {
    db.with(async |d| {
        NewsCountryLink::all()
            .filter(NewsCountryLink::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

// ── Translations ─────────────────────────────────────────────────────────

pub async fn list_translations(db: &Db, news_id: i64) -> Result<Vec<NewsTranslation>> {
    db.with(async |d| {
        NewsTranslation::all()
            .filter(NewsTranslation::fields().news_id().eq(news_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

/// Fetch all translations for the given locale across every news item.
/// Used by the news list endpoint to localize page-wide via `?locale=`.
pub async fn list_translations_for_locale(
    db: &Db,
    locale: &str,
) -> Result<Vec<NewsTranslation>> {
    let locale = locale.to_string();
    db.with(async |d| {
        NewsTranslation::all()
            .filter(NewsTranslation::fields().locale().eq(&locale))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

/// Fetch a single translation for (news_id, locale), if any.
pub async fn get_translation(
    db: &Db,
    news_id: i64,
    locale: &str,
) -> Result<Option<NewsTranslation>> {
    let locale = locale.to_string();
    db.with(async |d| {
        NewsTranslation::all()
            .filter(NewsTranslation::fields().news_id().eq(news_id))
            .filter(NewsTranslation::fields().locale().eq(&locale))
            .first()
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub async fn upsert_translation(
    db: &Db,
    news_id: i64,
    locale: &str,
    title: &str,
    summary: Option<&str>,
    content_md: Option<&str>,
    agent_summary_md: Option<&str>,
    translator: &str,
) -> Result<NewsTranslation> {
    let now = jiff::Timestamp::now();
    let locale_owned = locale.to_string();
    let existing = db
        .with(async |d| {
            NewsTranslation::all()
                .filter(NewsTranslation::fields().news_id().eq(news_id))
                .filter(NewsTranslation::fields().locale().eq(&locale_owned))
                .first()
                .exec(d)
                .await
        })
        .await?;

    let title = title.to_string();
    let summary = summary.map(str::to_string);
    let content_md = content_md.map(str::to_string);
    let agent_summary_md = agent_summary_md.map(str::to_string);
    let translator = translator.to_string();

    if let Some(mut row) = existing {
        let id = row.id;
        db.with(async |d| {
            row.update()
                .title(title)
                .summary(summary)
                .content_md(content_md)
                .agent_summary_md(agent_summary_md)
                .translator(translator)
                .updated_at(now)
                .exec(d)
                .await
        })
        .await?;
        let updated = db
            .with(async |d| NewsTranslation::filter_by_id(id).first().exec(d).await)
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(updated)
    } else {
        let locale = locale.to_string();
        let row = db
            .with(async |d| {
                toasty::create!(NewsTranslation {
                    news_id: news_id,
                    locale: locale,
                    title: title,
                    summary: summary,
                    content_md: content_md,
                    agent_summary_md: agent_summary_md,
                    translator: translator,
                    updated_at: now,
                })
                .exec(d)
                .await
            })
            .await?;
        Ok(row)
    }
}
