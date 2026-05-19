use axum::extract::State;
use axum::Json;

use crate::dto::broker::BrokerOut;
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<BrokerOut>>> {
    let rows = plutus_storage::queries::brokers::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
