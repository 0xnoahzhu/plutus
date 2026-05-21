use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use std::str::FromStr;

use plutus_core::audit::Actor;
use plutus_core::transaction::TransactionKind;

use crate::dto::transaction::{TransactionIn, TransactionOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListFilter {
    pub account_id: Option<i64>,
    pub stock_id: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
) -> ApiResult<Json<Vec<TransactionOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = if let Some(account_id) = f.account_id {
        plutus_storage::queries::transactions::list_for_account(&state.db, user_id, account_id)
            .await?
    } else if let Some(stock_id) = f.stock_id {
        plutus_storage::queries::transactions::list_for_stock(&state.db, user_id, stock_id).await?
    } else {
        plutus_storage::queries::transactions::list(&state.db, user_id).await?
    };
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<Json<TransactionOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::transactions::get(&state.db, user_id, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<TransactionIn>,
) -> ApiResult<Json<TransactionOut>> {
    let user_id = require_user(&actor.0)?;
    // Validate `kind` against the canonical enum BEFORE writing — silently
    // accepting an unknown kind would store the row but break the holdings
    // rollup (which filters by `TransactionKind::from_str`). The parser is
    // case-insensitive and accepts the `withdraw` alias, so most agent
    // dialects work; anything else is rejected here with a clear 400.
    let canonical_kind = TransactionKind::from_str(&input.kind)
        .map_err(|_| {
            ApiError::BadRequest(format!(
                "kind must be one of BUY, SELL, DIVIDEND, FEE, INTEREST, \
                 DEPOSIT, WITHDRAWAL, FX, CORPORATE_ACTION (case-insensitive); \
                 got {:?}",
                input.kind
            ))
        })?
        .as_str();
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
            user_id,
            account_id: input.account_id,
            stock_id: input.stock_id,
            // Stored in the canonical SCREAMING_SNAKE_CASE form so the
            // database is consistent regardless of how the caller wrote it.
            kind: canonical_kind,
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
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let user_id = require_user(&actor.0)?;
    plutus_storage::queries::transactions::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
