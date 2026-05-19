use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::screener::{ScreenerHitIn, ScreenerHitOut, ScreenerRunIn, ScreenerRunOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::i18n::LocaleQuery;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

pub async fn list_runs(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<ScreenerRunOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::screeners::list_runs(
        &state.db,
        plutus_storage::queries::screeners::ListFilter {
            user_id,
            locale: &l.locale,
            name: f.name.as_deref(),
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get_run(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<ScreenerRunOut>> {
    let user_id = require_user(&actor.0)?;
    let row =
        plutus_storage::queries::screeners::get_run(&state.db, user_id, &l.locale, id).await?;
    Ok(Json(row.into()))
}

pub async fn upsert_run(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<ScreenerRunIn>,
) -> ApiResult<Json<ScreenerRunOut>> {
    let user_id = require_user(&actor.0)?;
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    let criteria = match input.criteria {
        Some(v) => Some(
            serde_json::to_string(&v)
                .map_err(|e| ApiError::BadRequest(format!("criteria: {e}")))?,
        ),
        None => None,
    };
    let row = plutus_storage::queries::screeners::upsert_run(
        &state.db,
        plutus_storage::queries::screeners::NewRun {
            user_id,
            name: &input.name,
            kind: &input.kind,
            run_date: &input.run_date,
            universe: &input.universe,
            universe_size: input.universe_size,
            criteria: criteria.as_deref(),
            sentiment: input.sentiment.as_deref(),
            source: &input.source,
            content: input.content,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn list_hits(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(run_id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<ScreenerHitOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows =
        plutus_storage::queries::screeners::list_hits(&state.db, user_id, &l.locale, run_id)
            .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn list_hits_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<ScreenerHitOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::screeners::list_hits_for_stock(
        &state.db,
        user_id,
        &l.locale,
        stock_id,
    )
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert_hit(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(run_id): Path<i64>,
    Json(input): Json<ScreenerHitIn>,
) -> ApiResult<Json<ScreenerHitOut>> {
    let user_id = require_user(&actor.0)?;
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    // Verify the parent run belongs to this user before inserting a hit under it.
    plutus_storage::queries::screeners::get_run(&state.db, user_id, "en", run_id).await?;
    let metrics = match input.metrics {
        Some(v) => Some(
            serde_json::to_string(&v)
                .map_err(|e| ApiError::BadRequest(format!("metrics: {e}")))?,
        ),
        None => None,
    };
    let row = plutus_storage::queries::screeners::insert_hit(
        &state.db,
        plutus_storage::queries::screeners::NewHit {
            user_id,
            run_id,
            stock_id: input.stock_id,
            rank: input.rank,
            score: input.score,
            metrics: metrics.as_deref(),
            content: input.content,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
