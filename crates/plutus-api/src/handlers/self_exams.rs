use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::self_exam::{SelfExamIn, SelfExamOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::handlers::pagination::{
    clamp_limit, clamp_offset, paginate_slice, paginated_response_headers, PaginationFilter,
};
use crate::i18n::LocaleQuery;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub kind: Option<String>,
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
    let rows = plutus_storage::queries::self_exams::list(
        &state.db,
        plutus_storage::queries::self_exams::ListFilter {
            user_id,
            locale: &l.locale,
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    let total = rows.len() as i64;
    let page_slice = paginate_slice(rows, limit, offset);
    let body: Vec<SelfExamOut> = page_slice.into_iter().map(Into::into).collect();
    if p.is_paginating() {
        Ok((paginated_response_headers(total), Json(body)).into_response())
    } else {
        Ok(Json(body).into_response())
    }
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<SelfExamOut>> {
    let user_id = require_user(&actor.0)?;
    let row =
        plutus_storage::queries::self_exams::get(&state.db, user_id, &l.locale, id).await?;
    Ok(Json(row.into()))
}

pub async fn upsert(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<SelfExamIn>,
) -> ApiResult<Json<SelfExamOut>> {
    let user_id = require_user(&actor.0)?;
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    let metrics = match input.metrics {
        Some(v) => Some(
            serde_json::to_string(&v)
                .map_err(|e| ApiError::BadRequest(format!("metrics: {e}")))?,
        ),
        None => None,
    };
    let rec_ids = input
        .recommendation_ids
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".into()));
    let row = plutus_storage::queries::self_exams::upsert(
        &state.db,
        plutus_storage::queries::self_exams::NewExam {
            user_id,
            kind: &input.kind,
            period_start: &input.period_start,
            period_end: &input.period_end,
            metrics: metrics.as_deref(),
            recommendation_ids: rec_ids.as_deref(),
            source: &input.source,
            content: input.content,
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
    plutus_storage::queries::self_exams::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
