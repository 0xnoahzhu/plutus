use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use plutus_core::audit::Actor;

use crate::dto::watchlist::{WatchlistItemIn, WatchlistItemOut};
use crate::error::ApiResult;
use crate::handlers::access::require_user;
use crate::state::AppState;

const DEFAULT_PER_PAGE: i64 = 15;
const MAX_PER_PAGE: i64 = 500;

#[derive(Debug, Deserialize)]
pub struct ListItemsFilter {
    pub country: Option<String>,
    /// Case-insensitive substring match on stock symbol.
    pub q: Option<String>,
    /// 1-indexed page. When set, response carries X-Total-Count.
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list_items(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(filter): Query<ListItemsFilter>,
) -> ApiResult<axum::response::Response> {
    let user_id = require_user(&actor.0)?;
    let items = plutus_storage::queries::watchlist::list_items(&state.db, user_id).await?;

    // Resolve stock metadata for every item; the q-filter searches
    // against symbol/name and the country filter against market_code.
    let stock_ids: Vec<i64> = items.iter().map(|i| i.stock_id).collect();
    let stocks = plutus_storage::queries::stocks::list(
        &state.db,
        "en",
        plutus_storage::queries::stocks::ListFilter {
            symbol: None,
            q: None,
            ids: Some(&stock_ids),
            limit: None,
            offset: None,
        },
    )
    .await?;
    let stock_meta: HashMap<i64, (String, String, Option<String>)> = stocks
        .into_iter()
        .map(|s| (s.id, (s.symbol, s.market_code, s.name)))
        .collect();

    // Country filter.
    let market_codes: Option<HashSet<String>> = if let Some(country) = filter.country.as_deref() {
        let codes: HashSet<String> =
            plutus_storage::queries::markets::list_codes_by_country(&state.db, country)
                .await?
                .into_iter()
                .collect();
        Some(codes)
    } else {
        None
    };

    let q_upper = filter
        .q
        .as_deref()
        .map(|s| s.trim().to_ascii_uppercase())
        .filter(|s| !s.is_empty());

    let filtered: Vec<_> = items
        .into_iter()
        .filter(|i| {
            let Some((symbol, market_code, name)) = stock_meta.get(&i.stock_id) else {
                return false;
            };
            if let Some(ref codes) = market_codes {
                if !codes.contains(market_code) {
                    return false;
                }
            }
            if let Some(ref q) = q_upper {
                let sym_match = symbol.to_ascii_uppercase().contains(q);
                let name_match = name
                    .as_deref()
                    .map(|n| n.to_ascii_uppercase().contains(q))
                    .unwrap_or(false);
                if !sym_match && !name_match {
                    return false;
                }
            }
            true
        })
        .collect();

    let total = filtered.len() as i64;
    let paginating = filter.page.is_some();
    let page_slice: Vec<_> = if paginating {
        let per_page = filter
            .per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE);
        let page = filter.page.unwrap_or(1).max(1);
        let offset = ((page - 1) * per_page) as usize;
        filtered
            .into_iter()
            .skip(offset)
            .take(per_page as usize)
            .collect()
    } else {
        filtered
    };

    let mut headers = HeaderMap::new();
    if paginating {
        if let Ok(v) = HeaderValue::from_str(&total.to_string()) {
            headers.insert("X-Total-Count", v);
        }
    }
    let out: Vec<WatchlistItemOut> = page_slice.into_iter().map(Into::into).collect();
    Ok((headers, Json(out)).into_response())
}

pub async fn add_item(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<WatchlistItemIn>,
) -> ApiResult<Json<WatchlistItemOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::watchlist::add_item(
        &state.db,
        user_id,
        input.stock_id,
        input.notes.as_deref(),
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn remove_item(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let user_id = require_user(&actor.0)?;
    plutus_storage::queries::watchlist::remove_item(&state.db, user_id, stock_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
