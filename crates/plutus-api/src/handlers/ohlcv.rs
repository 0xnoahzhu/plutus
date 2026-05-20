use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ohlcv::{OhlcvBatchIn, OhlcvBatchOut, OhlcvIn, OhlcvOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::batch::validate_batch_size;
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
    // Path id is authoritative for the per-stock route; ignore any
    // stock_id in the body to keep this endpoint single-stock.
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

/// Cross-stock bulk insert. Each item must carry its own `stock_id`
/// because there's no path-level disambiguation. One transaction;
/// every row upserts against (stock_id, trade_date), so a repeat
/// nightly backfill refreshes existing bars instead of duplicating.
pub async fn batch_upsert(
    State(state): State<AppState>,
    Json(input): Json<OhlcvBatchIn>,
) -> ApiResult<Json<OhlcvBatchOut>> {
    validate_batch_size(input.items.len())?;
    for (i, item) in input.items.iter().enumerate() {
        if item.stock_id.is_none() {
            return Err(ApiError::BadRequest(format!(
                "items[{i}].stock_id is required in batch mode"
            )));
        }
    }
    let news: Vec<plutus_storage::queries::ohlcv::NewOhlcv<'_>> = input
        .items
        .iter()
        .map(|i| plutus_storage::queries::ohlcv::NewOhlcv {
            stock_id: i.stock_id.expect("validated above"),
            trade_date: &i.trade_date,
            open: i.open,
            high: i.high,
            low: i.low,
            close: i.close,
            adjusted_close: i.adjusted_close,
            volume: i.volume,
            source: &i.source,
        })
        .collect();
    let rows = plutus_storage::queries::ohlcv::batch_upsert(&state.db, &news).await?;
    Ok(Json(OhlcvBatchOut {
        count: rows.len(),
        items: rows.into_iter().map(Into::into).collect(),
    }))
}
