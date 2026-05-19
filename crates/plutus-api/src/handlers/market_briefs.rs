use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::market_brief::{MarketBriefIn, MarketBriefOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::i18n::LocaleQuery;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub country: Option<String>,
    pub kind: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<MarketBriefOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::market_briefs::list(
        &state.db,
        plutus_storage::queries::market_briefs::ListFilter {
            user_id,
            locale: &l.locale,
            country: f.country.as_deref(),
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
) -> ApiResult<Json<MarketBriefOut>> {
    let user_id = require_user(&actor.0)?;
    let row =
        plutus_storage::queries::market_briefs::get(&state.db, user_id, &l.locale, id).await?;
    Ok(Json(row.into()))
}

pub async fn upsert(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<MarketBriefIn>,
) -> ApiResult<Json<MarketBriefOut>> {
    let user_id = require_user(&actor.0)?;
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    let row = plutus_storage::queries::market_briefs::upsert(
        &state.db,
        plutus_storage::queries::market_briefs::NewBrief {
            user_id,
            country: &input.country,
            kind: &input.kind,
            trade_date: &input.trade_date,
            sentiment: input.sentiment.as_deref(),
            sentiment_score: input.sentiment_score,
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
    plutus_storage::queries::market_briefs::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
