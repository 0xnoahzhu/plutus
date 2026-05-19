use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::dto::transaction::{TransactionIn, TransactionOut};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListFilter {
    pub account_id: Option<i64>,
    pub stock_id: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<ListFilter>,
) -> ApiResult<Json<Vec<TransactionOut>>> {
    let rows = if let Some(account_id) = f.account_id {
        plutus_storage::queries::transactions::list_for_account(&state.db, account_id).await?
    } else if let Some(stock_id) = f.stock_id {
        plutus_storage::queries::transactions::list_for_stock(&state.db, stock_id).await?
    } else {
        plutus_storage::queries::transactions::list(&state.db).await?
    };
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<TransactionOut>> {
    let row = plutus_storage::queries::transactions::get(&state.db, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<TransactionIn>,
) -> ApiResult<Json<TransactionOut>> {
    let executed_at: jiff::Timestamp = input
        .executed_at
        .parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("executed_at: {e}")))?;
    let metadata_str = match input.source_metadata {
        Some(v) => Some(
            serde_json::to_string(&v)
                .map_err(|e| ApiError::BadRequest(format!("source_metadata: {e}")))?,
        ),
        None => None,
    };
    let row = plutus_storage::queries::transactions::create(
        &state.db,
        plutus_storage::queries::transactions::NewTransaction {
            account_id: input.account_id,
            stock_id: input.stock_id,
            kind: &input.kind,
            executed_at,
            quantity: input.quantity,
            price: input.price,
            trade_currency: &input.trade_currency,
            commission: input.commission,
            commission_currency: &input.commission_currency,
            tax: input.tax,
            tax_currency: &input.tax_currency,
            fx_rate_to_base: input.fx_rate_to_base,
            external_ref: input.external_ref.as_deref(),
            notes: input.notes.as_deref(),
            source: &input.source,
            source_metadata: metadata_str.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::transactions::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
