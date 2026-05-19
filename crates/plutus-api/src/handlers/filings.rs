use axum::extract::{Path, State};
use axum::Json;

use crate::dto::filing::{FilingIn, FilingOut};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub async fn list_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<FilingOut>>> {
    let rows = plutus_storage::queries::filings::list_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<FilingOut>> {
    let row = plutus_storage::queries::filings::get(&state.db, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<FilingIn>,
) -> ApiResult<Json<FilingOut>> {
    let filed_at: jiff::Timestamp = input
        .filed_at
        .parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("filed_at: {e}")))?;
    let row = plutus_storage::queries::filings::create(
        &state.db,
        plutus_storage::queries::filings::NewFiling {
            stock_id: input.stock_id,
            filing_type: &input.filing_type,
            fiscal_year: input.fiscal_year,
            fiscal_period: input.fiscal_period.as_deref(),
            period_end: input.period_end.as_deref(),
            filed_at,
            url: &input.url,
            title: &input.title,
            content_md: input.content_md.as_deref(),
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
