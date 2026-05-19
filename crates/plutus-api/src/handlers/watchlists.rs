use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::HashSet;

use crate::dto::watchlist::{WatchlistItemIn, WatchlistItemOut};
use crate::error::ApiResult;
use crate::state::AppState;

/// Optional country filter (ISO alpha-2).
#[derive(Debug, Deserialize)]
pub struct CountryFilter {
    pub country: Option<String>,
}

pub async fn list_items(
    State(state): State<AppState>,
    Query(filter): Query<CountryFilter>,
) -> ApiResult<Json<Vec<WatchlistItemOut>>> {
    let items = plutus_storage::queries::watchlist::list_items(&state.db).await?;

    let kept = if let Some(country) = filter.country.as_deref() {
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
        items
            .into_iter()
            .filter(|i| {
                stock_market
                    .get(&i.stock_id)
                    .map_or(false, |m| market_codes.contains(m))
            })
            .collect()
    } else {
        items
    };

    Ok(Json(kept.into_iter().map(Into::into).collect()))
}

pub async fn add_item(
    State(state): State<AppState>,
    Json(input): Json<WatchlistItemIn>,
) -> ApiResult<Json<WatchlistItemOut>> {
    let row = plutus_storage::queries::watchlist::add_item(
        &state.db,
        input.stock_id,
        input.notes.as_deref(),
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn remove_item(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::watchlist::remove_item(&state.db, stock_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
