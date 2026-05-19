use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::HashSet;

use crate::dto::stock::{
    StockIn, StockOut, StockPatch, StockTranslationIn, StockTranslationOut,
};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StocksListFilter {
    pub country: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(filter): Query<StocksListFilter>,
) -> ApiResult<Json<Vec<StockOut>>> {
    let rows = plutus_storage::queries::stocks::list(&state.db).await?;
    let Some(country) = filter.country.as_deref() else {
        return Ok(Json(rows.into_iter().map(Into::into).collect()));
    };

    // Resolve country → set of MIC codes, then filter stocks by market_code.
    // Unknown country (empty market set) returns no rows.
    let market_codes: HashSet<String> =
        plutus_storage::queries::markets::list_codes_by_country(&state.db, country)
            .await?
            .into_iter()
            .collect();
    let filtered: Vec<_> = rows
        .into_iter()
        .filter(|s| market_codes.contains(&s.market_code))
        .collect();
    Ok(Json(filtered.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<StockOut>> {
    let row = plutus_storage::queries::stocks::get(&state.db, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<StockIn>,
) -> ApiResult<Json<StockOut>> {
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
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn update(
    State(_state): State<AppState>,
    Path(_id): Path<i64>,
    Json(_patch): Json<StockPatch>,
) -> ApiResult<Json<StockOut>> {
    // Field-by-field updates not implemented in Phase 0; the agent can delete + recreate
    // or update translations. Returning 501 keeps the contract honest.
    Err(ApiError::BadRequest(
        "PATCH /stocks/:id is reserved for Phase 1".into(),
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::stocks::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn list_translations(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<Vec<StockTranslationOut>>> {
    let rows = plutus_storage::queries::stock_translations::list_for_stock(&state.db, id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn put_translation(
    State(state): State<AppState>,
    Path((id, locale)): Path<(i64, String)>,
    Json(input): Json<StockTranslationIn>,
) -> ApiResult<Json<StockTranslationOut>> {
    let row = plutus_storage::queries::stock_translations::upsert(
        &state.db,
        id,
        &locale,
        &input.name,
        input.description_md.as_deref(),
    )
    .await?;
    Ok(Json(row.into()))
}
