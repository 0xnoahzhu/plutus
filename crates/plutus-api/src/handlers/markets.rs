use axum::extract::State;
use axum::Json;

use crate::dto::market::MarketOut;
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<MarketOut>>> {
    let rows = plutus_storage::queries::markets::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
