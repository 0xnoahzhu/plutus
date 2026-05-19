use axum::extract::{Path, Query, State};
use axum::Json;
use std::collections::HashMap;

use crate::dto::news::{
    NewsCountryLinkIn, NewsCountryLinkOut, NewsIn, NewsMacroLinkIn, NewsMacroLinkOut, NewsOut,
    NewsSectorLinkIn, NewsSectorLinkOut, NewsStockLinkIn, NewsStockLinkOut,
    NewsTranslationIn, NewsTranslationOut,
};
use crate::error::{ApiError, ApiResult};
use crate::i18n::{LocaleQuery, DEFAULT_LOCALE};
use crate::state::AppState;

fn parse_ts(s: &str, field: &str) -> ApiResult<jiff::Timestamp> {
    s.parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("{field}: {e}")))
}

/// Patch `out` with non-empty fields from a news_translations row. Translation
/// rows can omit fields (the agent only filled in the parts it cared about),
/// so we keep the base-language value whenever the translation is missing or
/// empty.
fn apply_news_translation(
    out: &mut NewsOut,
    t: &plutus_storage::models::NewsTranslation,
) {
    if !t.title.is_empty() {
        out.title = t.title.clone();
    }
    if let Some(s) = &t.summary {
        if !s.is_empty() {
            out.summary = Some(s.clone());
        }
    }
    if let Some(s) = &t.content_md {
        if !s.is_empty() {
            out.content_md = Some(s.clone());
        }
    }
    if let Some(s) = &t.agent_summary_md {
        if !s.is_empty() {
            out.agent_summary_md = Some(s.clone());
        }
    }
}

pub async fn list(
    State(state): State<AppState>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<NewsOut>>> {
    let rows = plutus_storage::queries::news::list(&state.db).await?;
    let mut out: Vec<NewsOut> = rows.into_iter().map(Into::into).collect();
    if l.locale != DEFAULT_LOCALE {
        let trans = plutus_storage::queries::news::list_translations_for_locale(
            &state.db,
            &l.locale,
        )
        .await?;
        let by_news: HashMap<i64, plutus_storage::models::NewsTranslation> =
            trans.into_iter().map(|t| (t.news_id, t)).collect();
        for o in out.iter_mut() {
            if let Some(t) = by_news.get(&o.id) {
                apply_news_translation(o, t);
            }
        }
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<NewsOut>> {
    let row = plutus_storage::queries::news::get(&state.db, id).await?;
    let mut out: NewsOut = row.into();
    if l.locale != DEFAULT_LOCALE {
        if let Some(t) =
            plutus_storage::queries::news::get_translation(&state.db, id, &l.locale).await?
        {
            apply_news_translation(&mut out, &t);
        }
    }
    Ok(Json(out))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<NewsIn>,
) -> ApiResult<Json<NewsOut>> {
    let published_at = parse_ts(&input.published_at, "published_at")?;
    let fetched_at = match input.fetched_at.as_deref() {
        Some(s) => Some(parse_ts(s, "fetched_at")?),
        None => None,
    };
    let last_verified_at = match input.last_verified_at.as_deref() {
        Some(s) => Some(parse_ts(s, "last_verified_at")?),
        None => None,
    };
    let row = plutus_storage::queries::news::create(
        &state.db,
        plutus_storage::queries::news::NewNewsItem {
            external_id: input.external_id.as_deref(),
            url: &input.url,
            canonical_url: input.canonical_url.as_deref(),
            archive_url: input.archive_url.as_deref(),
            url_status: input.url_status,
            last_verified_at,
            title: &input.title,
            summary: input.summary.as_deref(),
            content_md: input.content_md.as_deref(),
            agent_summary_md: input.agent_summary_md.as_deref(),
            language: &input.language,
            source: &input.source,
            source_kind: &input.source_kind,
            category: &input.category,
            region: &input.region,
            published_at,
            fetched_at,
            sentiment: input.sentiment.as_deref(),
            sentiment_score: input.sentiment_score,
            importance: &input.importance,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::news::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Stock links ───────────────────────────────────────────────────────────

pub async fn list_stock_links(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
) -> ApiResult<Json<Vec<NewsStockLinkOut>>> {
    let rows = plutus_storage::queries::news::list_stock_links(&state.db, news_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn list_news_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<NewsStockLinkOut>>> {
    let rows = plutus_storage::queries::news::list_news_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn add_stock_link(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
    Json(input): Json<NewsStockLinkIn>,
) -> ApiResult<Json<NewsStockLinkOut>> {
    let row = plutus_storage::queries::news::add_stock_link(
        &state.db,
        news_id,
        input.stock_id,
        &input.relation,
        input.relevance,
    )
    .await?;
    Ok(Json(row.into()))
}

// ── Sector / macro / country links ────────────────────────────────────────

pub async fn list_sector_links(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
) -> ApiResult<Json<Vec<NewsSectorLinkOut>>> {
    let rows = plutus_storage::queries::news::list_sector_links(&state.db, news_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn add_sector_link(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
    Json(input): Json<NewsSectorLinkIn>,
) -> ApiResult<Json<NewsSectorLinkOut>> {
    let row = plutus_storage::queries::news::add_sector_link(
        &state.db,
        news_id,
        &input.sector_code,
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn list_macro_links(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
) -> ApiResult<Json<Vec<NewsMacroLinkOut>>> {
    let rows = plutus_storage::queries::news::list_macro_links(&state.db, news_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn add_macro_link(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
    Json(input): Json<NewsMacroLinkIn>,
) -> ApiResult<Json<NewsMacroLinkOut>> {
    let row = plutus_storage::queries::news::add_macro_link(
        &state.db,
        news_id,
        &input.indicator_code,
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn list_country_links(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
) -> ApiResult<Json<Vec<NewsCountryLinkOut>>> {
    let rows = plutus_storage::queries::news::list_country_links(&state.db, news_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn add_country_link(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
    Json(input): Json<NewsCountryLinkIn>,
) -> ApiResult<Json<NewsCountryLinkOut>> {
    let row =
        plutus_storage::queries::news::add_country_link(&state.db, news_id, &input.country).await?;
    Ok(Json(row.into()))
}

// ── Translations ─────────────────────────────────────────────────────────

pub async fn list_translations(
    State(state): State<AppState>,
    Path(news_id): Path<i64>,
) -> ApiResult<Json<Vec<NewsTranslationOut>>> {
    let rows = plutus_storage::queries::news::list_translations(&state.db, news_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn put_translation(
    State(state): State<AppState>,
    Path((news_id, locale)): Path<(i64, String)>,
    Json(input): Json<NewsTranslationIn>,
) -> ApiResult<Json<NewsTranslationOut>> {
    let row = plutus_storage::queries::news::upsert_translation(
        &state.db,
        news_id,
        &locale,
        &input.title,
        input.summary.as_deref(),
        input.content_md.as_deref(),
        input.agent_summary_md.as_deref(),
        &input.translator,
    )
    .await?;
    Ok(Json(row.into()))
}
