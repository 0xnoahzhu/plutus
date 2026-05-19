use axum::extract::{Path, Query, State};
use axum::Json;

use plutus_core::audit::Actor;

use crate::dto::correlation::{
    CorrelationPairIn, CorrelationPairOut, CorrelationRunIn, CorrelationRunOut, UniverseIn,
    UniverseOut,
};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::i18n::{apply_overrides, LocaleQuery};
use crate::state::AppState;

fn localize_run(out: &mut CorrelationRunOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

// ── Universes ─────────────────────────────────────────────────────────────

pub async fn list_universes(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
) -> ApiResult<Json<Vec<UniverseOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::correlations::list_universes(&state.db, user_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get_universe(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<Json<UniverseOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::correlations::get_universe(&state.db, user_id, id).await?;
    Ok(Json(row.into()))
}

pub async fn upsert_universe(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<UniverseIn>,
) -> ApiResult<Json<UniverseOut>> {
    let user_id = require_user(&actor.0)?;
    let stock_ids_json = serde_json::to_string(&input.stock_ids)
        .map_err(|e| ApiError::BadRequest(format!("stock_ids: {e}")))?;
    let row = plutus_storage::queries::correlations::upsert_universe(
        &state.db,
        user_id,
        &input.name,
        input.description_md.as_deref(),
        &stock_ids_json,
    )
    .await?;
    Ok(Json(row.into()))
}

// ── Runs ──────────────────────────────────────────────────────────────────

pub async fn list_runs(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<CorrelationRunOut>>> {
    let user_id = require_user(&actor.0)?;
    let mut rows = plutus_storage::queries::correlations::list_runs(&state.db, user_id).await?;
    rows.sort_by(|a, b| b.run_date.cmp(&a.run_date));
    let mut out: Vec<CorrelationRunOut> = rows.into_iter().map(Into::into).collect();
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
) -> ApiResult<Json<CorrelationRunOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::correlations::get_run(&state.db, user_id, id).await?;
    let mut out: CorrelationRunOut = row.into();
    localize_run(&mut out, &l.locale);
    Ok(Json(out))
}

pub async fn create_run(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<CorrelationRunIn>,
) -> ApiResult<Json<CorrelationRunOut>> {
    let user_id = require_user(&actor.0)?;
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
    let row = plutus_storage::queries::correlations::create_run(
        &state.db,
        plutus_storage::queries::correlations::NewRun {
            user_id,
            kind: &input.kind,
            run_date: &input.run_date,
            universe_id: input.universe_id,
            lookback_days: input.lookback_days,
            method: &input.method,
            summary_md: input.summary_md.as_deref(),
            metrics: metrics.as_deref(),
            source: &input.source,
            translations: translations.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}

// ── Pairs ─────────────────────────────────────────────────────────────────

pub async fn list_pairs(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(run_id): Path<i64>,
) -> ApiResult<Json<Vec<CorrelationPairOut>>> {
    let user_id = require_user(&actor.0)?;
    // Verify run ownership first; otherwise return NotFound.
    plutus_storage::queries::correlations::get_run(&state.db, user_id, run_id).await?;
    let rows =
        plutus_storage::queries::correlations::list_pairs(&state.db, user_id, run_id).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn list_pairs_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<CorrelationPairOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::correlations::list_pairs_for_stock(
        &state.db, user_id, stock_id,
    )
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn insert_pair(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(run_id): Path<i64>,
    Json(input): Json<CorrelationPairIn>,
) -> ApiResult<Json<CorrelationPairOut>> {
    let user_id = require_user(&actor.0)?;
    // Verify run ownership before inserting a pair under it.
    plutus_storage::queries::correlations::get_run(&state.db, user_id, run_id).await?;
    let row = plutus_storage::queries::correlations::insert_pair(
        &state.db,
        plutus_storage::queries::correlations::NewPair {
            user_id,
            run_id,
            stock_a_id: input.stock_a_id,
            stock_b_id: input.stock_b_id,
            correlation: input.correlation,
        },
    )
    .await?;
    Ok(Json(row.into()))
}
