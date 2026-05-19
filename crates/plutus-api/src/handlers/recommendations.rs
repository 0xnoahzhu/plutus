use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::recommendation::{RecommendationClosePatch, RecommendationIn, RecommendationOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::i18n::{apply_overrides, LocaleQuery};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub stock_id: Option<i64>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn localize(out: &mut RecommendationOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<RecommendationOut>>> {
    let user_id = require_user(&actor.0)?;
    let mut rows = plutus_storage::queries::recommendations::list(
        &state.db,
        plutus_storage::queries::recommendations::ListFilter {
            user_id,
            stock_id: f.stock_id,
            status: f.status.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    rows.sort_by(|a, b| b.issued_at.cmp(&a.issued_at));
    let mut out: Vec<RecommendationOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn list_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<RecommendationOut>>> {
    let user_id = require_user(&actor.0)?;
    let mut rows = plutus_storage::queries::recommendations::list(
        &state.db,
        plutus_storage::queries::recommendations::ListFilter {
            user_id,
            stock_id: Some(stock_id),
            status: None,
            from: None,
            to: None,
        },
    )
    .await?;
    rows.sort_by(|a, b| b.issued_at.cmp(&a.issued_at));
    let mut out: Vec<RecommendationOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<RecommendationOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::recommendations::get(&state.db, user_id, id).await?;
    let mut out: RecommendationOut = row.into();
    localize(&mut out, &l.locale);
    Ok(Json(out))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<RecommendationIn>,
) -> ApiResult<Json<RecommendationOut>> {
    let user_id = require_user(&actor.0)?;
    let issued_at = match input.issued_at.as_deref() {
        Some(s) => s
            .parse::<jiff::Timestamp>()
            .map_err(|e| ApiError::BadRequest(format!("issued_at: {e}")))?,
        None => jiff::Timestamp::now(),
    };
    let translations = match input.translations {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("translations: {e}")))?),
        None => None,
    };
    let row = plutus_storage::queries::recommendations::create(
        &state.db,
        plutus_storage::queries::recommendations::NewRecommendation {
            user_id,
            stock_id: input.stock_id,
            sector_code: input.sector_code.as_deref(),
            action: &input.action,
            confidence: input.confidence,
            rationale_md: &input.rationale_md,
            target_price: input.target_price,
            target_currency: input.target_currency.as_deref(),
            target_horizon: &input.target_horizon,
            issued_at,
            language: &input.language,
            source: &input.source,
            translations: translations.as_deref(),
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
