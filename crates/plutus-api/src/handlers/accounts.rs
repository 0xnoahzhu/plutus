use axum::extract::{Path, State};
use axum::Json;

use crate::dto::account::{AccountIn, AccountOut};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<AccountOut>>> {
    let rows = plutus_storage::queries::accounts::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<AccountOut>> {
    let row = plutus_storage::queries::accounts::get(&state.db, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<AccountIn>,
) -> ApiResult<Json<AccountOut>> {
    let row = plutus_storage::queries::accounts::create(
        &state.db,
        plutus_storage::queries::accounts::NewAccount {
            broker_id: input.broker_id,
            name: &input.name,
            account_number: input.account_number.as_deref(),
            base_currency: &input.base_currency,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
