use axum::extract::State;
use axum::Json;

use crate::dto::sector::{SectorIn, SectorOut};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<SectorOut>>> {
    let rows = plutus_storage::queries::sectors::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(input): Json<SectorIn>,
) -> ApiResult<Json<SectorOut>> {
    let row = plutus_storage::queries::sectors::upsert(
        &state.db,
        &input.code,
        &input.name,
        input.parent_code.as_deref(),
        &input.scheme,
    )
    .await?;
    Ok(Json(row.into()))
}
