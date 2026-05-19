use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::market_brief::{MarketBriefIn, MarketBriefOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::i18n::{apply_overrides, LocaleQuery};
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
    let mut rows = plutus_storage::queries::market_briefs::list(
        &state.db,
        plutus_storage::queries::market_briefs::ListFilter {
            user_id,
            country: f.country.as_deref(),
            kind: f.kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;
    rows.sort_by(|a, b| {
        b.trade_date
            .cmp(&a.trade_date)
            .then(a.kind.cmp(&b.kind))
            .then(a.country.cmp(&b.country))
    });
    let mut out: Vec<MarketBriefOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        let trans = o.translations.clone();
        apply_overrides(o, trans.as_deref(), &l.locale);
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<MarketBriefOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::market_briefs::get(&state.db, user_id, id).await?;
    let mut out: MarketBriefOut = row.into();
    let trans = out.translations.clone();
    apply_overrides(&mut out, trans.as_deref(), &l.locale);
    Ok(Json(out))
}

pub async fn upsert(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<MarketBriefIn>,
) -> ApiResult<Json<MarketBriefOut>> {
    let user_id = require_user(&actor.0)?;
    let translations = match input.translations {
        Some(v) => Some(
            serde_json::to_string(&v)
                .map_err(|e| ApiError::BadRequest(format!("translations: {e}")))?,
        ),
        None => None,
    };
    let row = plutus_storage::queries::market_briefs::upsert(
        &state.db,
        plutus_storage::queries::market_briefs::NewBrief {
            user_id,
            country: &input.country,
            kind: &input.kind,
            trade_date: &input.trade_date,
            headline: &input.headline,
            content_md: input.content_md.as_deref(),
            sentiment: input.sentiment.as_deref(),
            sentiment_score: input.sentiment_score,
            source: &input.source,
            language: &input.language,
            translations: translations.as_deref(),
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
