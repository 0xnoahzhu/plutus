use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::dto::watchlist_report::{WatchlistReportIn, WatchlistReportOut};
use crate::error::{ApiError, ApiResult};
use crate::i18n::{apply_overrides, LocaleQuery};
use crate::state::AppState;

fn localize(out: &mut WatchlistReportOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub watchlist_id: Option<i64>,
    pub kind: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<WatchlistReportOut>>> {
    let mut rows = plutus_storage::queries::watchlist_reports::list(
        &state.db,
        plutus_storage::queries::watchlist_reports::ListFilter {
            watchlist_id: f.watchlist_id,
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    rows.sort_by(|a, b| {
        b.period_start
            .cmp(&a.period_start)
            .then(a.kind.cmp(&b.kind))
            .then(a.watchlist_id.cmp(&b.watchlist_id))
    });
    let mut out: Vec<WatchlistReportOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn list_for_watchlist(
    State(state): State<AppState>,
    Path(watchlist_id): Path<i64>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<WatchlistReportOut>>> {
    let mut rows = plutus_storage::queries::watchlist_reports::list(
        &state.db,
        plutus_storage::queries::watchlist_reports::ListFilter {
            watchlist_id: Some(watchlist_id),
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    rows.sort_by(|a, b| b.period_start.cmp(&a.period_start).then(a.kind.cmp(&b.kind)));
    let mut out: Vec<WatchlistReportOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<WatchlistReportOut>> {
    let row = plutus_storage::queries::watchlist_reports::get(&state.db, id).await?;
    let mut out: WatchlistReportOut = row.into();
    localize(&mut out, &l.locale);
    Ok(Json(out))
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(input): Json<WatchlistReportIn>,
) -> ApiResult<Json<WatchlistReportOut>> {
    let metrics = match input.metrics {
        Some(v) => Some(
            serde_json::to_string(&v)
                .map_err(|e| ApiError::BadRequest(format!("metrics: {e}")))?,
        ),
        None => None,
    };
    let translations = match input.translations {
        Some(v) => Some(
            serde_json::to_string(&v)
                .map_err(|e| ApiError::BadRequest(format!("translations: {e}")))?,
        ),
        None => None,
    };
    let row = plutus_storage::queries::watchlist_reports::upsert(
        &state.db,
        plutus_storage::queries::watchlist_reports::NewReport {
            watchlist_id: input.watchlist_id,
            kind: &input.kind,
            period_start: &input.period_start,
            period_end: &input.period_end,
            headline: &input.headline,
            summary_md: input.summary_md.as_deref(),
            content_md: input.content_md.as_deref(),
            sentiment: input.sentiment.as_deref(),
            sentiment_score: input.sentiment_score,
            metrics: metrics.as_deref(),
            notes: input.notes.as_deref(),
            language: &input.language,
            source: &input.source,
            translations: translations.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::watchlist_reports::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
