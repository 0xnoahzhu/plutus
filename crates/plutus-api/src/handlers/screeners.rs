use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::screener::{ScreenerHitIn, ScreenerHitOut, ScreenerRunIn, ScreenerRunOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::i18n::{apply_overrides, LocaleQuery};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn localize_run(out: &mut ScreenerRunOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

fn localize_hit(out: &mut ScreenerHitOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

pub async fn list_runs(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<ScreenerRunOut>>> {
    let user_id = require_user(&actor.0)?;
    let mut rows = plutus_storage::queries::screeners::list_runs(
        &state.db,
        plutus_storage::queries::screeners::ListFilter {
            user_id,
            name: f.name.as_deref(),
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    rows.sort_by(|a, b| b.run_date.cmp(&a.run_date).then(a.name.cmp(&b.name)));
    let mut out: Vec<ScreenerRunOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize_run(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn get_run(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<ScreenerRunOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::screeners::get_run(&state.db, user_id, id).await?;
    let mut out: ScreenerRunOut = row.into();
    localize_run(&mut out, &l.locale);
    Ok(Json(out))
}

pub async fn upsert_run(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<ScreenerRunIn>,
) -> ApiResult<Json<ScreenerRunOut>> {
    let user_id = require_user(&actor.0)?;
    let criteria = match input.criteria {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("criteria: {e}")))?),
        None => None,
    };
    let translations = match input.translations {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("translations: {e}")))?),
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
            description_md: input.description_md.as_deref(),
            summary_md: input.summary_md.as_deref(),
            sentiment: input.sentiment.as_deref(),
            language: &input.language,
            source: &input.source,
            translations: translations.as_deref(),
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
        plutus_storage::queries::screeners::list_hits(&state.db, user_id, run_id).await?;
    let mut out: Vec<ScreenerHitOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize_hit(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn list_hits_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<ScreenerHitOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows =
        plutus_storage::queries::screeners::list_hits_for_stock(&state.db, user_id, stock_id)
            .await?;
    let mut out: Vec<ScreenerHitOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize_hit(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn insert_hit(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(run_id): Path<i64>,
    Json(input): Json<ScreenerHitIn>,
) -> ApiResult<Json<ScreenerHitOut>> {
    let user_id = require_user(&actor.0)?;
    // Verify the parent run belongs to this user before inserting a hit under it.
    plutus_storage::queries::screeners::get_run(&state.db, user_id, run_id).await?;
    let metrics = match input.metrics {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("metrics: {e}")))?),
        None => None,
    };
    let translations = match input.translations {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("translations: {e}")))?),
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
            rationale_md: input.rationale_md.as_deref(),
            metrics: metrics.as_deref(),
            translations: translations.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}
