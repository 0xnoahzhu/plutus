use axum::extract::State;
use axum::Json;

use crate::dto::fx::{FxIn, FxOut};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<FxOut>>> {
    let rows = plutus_storage::queries::fx::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert(
    State(state): State<AppState>,
    Json(input): Json<FxIn>,
) -> ApiResult<Json<FxOut>> {
    let row = plutus_storage::queries::fx::insert(
        &state.db,
        &input.base_currency,
        &input.quote_currency,
        &input.rate_date,
        input.rate,
        &input.source,
    )
    .await?;
    Ok(Json(row.into()))
}
