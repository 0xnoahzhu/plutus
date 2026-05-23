use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::recommendation::{RecommendationClosePatch, RecommendationIn, RecommendationOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::handlers::pagination::{
    clamp_limit, clamp_offset, paginate_slice, paginated_response_headers, PaginationFilter,
};
use crate::i18n::LocaleQuery;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub stock_id: Option<i64>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
    Query(p): Query<PaginationFilter>,
) -> ApiResult<axum::response::Response> {
    let user_id = require_user(&actor.0)?;
    let limit = clamp_limit(p.limit)?;
    let offset = clamp_offset(p.offset)?;
    let rows = plutus_storage::queries::recommendations::list(
        &state.db,
        plutus_storage::queries::recommendations::ListFilter {
            user_id,
            locale: &l.locale,
            stock_id: f.stock_id,
            status: f.status.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    let total = rows.len() as i64;
    let page_slice = paginate_slice(rows, limit, offset);
    let body: Vec<RecommendationOut> = page_slice.into_iter().map(Into::into).collect();
    if p.is_paginating() {
        Ok((paginated_response_headers(total), Json(body)).into_response())
    } else {
        Ok(Json(body).into_response())
    }
}

pub async fn list_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<RecommendationOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::recommendations::list(
        &state.db,
        plutus_storage::queries::recommendations::ListFilter {
            user_id,
            locale: &l.locale,
            stock_id: Some(stock_id),
            status: None,
            from: None,
            to: None,
        },
    )
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<RecommendationOut>> {
    let user_id = require_user(&actor.0)?;
    let row =
        plutus_storage::queries::recommendations::get(&state.db, user_id, &l.locale, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<RecommendationIn>,
) -> ApiResult<Json<RecommendationOut>> {
    let user_id = require_user(&actor.0)?;
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    let issued_at = match input.issued_at.as_deref() {
        Some(s) => s
            .parse::<jiff::Timestamp>()
            .map_err(|e| ApiError::BadRequest(format!("issued_at: {e}")))?,
        None => jiff::Timestamp::now(),
    };
    let row = plutus_storage::queries::recommendations::create(
        &state.db,
        plutus_storage::queries::recommendations::NewRecommendation {
            user_id,
            stock_id: input.stock_id,
            sector_code: input.sector_code.as_deref(),
            action: &input.action,
            confidence: input.confidence,
            target_price: input.target_price,
            target_currency: input.target_currency.as_deref(),
            target_horizon: &input.target_horizon,
            issued_at,
            source: &input.source,
            content: input.content,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn close(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(patch): Json<RecommendationClosePatch>,
) -> ApiResult<Json<RecommendationOut>> {
    let user_id = require_user(&actor.0)?;
    let closed_at = match patch.closed_at.as_deref() {
        Some(s) => s
            .parse::<jiff::Timestamp>()
            .map_err(|e| ApiError::BadRequest(format!("closed_at: {e}")))?,
        None => jiff::Timestamp::now(),
    };
    let row = plutus_storage::queries::recommendations::close(
        &state.db,
        user_id,
        id,
        plutus_storage::queries::recommendations::ClosePatch {
            status: &patch.status,
            outcome_md: patch.outcome_md.as_deref(),
            pnl_pct: patch.pnl_pct,
            closed_at,
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
    plutus_storage::queries::recommendations::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
