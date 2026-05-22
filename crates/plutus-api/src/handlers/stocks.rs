use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::collections::HashSet;

use crate::dto::stock::{StockIn, StockOut, StockPatch};
use crate::error::{ApiError, ApiResult};
use crate::i18n::LocaleQuery;
use crate::state::AppState;

/// Per-page result cap so a 1-char `?q=a` can't dump every ticker.
const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 500;
/// Page-size default for `?page=N` requests. Smaller than the catalog
/// listing's MAX_LIMIT so a stock browser page is readable; agents
/// that want bulk can pass `?per_page=500` instead.
const DEFAULT_PER_PAGE: i64 = 20;

#[derive(Debug, Deserialize)]
pub struct StocksListFilter {
    /// ISO country code (US/HK/CN). Mapped to a set of MIC codes and
    /// matched against `stocks.market_code`.
    pub country: Option<String>,
    /// Exact ticker, case-insensitive. Returns 0 or 1 row.
    pub symbol: Option<String>,
    /// Substring across the ticker AND the localized `name` from content
    /// JSONB. Case-insensitive ILIKE. Designed for the "agent received a
    /// ticker-ish string, find the matching stock" workflow.
    pub q: Option<String>,
    /// Comma-separated list of stock ids for a precise fetch (e.g.
    /// `?ids=1,42,99`). When set, the limit cap is bypassed because
    /// the result set is bounded by the caller-supplied list. Used by
    /// pages that have already fetched user data (holdings, watchlist
    /// items, transactions) and need to join symbols/market codes
    /// without hitting the global LIMIT.
    pub ids: Option<String>,
    /// Result cap. Defaults to DEFAULT_LIMIT, clamped to MAX_LIMIT.
    /// Ignored when `ids` is set.
    pub limit: Option<i64>,
    /// 1-indexed page number. When set, response carries an
    /// `X-Total-Count` header with the total matching row count so
    /// the caller can render pagination controls. Mutually exclusive
    /// with `ids` (id-list fetches are already bounded). Pairs with
    /// `per_page` for the page size; defaults to DEFAULT_PER_PAGE.
    pub page: Option<i64>,
    /// Page size for the `?page=N` mode. Defaults to DEFAULT_PER_PAGE,
    /// clamped to MAX_LIMIT.
    pub per_page: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(filter): Query<StocksListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<axum::response::Response> {
    // Parse the optional id list. Empty / all-bad-numbers yields an
    // empty slice, which the storage layer handles by producing an
    // empty result set rather than a SQL error.
    let ids_owned: Option<Vec<i64>> = filter.ids.as_deref().map(|raw| {
        raw.split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect()
    });
    let paginating = filter.page.is_some() && ids_owned.is_none();
    // When the caller targets a specific id list we skip the cap; the
    // SQL only returns those rows anyway. When paginating, per_page
    // becomes the limit and (page-1)*per_page is the offset.
    let (effective_limit, effective_offset) = if ids_owned.is_some() {
        (None, None)
    } else if paginating {
        let per_page = match filter.per_page {
            Some(n) if n <= 0 => {
                return Err(ApiError::BadRequest("per_page must be > 0".into()));
            }
            Some(n) if n > MAX_LIMIT => {
                return Err(ApiError::BadRequest(format!(
                    "per_page must be ≤ {MAX_LIMIT}"
                )));
            }
            Some(n) => n,
            None => DEFAULT_PER_PAGE,
        };
        let page = filter.page.unwrap_or(1).max(1);
        let offset = (page - 1).saturating_mul(per_page);
        (Some(per_page), Some(offset))
    } else {
        let n = match filter.limit {
            Some(n) if n <= 0 => {
                return Err(ApiError::BadRequest("limit must be > 0".into()));
            }
            Some(n) if n > MAX_LIMIT => {
                return Err(ApiError::BadRequest(format!(
                    "limit must be ≤ {MAX_LIMIT}"
                )));
            }
            Some(n) => n,
            None => DEFAULT_LIMIT,
        };
        (Some(n), None)
    };
    let storage_filter = plutus_storage::queries::stocks::ListFilter {
        symbol: filter.symbol.as_deref(),
        q: filter.q.as_deref(),
        ids: ids_owned.as_deref(),
        limit: effective_limit,
        offset: effective_offset,
    };
    let rows =
        plutus_storage::queries::stocks::list(&state.db, &l.locale, storage_filter).await?;

    // Country filter happens AFTER the DB-level filters because the
    // country → MIC mapping is in code, not the DB. With a country
    // filter active we may post-filter to fewer than `limit` rows;
    // that's acceptable for the current ~10k-stock scale.
    let filtered: Vec<StockOut> = if let Some(country) = filter.country.as_deref() {
        let market_codes: HashSet<String> =
            plutus_storage::queries::markets::list_codes_by_country(&state.db, country)
                .await?
                .into_iter()
                .collect();
        rows.into_iter()
            .filter(|s| market_codes.contains(&s.market_code))
            .map(Into::into)
            .collect()
    } else {
        rows.into_iter().map(Into::into).collect()
    };

    // Pagination header: only compute COUNT when the caller actually
    // asked for pagination (saves a query on the agent's bulk fetches).
    let mut headers = HeaderMap::new();
    if paginating {
        let count_filter = plutus_storage::queries::stocks::ListFilter {
            symbol: filter.symbol.as_deref(),
            q: filter.q.as_deref(),
            ids: None,
            limit: None,
            offset: None,
        };
        let total = plutus_storage::queries::stocks::count(&state.db, &count_filter).await?;
        if let Ok(v) = HeaderValue::from_str(&total.to_string()) {
            headers.insert("X-Total-Count", v);
        }
    }

    Ok((headers, Json(filtered)).into_response())
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<StockOut>> {
    let row = plutus_storage::queries::stocks::get(&state.db, &l.locale, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<StockIn>,
) -> ApiResult<Json<StockOut>> {
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    let row = plutus_storage::queries::stocks::create(
        &state.db,
        plutus_storage::queries::stocks::NewStock {
            market_code: &input.market_code,
            symbol: &input.symbol,
            isin: input.isin.as_deref(),
            figi: input.figi.as_deref(),
            currency: &input.currency,
            lot_size: input.lot_size,
            asset_class: &input.asset_class,
            sector_code: input.sector_code.as_deref(),
            content: input.content,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
    Json(patch): Json<StockPatch>,
) -> ApiResult<Json<StockOut>> {
    let Some(content) = patch.content else {
        return Err(ApiError::BadRequest(
            "PATCH body must include `content` (full multi-locale blob)".into(),
        ));
    };
    if !content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    let row =
        plutus_storage::queries::stocks::update_content(&state.db, &l.locale, id, &content).await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::stocks::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
