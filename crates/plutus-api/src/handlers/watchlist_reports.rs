use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::watchlist_report::{WatchlistReportIn, WatchlistReportOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
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
) -> ApiResult<Json<Vec<WatchlistReportOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::watchlist_reports::list(
        &state.db,
        plutus_storage::queries::watchlist_reports::ListFilter {
            user_id,
            locale: &l.locale,
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
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
) -> ApiResult<Json<WatchlistReportOut>> {
    let user_id = require_user(&actor.0)?;
    let row =
        plutus_storage::queries::watchlist_reports::get(&state.db, user_id, &l.locale, id).await?;
    Ok(Json(row.into()))
}

pub async fn upsert(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<WatchlistReportIn>,
) -> ApiResult<Json<WatchlistReportOut>> {
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
    let row = plutus_storage::queries::watchlist_reports::upsert(
        &state.db,
        plutus_storage::queries::watchlist_reports::NewReport {
            user_id,
            kind: &input.kind,
            period_start: &input.period_start,
            period_end: &input.period_end,
            sentiment: input.sentiment.as_deref(),
            sentiment_score: input.sentiment_score,
            metrics: metrics.as_deref(),
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
    plutus_storage::queries::watchlist_reports::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
