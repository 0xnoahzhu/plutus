use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;

use crate::dto::insider::{InsiderTxnBatchIn, InsiderTxnBatchOut, InsiderTxnIn, InsiderTxnOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::batch::validate_batch_size;
use crate::handlers::pagination::{
    clamp_limit, clamp_offset, paginated_response_headers, PaginationFilter,
};
use crate::state::AppState;

pub async fn list_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
    Query(p): Query<PaginationFilter>,
) -> ApiResult<axum::response::Response> {
    let limit = clamp_limit(p.limit)?;
    let offset = clamp_offset(p.offset)?;
    let rows = plutus_storage::queries::insider::list_for_stock(
        &state.db, stock_id, limit, offset,
    )
    .await?;
    let body: Vec<InsiderTxnOut> = rows.into_iter().map(Into::into).collect();
    if p.is_paginating() {
        let total =
            plutus_storage::queries::insider::count_for_stock(&state.db, stock_id).await?;
        Ok((paginated_response_headers(total), Json(body)).into_response())
    } else {
        Ok(Json(body).into_response())
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::insider::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
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

pub async fn batch_insert(
    State(state): State<AppState>,
    Json(input): Json<InsiderTxnBatchIn>,
) -> ApiResult<Json<InsiderTxnBatchOut>> {
    validate_batch_size(input.items.len())?;
    // Parse + validate every filed_at timestamp up front so a single
    // malformed row fails fast without partial writes.
    let parsed_filed_at: Vec<jiff::Timestamp> = input
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            item.filed_at
                .parse()
                .map_err(|e: jiff::Error| {
                    ApiError::BadRequest(format!("items[{i}].filed_at: {e}"))
                })
        })
        .collect::<Result<_, _>>()?;
    let news: Vec<plutus_storage::queries::insider::NewInsiderTxn<'_>> = input
        .items
        .iter()
        .zip(parsed_filed_at.iter())
        .map(|(item, filed_at)| plutus_storage::queries::insider::NewInsiderTxn {
            stock_id: item.stock_id,
            person_name: &item.person_name,
            role: item.role.as_deref(),
            txn_kind: &item.txn_kind,
            shares: item.shares,
            price: item.price,
            currency: item.currency.as_deref(),
            executed_at: &item.executed_at,
            filed_at: *filed_at,
            source: &item.source,
            source_url: item.source_url.as_deref(),
        })
        .collect();
    let rows = plutus_storage::queries::insider::batch_insert(&state.db, &news).await?;
    Ok(Json(InsiderTxnBatchOut {
        count: rows.len(),
        items: rows.into_iter().map(Into::into).collect(),
    }))
}
