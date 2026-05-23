use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use plutus_core::audit::Actor;
use plutus_storage::queries::unread::{self, EntityKind};

use crate::dto::earnings::{EarningsBatchIn, EarningsBatchOut, EarningsIn, EarningsOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::maybe_user_id;
use crate::handlers::batch::validate_batch_size;
use crate::handlers::pagination::{
    clamp_limit, clamp_offset, paginate_slice, paginated_response_headers, PaginationFilter,
};
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
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
    Query(p): Query<PaginationFilter>,
) -> ApiResult<axum::response::Response> {
    let plimit = clamp_limit(p.limit)?;
    let poffset = clamp_offset(p.offset)?;
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
        // Stock translatable text is not needed here — we only consult
        // (id, market_code) for the country filter. Pass "en" so the
        // projection picks the default locale without an extra hop.
        let stocks = plutus_storage::queries::stocks::list(
            &state.db,
            "en",
            plutus_storage::queries::stocks::ListFilter {
                symbol: None,
                q: None,
                sector_code: None,
                ids: None,
                limit: None,
                offset: None,
            },
        )
        .await?;
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
    let total = rows.len() as i64;
    let page_slice = paginate_slice(rows, plimit, poffset);
    let mut body: Vec<EarningsOut> = page_slice.into_iter().map(Into::into).collect();
    if let Some(user_id) = maybe_user_id(&actor.0) {
        let ids: Vec<i64> = body.iter().map(|e| e.id).collect();
        let read_ats =
            unread::read_ats(&state.db, user_id, EntityKind::EarningsEvent, &ids).await?;
        for e in &mut body {
            e.read_at = read_ats.get(&e.id).map(jiff::Timestamp::to_string);
        }
    }
    if p.is_paginating() {
        Ok((paginated_response_headers(total), Json(body)).into_response())
    } else {
        Ok(Json(body).into_response())
    }
}

pub async fn list_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
    Query(p): Query<PaginationFilter>,
) -> ApiResult<axum::response::Response> {
    let limit = clamp_limit(p.limit)?;
    let offset = crate::handlers::pagination::clamp_offset(p.offset)?;
    let rows = plutus_storage::queries::earnings::list_for_stock(
        &state.db, stock_id, limit, offset,
    )
    .await?;
    let body: Vec<EarningsOut> = rows.into_iter().map(Into::into).collect();
    if p.is_paginating() {
        let total =
            plutus_storage::queries::earnings::count_for_stock(&state.db, stock_id).await?;
        Ok((paginated_response_headers(total), Json(body)).into_response())
    } else {
        Ok(Json(body).into_response())
    }
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<Json<EarningsOut>> {
    let row = plutus_storage::queries::earnings::get(&state.db, id).await?;
    let mut out: EarningsOut = row.into();
    if let Some(user_id) = maybe_user_id(&actor.0) {
        unread::mark_read(&state.db, user_id, EntityKind::EarningsEvent, id).await?;
        out.read_at = Some(jiff::Timestamp::now().to_string());
    }
    Ok(Json(out))
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

/// Batch upsert. Each item conflicts on the natural key
/// (stock_id, fiscal_year, fiscal_period); the whole batch is one tx.
pub async fn batch_upsert(
    State(state): State<AppState>,
    Json(input): Json<EarningsBatchIn>,
) -> ApiResult<Json<EarningsBatchOut>> {
    validate_batch_size(input.items.len())?;
    // Pre-parse announce_at strings so a malformed timestamp at item N
    // doesn't poison the rest of the request.
    let parsed_announce_at: Vec<Option<jiff::Timestamp>> = input
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| match item.announce_at.as_deref() {
            Some(s) => s
                .parse::<jiff::Timestamp>()
                .map(Some)
                .map_err(|e| ApiError::BadRequest(format!("items[{i}].announce_at: {e}"))),
            None => Ok(None),
        })
        .collect::<ApiResult<_>>()?;
    let news: Vec<plutus_storage::queries::earnings::NewEarnings<'_>> = input
        .items
        .iter()
        .zip(parsed_announce_at.iter())
        .map(|(item, announce_at)| plutus_storage::queries::earnings::NewEarnings {
            stock_id: item.stock_id,
            fiscal_year: item.fiscal_year,
            fiscal_period: &item.fiscal_period,
            announce_at: *announce_at,
            announce_date: &item.announce_date,
            announce_timing: &item.announce_timing,
            status: &item.status,
            eps_estimate: item.eps_estimate,
            eps_actual: item.eps_actual,
            revenue_estimate: item.revenue_estimate,
            revenue_actual: item.revenue_actual,
            currency: item.currency.as_deref(),
            guidance_md: item.guidance_md.as_deref(),
            notes: item.notes.as_deref(),
            url: item.url.as_deref(),
            source: &item.source,
        })
        .collect();
    let rows = plutus_storage::queries::earnings::batch_upsert(&state.db, news).await?;
    Ok(Json(EarningsBatchOut {
        count: rows.len(),
        items: rows.into_iter().map(Into::into).collect(),
    }))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::earnings::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
