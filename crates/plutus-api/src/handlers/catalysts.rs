use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::catalyst::{CatalystBatchIn, CatalystBatchOut, CatalystIn, CatalystOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::handlers::batch::{validate_batch_size, MAX_BATCH};
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

/// All-or-nothing batch create. Validates every item up front (JSONB
/// content shape, batch size), then forwards to the storage layer which
/// runs the whole insert in one transaction. If any row fails (DB
/// constraint, malformed content) nothing persists. Each row upserts
/// against `catalysts_natural_key`, so a re-run with the same source
/// refreshes existing rows instead of duplicating.
pub async fn batch_create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<CatalystBatchIn>,
) -> ApiResult<Json<CatalystBatchOut>> {
    let user_id = require_user(&actor.0)?;
    validate_batch_size(input.items.len())?;
    // Validate every content blob before opening the transaction so a
    // single bad item fails the whole request cheaply.
    for (i, item) in input.items.iter().enumerate() {
        if !item.content.is_object() {
            return Err(ApiError::BadRequest(format!(
                "items[{i}].content must be a JSON object keyed by locale"
            )));
        }
    }
    let news: Vec<plutus_storage::queries::catalysts::NewCatalyst<'_>> = input
        .items
        .iter()
        .map(|i| plutus_storage::queries::catalysts::NewCatalyst {
            user_id,
            stock_id: i.stock_id,
            sector_code: i.sector_code.as_deref(),
            country: i.country.as_deref(),
            catalyst_kind: &i.catalyst_kind,
            catalyst_date: &i.catalyst_date,
            date_confidence: &i.date_confidence,
            impact_level: &i.impact_level,
            status: &i.status,
            url: i.url.as_deref(),
            source: &i.source,
            content: i.content.clone(),
        })
        .collect();
    let rows = plutus_storage::queries::catalysts::batch_create(&state.db, news).await?;
    Ok(Json(CatalystBatchOut {
        count: rows.len(),
        items: rows.into_iter().map(Into::into).collect(),
    }))
}

// Silence unused-import warning for MAX_BATCH while keeping it
// exported from the helper module (other handlers will reference it
// when their batch endpoints land).
#[allow(dead_code)]
const _USE_MAX_BATCH: usize = MAX_BATCH;
