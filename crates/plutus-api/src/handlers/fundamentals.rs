use axum::extract::{Path, State};
use axum::Json;

use crate::dto::fundamentals::{FundamentalsIn, FundamentalsOut};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<FundamentalsOut>>> {
    let rows = plutus_storage::queries::fundamentals::list_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert(
    State(state): State<AppState>,
    Json(input): Json<FundamentalsIn>,
) -> ApiResult<Json<FundamentalsOut>> {
    let row = plutus_storage::queries::fundamentals::insert(
        &state.db,
        plutus_storage::queries::fundamentals::NewFundamentals {
            stock_id: input.stock_id,
            fiscal_year: input.fiscal_year,
            fiscal_period: &input.fiscal_period,
            period_end: &input.period_end,
            currency: &input.currency,
            revenue: input.revenue,
            gross_profit: input.gross_profit,
            operating_income: input.operating_income,
            net_income: input.net_income,
            eps_basic: input.eps_basic,
            eps_diluted: input.eps_diluted,
            cash: input.cash,
            total_debt: input.total_debt,
            total_equity: input.total_equity,
            operating_cf: input.operating_cf,
            free_cf: input.free_cf,
            shares_outstanding: input.shares_outstanding,
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
