use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::catalyst::{CatalystIn, CatalystOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::i18n::LocaleQuery;
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

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<CatalystOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::catalysts::list(
        &state.db,
        plutus_storage::queries::catalysts::ListFilter {
            user_id,
            locale: &l.locale,
            stock_id: f.stock_id,
            sector_code: f.sector_code.as_deref(),
            country: f.country.as_deref(),
            catalyst_kind: f.catalyst_kind.as_deref(),
            status: f.status.as_deref(),
            impact_level: f.impact_level.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn list_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<CatalystOut>>> {
    let user_id = require_user(&actor.0)?;
    let rows = plutus_storage::queries::catalysts::list_for_stock(
        &state.db,
        user_id,
        &l.locale,
        stock_id,
    )
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<CatalystOut>> {
    let user_id = require_user(&actor.0)?;
    let row =
        plutus_storage::queries::catalysts::get(&state.db, user_id, &l.locale, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<CatalystIn>,
) -> ApiResult<Json<CatalystOut>> {
    let user_id = require_user(&actor.0)?;
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    let row = plutus_storage::queries::catalysts::create(
        &state.db,
        plutus_storage::queries::catalysts::NewCatalyst {
            user_id,
            stock_id: input.stock_id,
            sector_code: input.sector_code.as_deref(),
            country: input.country.as_deref(),
            catalyst_kind: &input.catalyst_kind,
            catalyst_date: &input.catalyst_date,
            date_confidence: &input.date_confidence,
            impact_level: &input.impact_level,
            status: &input.status,
            url: input.url.as_deref(),
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
    plutus_storage::queries::catalysts::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
