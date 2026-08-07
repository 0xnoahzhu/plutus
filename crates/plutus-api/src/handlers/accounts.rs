use axum::extract::{Path, State};
use axum::Json;

use plutus_core::audit::Actor;

use crate::dto::account::{AccountIn, AccountOut, AccountPatch};
use crate::error::{ApiError, ApiResult};
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

/// `PATCH /accounts/{id}` — rename, or set the cash anchor.
///
/// The anchor is the answer to "how much cash is actually in here". A
/// ledger recorded from mid-life can't derive that on its own, so the
/// user pins a balance from their broker statement and the rollup adds
/// only the flows after it.
pub async fn update(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(patch): Json<AccountPatch>,
) -> ApiResult<Json<AccountOut>> {
    let user_id = require_user(&actor.0)?;
    if let Some(name) = patch.name.as_deref() {
        if name.trim().is_empty() {
            return Err(ApiError::BadRequest("name must not be empty".into()));
        }
    }
    let cash_as_of = match patch.cash_as_of.as_ref().map(Option::as_deref) {
        Some(Some(s)) => Some(Some(s.parse::<jiff::Timestamp>().map_err(|e| {
            ApiError::BadRequest(format!("cash_as_of: {e}"))
        })?)),
        Some(None) => Some(None),
        None => None,
    };
    // A balance without a timestamp is the one combination that silently
    // produces a wrong number: every trade already in the ledger would be
    // applied on top of a balance that already reflects them. Reject it
    // rather than let the user discover the double-count later.
    if patch.cash_balance.is_some() && cash_as_of.is_none() {
        let existing = plutus_storage::queries::accounts::get(&state.db, user_id, id).await?;
        if existing.cash_as_of.is_none() {
            return Err(ApiError::BadRequest(
                "setting cash_balance needs cash_as_of — without it every \
                 existing transaction is counted on top of a balance that \
                 already includes them. Send the timestamp the balance was \
                 true, or null it explicitly to count the whole ledger."
                    .into(),
            ));
        }
    }
    let row = plutus_storage::queries::accounts::update(
        &state.db,
        user_id,
        id,
        plutus_storage::queries::accounts::AccountPatch {
            name: patch.name.as_deref(),
            cash_balance: patch.cash_balance,
            cash_as_of,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let user_id = require_user(&actor.0)?;
    plutus_storage::queries::accounts::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
