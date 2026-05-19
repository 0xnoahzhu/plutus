use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::dto::self_exam::{SelfExamIn, SelfExamOut};
use crate::error::{ApiError, ApiResult};
use crate::i18n::{apply_overrides, LocaleQuery};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub kind: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn localize(out: &mut SelfExamOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<SelfExamOut>>> {
    let mut rows = plutus_storage::queries::self_exams::list(
        &state.db,
        plutus_storage::queries::self_exams::ListFilter {
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    rows.sort_by(|a, b| b.period_start.cmp(&a.period_start));
    let mut out: Vec<SelfExamOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<SelfExamOut>> {
    let row = plutus_storage::queries::self_exams::get(&state.db, id).await?;
    let mut out: SelfExamOut = row.into();
    localize(&mut out, &l.locale);
    Ok(Json(out))
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(input): Json<SelfExamIn>,
) -> ApiResult<Json<SelfExamOut>> {
    let metrics = match input.metrics {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("metrics: {e}")))?),
        None => None,
    };
    let rec_ids = input
        .recommendation_ids
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".into()));
    let translations = match input.translations {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("translations: {e}")))?),
        None => None,
    };
    let row = plutus_storage::queries::self_exams::upsert(
        &state.db,
        plutus_storage::queries::self_exams::NewExam {
            kind: &input.kind,
            period_start: &input.period_start,
            period_end: &input.period_end,
            headline: &input.headline,
            content_md: input.content_md.as_deref(),
            metrics: metrics.as_deref(),
            recommendation_ids: rec_ids.as_deref(),
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
    plutus_storage::queries::self_exams::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
