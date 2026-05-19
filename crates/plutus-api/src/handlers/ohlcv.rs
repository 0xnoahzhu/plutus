use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ohlcv::{OhlcvIn, OhlcvOut};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<OhlcvOut>>> {
    let rows = plutus_storage::queries::ohlcv::list_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert_one(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
    Json(input): Json<OhlcvIn>,
) -> ApiResult<Json<OhlcvOut>> {
    let row = plutus_storage::queries::ohlcv::insert(
        &state.db,
        plutus_storage::queries::ohlcv::NewOhlcv {
            stock_id,
            trade_date: &input.trade_date,
            open: input.open,
            high: input.high,
            low: input.low,
            close: input.close,
            adjusted_close: input.adjusted_close,
            volume: input.volume,
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
