use axum::extract::{Path, State};
use axum::Json;

use crate::dto::connect::{ConnectFlowIn, ConnectFlowOut, ConnectHoldingsIn, ConnectHoldingsOut};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_flow(State(state): State<AppState>) -> ApiResult<Json<Vec<ConnectFlowOut>>> {
    let rows = plutus_storage::queries::connect::list_flow(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert_flow(
    State(state): State<AppState>,
    Json(input): Json<ConnectFlowIn>,
) -> ApiResult<Json<ConnectFlowOut>> {
    let row = plutus_storage::queries::connect::insert_flow(
        &state.db,
        plutus_storage::queries::connect::NewFlow {
            market_code: &input.market_code,
            direction: &input.direction,
            flow_date: &input.flow_date,
            net_buy: input.net_buy,
            net_buy_currency: &input.net_buy_currency,
            total_buy: input.total_buy,
            total_sell: input.total_sell,
            quota_balance: input.quota_balance,
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn list_holdings_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<ConnectHoldingsOut>>> {
    let rows =
        plutus_storage::queries::connect::list_holdings_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert_holdings(
    State(state): State<AppState>,
    Json(input): Json<ConnectHoldingsIn>,
) -> ApiResult<Json<ConnectHoldingsOut>> {
    let row = plutus_storage::queries::connect::insert_holdings(
        &state.db,
        plutus_storage::queries::connect::NewHoldings {
            stock_id: input.stock_id,
            direction: &input.direction,
            holding_date: &input.holding_date,
            shares: input.shares,
            value: input.value,
            value_currency: input.value_currency.as_deref(),
            pct_of_float: input.pct_of_float,
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
