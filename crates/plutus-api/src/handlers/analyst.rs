use axum::extract::{Path, State};
use axum::Json;

use crate::dto::analyst::{
    AnalystEstimateIn, AnalystEstimateOut, AnalystRatingIn, AnalystRatingOut,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_estimates(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<AnalystEstimateOut>>> {
    let rows =
        plutus_storage::queries::analyst::list_estimates_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
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
) -> ApiResult<Json<Vec<AnalystRatingOut>>> {
    let rows =
        plutus_storage::queries::analyst::list_ratings_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
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
