use axum::extract::{Path, State};
use axum::Json;

use crate::dto::insider::{InsiderTxnIn, InsiderTxnOut};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub async fn list_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<InsiderTxnOut>>> {
    let rows = plutus_storage::queries::insider::list_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert(
    State(state): State<AppState>,
    Json(input): Json<InsiderTxnIn>,
) -> ApiResult<Json<InsiderTxnOut>> {
    let filed_at: jiff::Timestamp = input
        .filed_at
        .parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("filed_at: {e}")))?;
    let row = plutus_storage::queries::insider::insert(
        &state.db,
        plutus_storage::queries::insider::NewInsiderTxn {
            stock_id: input.stock_id,
            person_name: &input.person_name,
            role: input.role.as_deref(),
            txn_kind: &input.txn_kind,
            shares: input.shares,
            price: input.price,
            currency: input.currency.as_deref(),
            executed_at: &input.executed_at,
            filed_at,
            source: &input.source,
            source_url: input.source_url.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}
