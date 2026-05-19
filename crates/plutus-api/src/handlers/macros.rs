use axum::extract::{Path, State};
use axum::Json;

use crate::dto::macros::{
    MacroIndicatorIn, MacroIndicatorOut, MacroObservationIn, MacroObservationOut,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_indicators(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<MacroIndicatorOut>>> {
    let rows = plutus_storage::queries::macros::list_indicators(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn upsert_indicator(
    State(state): State<AppState>,
    Json(input): Json<MacroIndicatorIn>,
) -> ApiResult<Json<MacroIndicatorOut>> {
    let row = plutus_storage::queries::macros::upsert_indicator(
        &state.db,
        plutus_storage::queries::macros::NewIndicator {
            code: &input.code,
            name: &input.name,
            country: &input.country,
            unit: &input.unit,
            frequency: &input.frequency,
            source: &input.source,
            description: input.description.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn list_observations(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> ApiResult<Json<Vec<MacroObservationOut>>> {
    let rows = plutus_storage::queries::macros::list_observations(&state.db, &code).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert_observation(
    State(state): State<AppState>,
    Json(input): Json<MacroObservationIn>,
) -> ApiResult<Json<MacroObservationOut>> {
    let row = plutus_storage::queries::macros::insert_observation(
        &state.db,
        plutus_storage::queries::macros::NewObservation {
            indicator_code: &input.indicator_code,
            obs_date: &input.obs_date,
            value: input.value,
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
