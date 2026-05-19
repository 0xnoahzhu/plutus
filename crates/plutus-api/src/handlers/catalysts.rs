use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::dto::catalyst::{CatalystIn, CatalystOut};
use crate::error::{ApiError, ApiResult};
use crate::i18n::{apply_overrides, LocaleQuery};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub stock_id: Option<i64>,
    pub sector_code: Option<String>,
    pub country: Option<String>,
    pub catalyst_kind: Option<String>,
    pub status: Option<String>,
    pub impact_level: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn localize(out: &mut CatalystOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<CatalystOut>>> {
    let mut rows = plutus_storage::queries::catalysts::list(
        &state.db,
        plutus_storage::queries::catalysts::ListFilter {
            stock_id: f.stock_id,
            sector_code: f.sector_code.as_deref(),
            catalyst_kind: f.catalyst_kind.as_deref(),
            status: f.status.as_deref(),
            impact_level: f.impact_level.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    if let Some(country) = f.country.as_deref() {
        rows.retain(|c| c.country.as_deref() == Some(country));
    }
    rows.sort_by(|a, b| a.catalyst_date.cmp(&b.catalyst_date));
    let mut out: Vec<CatalystOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn list_for_stock(
    State(state): State<AppState>,
    Path(stock_id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<CatalystOut>>> {
    let mut rows = plutus_storage::queries::catalysts::list_for_stock(&state.db, stock_id).await?;
    rows.sort_by(|a, b| a.catalyst_date.cmp(&b.catalyst_date));
    let mut out: Vec<CatalystOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<CatalystOut>> {
    let row = plutus_storage::queries::catalysts::get(&state.db, id).await?;
    let mut out: CatalystOut = row.into();
    localize(&mut out, &l.locale);
    Ok(Json(out))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CatalystIn>,
) -> ApiResult<Json<CatalystOut>> {
    let translations = match input.translations {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("translations: {e}")))?),
        None => None,
    };
    let row = plutus_storage::queries::catalysts::create(
        &state.db,
        plutus_storage::queries::catalysts::NewCatalyst {
            stock_id: input.stock_id,
            sector_code: input.sector_code.as_deref(),
            country: input.country.as_deref(),
            catalyst_kind: &input.catalyst_kind,
            title: &input.title,
            summary_md: input.summary_md.as_deref(),
            catalyst_date: &input.catalyst_date,
            date_confidence: &input.date_confidence,
            impact_level: &input.impact_level,
            bull_case_md: input.bull_case_md.as_deref(),
            bear_case_md: input.bear_case_md.as_deref(),
            status: &input.status,
            notes: input.notes.as_deref(),
            url: input.url.as_deref(),
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
    plutus_storage::queries::catalysts::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
