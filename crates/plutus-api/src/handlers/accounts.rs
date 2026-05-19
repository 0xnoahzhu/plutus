use axum::extract::{Path, State};
use axum::Json;

use plutus_core::audit::Actor;

use crate::dto::account::{AccountIn, AccountOut};
use crate::error::ApiResult;
use crate::handlers::access::require_user;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> ApiResult<Json<Vec<AccountOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::accounts::list(&state.db, user_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<Json<AccountOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::accounts::get(&state.db, user_id, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<AccountIn>,
) -> ApiResult<Json<AccountOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::accounts::create(
        &state.db,
        plutus_storage::queries::accounts::NewAccount {
            user_id,
            broker_id: input.broker_id,
            name: &input.name,
            account_number: input.account_number.as_deref(),
            base_currency: &input.base_currency,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
