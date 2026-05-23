use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;
use plutus_storage::queries::unread::{self, EntityKind};

use crate::dto::portfolio_review::{PortfolioReviewIn, PortfolioReviewOut};
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
) -> ApiResult<Json<Vec<PortfolioReviewOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::portfolio_reviews::list(
        &state.db,
        plutus_storage::queries::portfolio_reviews::ListFilter {
            user_id,
            locale: &l.locale,
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    let mut body: Vec<PortfolioReviewOut> =
        rows.into_iter().map(Into::into).collect();
    let ids: Vec<i64> = body.iter().map(|r| r.id).collect();
    let read_ats =
        unread::read_ats(&state.db, user_id, EntityKind::PortfolioReview, &ids).await?;
    for r in &mut body {
        r.read_at = read_ats.get(&r.id).map(jiff::Timestamp::to_string);
    }
    Ok(Json(body))
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<PortfolioReviewOut>> {
    let user_id = require_user(&actor.0)?;
    let row =
        plutus_storage::queries::portfolio_reviews::get(&state.db, user_id, &l.locale, id).await?;
    unread::mark_read(&state.db, user_id, EntityKind::PortfolioReview, id).await?;
    let mut out: PortfolioReviewOut = row.into();
    out.read_at = Some(jiff::Timestamp::now().to_string());
    Ok(Json(out))
}

pub async fn upsert(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<PortfolioReviewIn>,
) -> ApiResult<Json<PortfolioReviewOut>> {
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
    let row = plutus_storage::queries::portfolio_reviews::upsert(
        &state.db,
        plutus_storage::queries::portfolio_reviews::NewReview {
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
    plutus_storage::queries::portfolio_reviews::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
