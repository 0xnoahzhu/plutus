use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::dto::portfolio_review::{PortfolioReviewIn, PortfolioReviewOut};
use crate::error::{ApiError, ApiResult};
use crate::i18n::{apply_overrides, LocaleQuery};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub kind: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn localize(out: &mut PortfolioReviewOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<PortfolioReviewOut>>> {
    let mut rows = plutus_storage::queries::portfolio_reviews::list(
        &state.db,
        plutus_storage::queries::portfolio_reviews::ListFilter {
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    rows.sort_by(|a, b| b.period_start.cmp(&a.period_start).then(a.kind.cmp(&b.kind)));
    let mut out: Vec<PortfolioReviewOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<PortfolioReviewOut>> {
    let row = plutus_storage::queries::portfolio_reviews::get(&state.db, id).await?;
    let mut out: PortfolioReviewOut = row.into();
    localize(&mut out, &l.locale);
    Ok(Json(out))
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(input): Json<PortfolioReviewIn>,
) -> ApiResult<Json<PortfolioReviewOut>> {
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
    let row = plutus_storage::queries::portfolio_reviews::upsert(
        &state.db,
        plutus_storage::queries::portfolio_reviews::NewReview {
            kind: &input.kind,
            period_start: &input.period_start,
            period_end: &input.period_end,
            headline: &input.headline,
            summary_md: input.summary_md.as_deref(),
            content_md: input.content_md.as_deref(),
            decisions_md: input.decisions_md.as_deref(),
            sentiment: input.sentiment.as_deref(),
            sentiment_score: input.sentiment_score,
            metrics: metrics.as_deref(),
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
    plutus_storage::queries::portfolio_reviews::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
