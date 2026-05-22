use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;

use crate::dto::analyst::{
    AnalystEstimateBatchIn, AnalystEstimateBatchOut, AnalystEstimateIn, AnalystEstimateOut,
    AnalystRatingBatchIn, AnalystRatingBatchOut, AnalystRatingIn, AnalystRatingOut,
};
use crate::error::ApiResult;
use crate::handlers::batch::validate_batch_size;
use crate::handlers::pagination::{
    clamp_limit, clamp_offset, paginated_response_headers, PaginationFilter,
};
use crate::state::AppState;

pub async fn list_estimates(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
    Query(p): Query<PaginationFilter>,
) -> ApiResult<axum::response::Response> {
    let limit = clamp_limit(p.limit)?;
    let offset = clamp_offset(p.offset)?;
    let rows = plutus_storage::queries::analyst::list_estimates_for_stock(
        &state.db, stock_id, limit, offset,
    )
    .await?;
    let body: Vec<AnalystEstimateOut> = rows.into_iter().map(Into::into).collect();
    if p.is_paginating() {
        let total =
            plutus_storage::queries::analyst::count_estimates_for_stock(&state.db, stock_id)
                .await?;
        Ok((paginated_response_headers(total), Json(body)).into_response())
    } else {
        Ok(Json(body).into_response())
    }
}

pub async fn delete_estimate(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::analyst::delete_estimate(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn insert_estimate(
    State(state): State<AppState>,
    Json(input): Json<AnalystEstimateIn>,
) -> ApiResult<Json<AnalystEstimateOut>> {
    let row = plutus_storage::queries::analyst::insert_estimate(
        &state.db,
        plutus_storage::queries::analyst::NewEstimate {
            stock_id: input.stock_id,
            metric: &input.metric,
            fiscal_year: input.fiscal_year,
            fiscal_period: &input.fiscal_period,
            consensus_mean: input.consensus_mean,
            consensus_median: input.consensus_median,
            estimate_low: input.estimate_low,
            estimate_high: input.estimate_high,
            num_analysts: input.num_analysts,
            as_of_date: &input.as_of_date,
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn list_ratings(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
    Query(p): Query<PaginationFilter>,
) -> ApiResult<axum::response::Response> {
    let limit = clamp_limit(p.limit)?;
    let offset = clamp_offset(p.offset)?;
    let rows = plutus_storage::queries::analyst::list_ratings_for_stock(
        &state.db, stock_id, limit, offset,
    )
    .await?;
    let body: Vec<AnalystRatingOut> = rows.into_iter().map(Into::into).collect();
    if p.is_paginating() {
        let total =
            plutus_storage::queries::analyst::count_ratings_for_stock(&state.db, stock_id)
                .await?;
        Ok((paginated_response_headers(total), Json(body)).into_response())
    } else {
        Ok(Json(body).into_response())
    }
}

pub async fn delete_rating(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::analyst::delete_rating(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn insert_rating(
    State(state): State<AppState>,
    Json(input): Json<AnalystRatingIn>,
) -> ApiResult<Json<AnalystRatingOut>> {
    let row = plutus_storage::queries::analyst::insert_rating(
        &state.db,
        plutus_storage::queries::analyst::NewRating {
            stock_id: input.stock_id,
            firm: &input.firm,
            analyst_name: input.analyst_name.as_deref(),
            rating: &input.rating,
            rating_action: &input.rating_action,
            previous_rating: input.previous_rating.as_deref(),
            target_price: input.target_price,
            target_currency: input.target_currency.as_deref(),
            previous_target: input.previous_target,
            rated_at: &input.rated_at,
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn batch_estimates(
    State(state): State<AppState>,
    Json(input): Json<AnalystEstimateBatchIn>,
) -> ApiResult<Json<AnalystEstimateBatchOut>> {
    validate_batch_size(input.items.len())?;
    let news: Vec<plutus_storage::queries::analyst::NewEstimate<'_>> = input
        .items
        .iter()
        .map(|i| plutus_storage::queries::analyst::NewEstimate {
            stock_id: i.stock_id,
            metric: &i.metric,
            fiscal_year: i.fiscal_year,
            fiscal_period: &i.fiscal_period,
            consensus_mean: i.consensus_mean,
            consensus_median: i.consensus_median,
            estimate_low: i.estimate_low,
            estimate_high: i.estimate_high,
            num_analysts: i.num_analysts,
            as_of_date: &i.as_of_date,
            source: &i.source,
        })
        .collect();
    let rows =
        plutus_storage::queries::analyst::batch_insert_estimates(&state.db, &news).await?;
    Ok(Json(AnalystEstimateBatchOut {
        count: rows.len(),
        items: rows.into_iter().map(Into::into).collect(),
    }))
}

pub async fn batch_ratings(
    State(state): State<AppState>,
    Json(input): Json<AnalystRatingBatchIn>,
) -> ApiResult<Json<AnalystRatingBatchOut>> {
    validate_batch_size(input.items.len())?;
    let news: Vec<plutus_storage::queries::analyst::NewRating<'_>> = input
        .items
        .iter()
        .map(|i| plutus_storage::queries::analyst::NewRating {
            stock_id: i.stock_id,
            firm: &i.firm,
            analyst_name: i.analyst_name.as_deref(),
            rating: &i.rating,
            rating_action: &i.rating_action,
            previous_rating: i.previous_rating.as_deref(),
            target_price: i.target_price,
            target_currency: i.target_currency.as_deref(),
            previous_target: i.previous_target,
            rated_at: &i.rated_at,
            source: &i.source,
        })
        .collect();
    let rows =
        plutus_storage::queries::analyst::batch_insert_ratings(&state.db, &news).await?;
    Ok(Json(AnalystRatingBatchOut {
        count: rows.len(),
        items: rows.into_iter().map(Into::into).collect(),
    }))
}
