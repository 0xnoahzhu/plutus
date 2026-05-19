use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::HashSet;

use std::collections::BTreeSet;

use crate::dto::watchlist::{
    WatchlistIn, WatchlistItemIn, WatchlistItemOut, WatchlistOut, WatchlistPatch,
    WatchlistStockOut,
};
use crate::error::ApiResult;
use crate::state::AppState;

/// Optional country filter accepted on `list` and `list_items`. ISO alpha-2.
#[derive(Debug, Deserialize)]
pub struct CountryFilter {
    pub country: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(filter): Query<CountryFilter>,
) -> ApiResult<Json<Vec<WatchlistOut>>> {
    let all = plutus_storage::queries::watchlists::list(&state.db).await?;
    let Some(country) = filter.country.as_deref() else {
        return Ok(Json(all.into_iter().map(Into::into).collect()));
    };

    // Resolve the country into a set of MIC codes, then keep only watchlists
    // that have at least one item whose stock trades on one of those markets.
    let market_codes: HashSet<String> =
        plutus_storage::queries::markets::list_codes_by_country(&state.db, country)
            .await?
            .into_iter()
            .collect();
    if market_codes.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // One pass: pull every stock once, then probe per watchlist.
    let stocks = plutus_storage::queries::stocks::list(&state.db).await?;
    let stock_market: std::collections::HashMap<i64, String> = stocks
        .into_iter()
        .map(|s| (s.id, s.market_code))
        .collect();

    let mut kept = Vec::new();
    for w in all {
        let items = plutus_storage::queries::watchlists::list_items(&state.db, w.id).await?;
        let has_match = items
            .iter()
            .any(|i| stock_market.get(&i.stock_id).map_or(false, |m| market_codes.contains(m)));
        if has_match {
            kept.push(w);
        }
    }
    Ok(Json(kept.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<WatchlistOut>> {
    let row = plutus_storage::queries::watchlists::get(&state.db, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<WatchlistIn>,
) -> ApiResult<Json<WatchlistOut>> {
    let row = plutus_storage::queries::watchlists::create(
        &state.db,
        &input.name,
        input.description.as_deref(),
        input.sort_order,
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(patch): Json<WatchlistPatch>,
) -> ApiResult<Json<WatchlistOut>> {
    // PATCH semantics:
    //   field omitted     → None              → leave alone
    //   field: null       → Some(None)        → clear
    //   field: "x"        → Some(Some("x"))   → set to "x"
    let description = patch
        .description
        .map(|inner| inner.as_deref().map(str::to_string));
    let row = plutus_storage::queries::watchlists::update(
        &state.db,
        id,
        patch.name.as_deref(),
        description.as_ref().map(|inner| inner.as_deref()),
        patch.sort_order,
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::watchlists::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn list_items(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(filter): Query<CountryFilter>,
) -> ApiResult<Json<Vec<WatchlistItemOut>>> {
    let items = plutus_storage::queries::watchlists::list_items(&state.db, id).await?;
    let Some(country) = filter.country.as_deref() else {
        return Ok(Json(items.into_iter().map(Into::into).collect()));
    };

    let market_codes: HashSet<String> =
        plutus_storage::queries::markets::list_codes_by_country(&state.db, country)
            .await?
            .into_iter()
            .collect();
    if market_codes.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let stocks = plutus_storage::queries::stocks::list(&state.db).await?;
    let stock_market: std::collections::HashMap<i64, String> = stocks
        .into_iter()
        .map(|s| (s.id, s.market_code))
        .collect();
    let kept: Vec<_> = items
        .into_iter()
        .filter(|i| stock_market.get(&i.stock_id).map_or(false, |m| market_codes.contains(m)))
        .collect();
    Ok(Json(kept.into_iter().map(Into::into).collect()))
}

pub async fn add_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<WatchlistItemIn>,
) -> ApiResult<Json<WatchlistItemOut>> {
    let row = plutus_storage::queries::watchlists::add_item(
        &state.db,
        id,
        input.stock_id,
        input.notes.as_deref(),
    )
    .await?;
    Ok(Json(row.into()))
}

/// Flatten all watchlist items into a deduplicated stock list, optionally
/// scoped to a single country. Each stock carries every watchlist id that
/// contains it. Useful for "what HK names am I tracking anywhere?" queries.
pub async fn list_stocks(
    State(state): State<AppState>,
    Query(filter): Query<CountryFilter>,
) -> ApiResult<Json<Vec<WatchlistStockOut>>> {
    let watchlists = plutus_storage::queries::watchlists::list(&state.db).await?;
    let stocks = plutus_storage::queries::stocks::list(&state.db).await?;
    let stock_by_id: std::collections::HashMap<i64, _> =
        stocks.into_iter().map(|s| (s.id, s)).collect();

    // stock_id → sorted set of watchlist ids
    let mut membership: std::collections::HashMap<i64, BTreeSet<i64>> =
        std::collections::HashMap::new();
    for w in &watchlists {
        let items = plutus_storage::queries::watchlists::list_items(&state.db, w.id).await?;
        for it in items {
            membership.entry(it.stock_id).or_default().insert(w.id);
        }
    }

    // Optional country filter — resolved into a set of MIC codes.
    let market_filter: Option<HashSet<String>> = match filter.country.as_deref() {
        Some(c) => Some(
            plutus_storage::queries::markets::list_codes_by_country(&state.db, c)
                .await?
                .into_iter()
                .collect(),
        ),
        None => None,
    };

    let mut out: Vec<WatchlistStockOut> = membership
        .into_iter()
        .filter_map(|(stock_id, group_ids)| {
            let s = stock_by_id.get(&stock_id)?;
            if let Some(mkts) = &market_filter {
                if !mkts.contains(&s.market_code) {
                    return None;
                }
            }
            Some(WatchlistStockOut {
                id: s.id,
                market_code: s.market_code.clone(),
                symbol: s.symbol.clone(),
                currency: s.currency.clone(),
                asset_class: s.asset_class.clone(),
                sector_code: s.sector_code.clone(),
                watchlist_ids: group_ids.into_iter().collect(),
            })
        })
        .collect();
    out.sort_by(|a, b| a.market_code.cmp(&b.market_code).then(a.symbol.cmp(&b.symbol)));
    Ok(Json(out))
}

pub async fn remove_item(
    State(state): State<AppState>,
    Path((id, stock_id)): Path<(i64, i64)>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::watchlists::remove_item(&state.db, id, stock_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
