use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;

use plutus_core::audit::Actor;
use plutus_storage::queries::unread::{self, EntityKind};

use crate::dto::news::{
    NewsCountryLinkIn, NewsCountryLinkOut, NewsIn, NewsMacroLinkIn, NewsMacroLinkOut, NewsOut,
    NewsSectorLinkIn, NewsSectorLinkOut, NewsStockLinkIn, NewsStockLinkOut,
};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::maybe_user_id;
use crate::handlers::pagination::{
    clamp_limit, clamp_offset, paginate_slice, paginated_response_headers, PaginationFilter,
};
use crate::i18n::LocaleQuery;
use crate::state::AppState;

fn parse_ts(s: &str, field: &str) -> ApiResult<jiff::Timestamp> {
    s.parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("{field}: {e}")))
}

/// Coerce the caller-supplied region to the canonical form the storage
/// layer and the web filters expect: uppercase ISO-ish country code
/// (`US` / `HK` / `CN`) or the literal `global`. Case is normalized so a
/// lowercase POST still lands as the right row; anything else is a 400
/// — silent fallbacks would hide the agent's bug.
fn normalize_region(raw: &str) -> ApiResult<String> {
    let trimmed = raw.trim();
    match trimmed.to_ascii_uppercase().as_str() {
        "US" => Ok("US".into()),
        "HK" => Ok("HK".into()),
        "CN" => Ok("CN".into()),
        "GLOBAL" => Ok("global".into()),
        _ => Err(ApiError::BadRequest(format!(
            "region must be one of US, HK, CN, global (got `{raw}`)"
        ))),
    }
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(l): Query<LocaleQuery>,
    Query(p): Query<PaginationFilter>,
) -> ApiResult<axum::response::Response> {
    let limit = clamp_limit(p.limit)?;
    let offset = clamp_offset(p.offset)?;
    let rows = plutus_storage::queries::news::list(&state.db, &l.locale).await?;
    let total = rows.len() as i64;
    let mut body: Vec<NewsOut> = paginate_slice(rows, limit, offset)
        .into_iter()
        .map(Into::into)
        .collect();
    if let Some(user_id) = maybe_user_id(&actor.0) {
        let ids: Vec<i64> = body.iter().map(|n| n.id).collect();
        let read_ats =
            unread::read_ats(&state.db, user_id, EntityKind::News, &ids).await?;
        for n in &mut body {
            n.read_at = read_ats.get(&n.id).map(jiff::Timestamp::to_string);
        }
    }
    if p.is_paginating() {
        Ok((paginated_response_headers(total), Json(body)).into_response())
    } else {
        Ok(Json(body).into_response())
    }
}


pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<NewsOut>> {
    let row = plutus_storage::queries::news::get(&state.db, &l.locale, id).await?;
    let mut out: NewsOut = row.into();
    if let Some(user_id) = maybe_user_id(&actor.0) {
        unread::mark_read(&state.db, user_id, EntityKind::News, id).await?;
        out.read_at = Some(jiff::Timestamp::now().to_string());
    }
    Ok(Json(out))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<NewsIn>,
) -> ApiResult<Json<NewsOut>> {
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    let region = normalize_region(&input.region)?;
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
            source: &input.source,
            source_kind: &input.source_kind,
            category: &input.category,
            region: &region,
            published_at,
            fetched_at,
            sentiment: input.sentiment.as_deref(),
            sentiment_score: input.sentiment_score,
            importance: &input.importance,
            content: input.content,
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
