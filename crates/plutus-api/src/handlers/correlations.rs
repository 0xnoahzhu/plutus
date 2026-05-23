use axum::extract::{Path, Query, State};
use axum::Json;

use plutus_core::audit::Actor;
use plutus_storage::queries::unread::{self, EntityKind};

use crate::dto::correlation::{
    CorrelationPairIn, CorrelationPairOut, CorrelationRunIn, CorrelationRunOut, UniverseIn,
    UniverseOut,
};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::i18n::LocaleQuery;
use crate::state::AppState;

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

/// Delete a universe definition. Returns 409 if any correlation_run still
/// references it — the caller must delete those runs first. Per-user
/// scoped: deleting someone else's universe returns 404, not 409.
pub async fn delete_universe(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let user_id = require_user(&actor.0)?;
    plutus_storage::queries::correlations::delete_universe(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Runs ──────────────────────────────────────────────────────────────────

pub async fn list_runs(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<CorrelationRunOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows =
        plutus_storage::queries::correlations::list_runs(&state.db, user_id, &l.locale).await?;
    let mut body: Vec<CorrelationRunOut> = rows.into_iter().map(Into::into).collect();
    let ids: Vec<i64> = body.iter().map(|r| r.id).collect();
    let read_ats =
        unread::read_ats(&state.db, user_id, EntityKind::CorrelationRun, &ids).await?;
    for r in &mut body {
        r.read_at = read_ats.get(&r.id).map(jiff::Timestamp::to_string);
    }
    Ok(Json(body))
}

pub async fn get_run(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<CorrelationRunOut>> {
    let user_id = require_user(&actor.0)?;
    let row =
        plutus_storage::queries::correlations::get_run(&state.db, user_id, &l.locale, id).await?;
    unread::mark_read(&state.db, user_id, EntityKind::CorrelationRun, id).await?;
    let mut out: CorrelationRunOut = row.into();
    out.read_at = Some(jiff::Timestamp::now().to_string());
    Ok(Json(out))
}

pub async fn create_run(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<CorrelationRunIn>,
) -> ApiResult<Json<CorrelationRunOut>> {
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
    let row = plutus_storage::queries::correlations::create_run(
        &state.db,
        plutus_storage::queries::correlations::NewRun {
            user_id,
            kind: &input.kind,
            run_date: &input.run_date,
            universe_id: input.universe_id,
            lookback_days: input.lookback_days,
            method: &input.method,
            metrics: metrics.as_deref(),
            source: &input.source,
            content: input.content,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

/// Delete a correlation_run and all its pairs in one transaction. Use to
/// clean up after an obsolete run (re-ran with different parameters) or
/// to free a universe before deleting it.
pub async fn delete_run(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let user_id = require_user(&actor.0)?;
    plutus_storage::queries::correlations::delete_run(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Pairs ─────────────────────────────────────────────────────────────────

pub async fn list_pairs(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(run_id): Path<i64>,
) -> ApiResult<Json<Vec<CorrelationPairOut>>> {
    let user_id = require_user(&actor.0)?;
    // Verify run ownership first; otherwise return NotFound.
    plutus_storage::queries::correlations::get_run(&state.db, user_id, "en", run_id).await?;
    let mut rows =
        plutus_storage::queries::correlations::list_pairs(&state.db, user_id, run_id).await?;
    sort_pairs_by_abs_corr(&mut rows);
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn list_pairs_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
) -> ApiResult<Json<Vec<CorrelationPairOut>>> {
    let user_id = require_user(&actor.0)?;
    let mut rows = plutus_storage::queries::correlations::list_pairs_for_stock(
        &state.db, user_id, stock_id,
    )
    .await?;
    sort_pairs_by_abs_corr(&mut rows);
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

/// Sort pairs by `|correlation|` descending so the strongest signals
/// (positive or negative) lead. `id` is the deterministic tie-breaker
/// for identical magnitudes. Toasty's order_by builder can't express
/// `ABS()` today; if the pair count ever outgrows in-memory sort,
/// switch list_pairs to raw SQL with `ORDER BY ABS(correlation) DESC`.
fn sort_pairs_by_abs_corr(rows: &mut [plutus_storage::models::CorrelationPair]) {
    rows.sort_by(|a, b| {
        b.correlation
            .abs()
            .cmp(&a.correlation.abs())
            .then_with(|| a.id.cmp(&b.id))
    });
}

pub async fn insert_pair(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(run_id): Path<i64>,
    Json(input): Json<CorrelationPairIn>,
) -> ApiResult<Json<CorrelationPairOut>> {
    let user_id = require_user(&actor.0)?;
    // Verify run ownership before inserting a pair under it.
    plutus_storage::queries::correlations::get_run(&state.db, user_id, "en", run_id).await?;
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
