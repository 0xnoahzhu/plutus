use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::HashSet;

use plutus_core::audit::Actor;

use crate::dto::watchlist::{WatchlistItemIn, WatchlistItemOut};
use crate::error::ApiResult;
use crate::handlers::access::require_user;
use crate::state::AppState;

/// Optional country filter (ISO alpha-2).
#[derive(Debug, Deserialize)]
pub struct CountryFilter {
    pub country: Option<String>,
}

pub async fn list_items(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(filter): Query<CountryFilter>,
) -> ApiResult<Json<Vec<WatchlistItemOut>>> {
    let user_id = require_user(&actor.0)?;
    let items = plutus_storage::queries::watchlist::list_items(&state.db, user_id).await?;

    let kept = if let Some(country) = filter.country.as_deref() {
        let market_codes: HashSet<String> =
            plutus_storage::queries::markets::list_codes_by_country(&state.db, country)
                .await?
                .into_iter()
                .collect();
        if market_codes.is_empty() {
            return Ok(Json(Vec::new()));
        }
        // Stock translatable text is not needed here — we only consult
        // (id, market_code) for the country filter. Pass "en" so the
        // projection picks the default locale without an extra hop.
        let stocks = plutus_storage::queries::stocks::list(&state.db, "en").await?;
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
