use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::HashSet;

use crate::dto::stock::{StockIn, StockOut, StockPatch};
use crate::error::{ApiError, ApiResult};
use crate::i18n::LocaleQuery;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StocksListFilter {
    pub country: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(filter): Query<StocksListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<StockOut>>> {
    let rows = plutus_storage::queries::stocks::list(&state.db, &l.locale).await?;
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
