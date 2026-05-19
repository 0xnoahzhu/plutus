use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::dto::earnings::{EarningsIn, EarningsOut};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub stock_id: Option<i64>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub country: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<ListFilter>,
) -> ApiResult<Json<Vec<EarningsOut>>> {
    let mut rows = plutus_storage::queries::earnings::list(
        &state.db,
        plutus_storage::queries::earnings::ListFilter {
            stock_id: f.stock_id,
            status: f.status.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;

    // Country filter, resolved via stocks.market_code → markets.country.
    if let Some(country) = f.country.as_deref() {
        let market_codes: HashSet<String> =
            plutus_storage::queries::markets::list_codes_by_country(&state.db, country)
                .await?
                .into_iter()
                .collect();
        let stocks = plutus_storage::queries::stocks::list(&state.db).await?;
        let stock_market: HashMap<i64, String> = stocks
            .into_iter()
            .map(|s| (s.id, s.market_code))
            .collect();
        rows.retain(|e| {
            stock_market
                .get(&e.stock_id)
                .map_or(false, |m| market_codes.contains(m))
        });
    }

    // Soonest date first (upcoming at top); deterministic by stock_id within same date.
    rows.sort_by(|a, b| {
        a.announce_date
            .cmp(&b.announce_date)
            .then(a.stock_id.cmp(&b.stock_id))
    });
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn list_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<EarningsOut>>> {
    let rows = plutus_storage::queries::earnings::list_for_stock(&state.db, stock_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<EarningsOut>> {
    let row = plutus_storage::queries::earnings::get(&state.db, id).await?;
    Ok(Json(row.into()))
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(input): Json<EarningsIn>,
) -> ApiResult<Json<EarningsOut>> {
    let announce_at = match input.announce_at.as_deref() {
        Some(s) => Some(
            s.parse::<jiff::Timestamp>()
                .map_err(|e| ApiError::BadRequest(format!("announce_at: {e}")))?,
        ),
        None => None,
    };
    let row = plutus_storage::queries::earnings::upsert(
        &state.db,
        plutus_storage::queries::earnings::NewEarnings {
            stock_id: input.stock_id,
            fiscal_year: input.fiscal_year,
            fiscal_period: &input.fiscal_period,
            announce_at,
            announce_date: &input.announce_date,
            announce_timing: &input.announce_timing,
            status: &input.status,
            eps_estimate: input.eps_estimate,
            eps_actual: input.eps_actual,
            revenue_estimate: input.revenue_estimate,
            revenue_actual: input.revenue_actual,
            currency: input.currency.as_deref(),
            guidance_md: input.guidance_md.as_deref(),
            notes: input.notes.as_deref(),
            url: input.url.as_deref(),
            source: &input.source,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::earnings::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
