use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::dto::macro_event::{
    MacroEventBatchIn, MacroEventBatchOut, MacroEventIn, MacroEventOut,
};
use crate::error::{ApiError, ApiResult};
use crate::handlers::batch::validate_batch_size;
use crate::i18n::LocaleQuery;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub indicator_code: Option<String>,
    pub event_kind: Option<String>,
    pub country: Option<String>, // resolved through macro_indicators.country
    pub from: Option<String>,
    pub to: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<MacroEventOut>>> {
    let mut rows = plutus_storage::queries::macro_events::list(
        &state.db,
        plutus_storage::queries::macro_events::ListFilter {
            locale: &l.locale,
            indicator_code: f.indicator_code.as_deref(),
            event_kind: f.event_kind.as_deref(),
            from: f.from.as_deref(),
            to: f.to.as_deref(),
        },
    )
    .await?;

    if let Some(country) = f.country.as_deref() {
        let indicators = plutus_storage::queries::macros::list_indicators(&state.db).await?;
        let indicator_country: HashMap<String, String> = indicators
            .into_iter()
            .map(|i| (i.code, i.country))
            .collect();
        let country_owned = country.to_string();
        rows.retain(|e| {
            indicator_country
                .get(&e.indicator_code)
                .map_or(false, |c| c == &country_owned)
        });
    }

    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<MacroEventOut>> {
    let row = plutus_storage::queries::macro_events::get(&state.db, &l.locale, id).await?;
    Ok(Json(row.into()))
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(input): Json<MacroEventIn>,
) -> ApiResult<Json<MacroEventOut>> {
    if !input.content.is_object() {
        return Err(ApiError::BadRequest(
            "content must be a JSON object keyed by locale".into(),
        ));
    }
    ensure_indicators_known(&state, std::iter::once(input.indicator_code.as_str())).await?;
    let row = plutus_storage::queries::macro_events::upsert(
        &state.db,
        plutus_storage::queries::macro_events::NewMacroEvent {
            indicator_code: &input.indicator_code,
            event_date: &input.event_date,
            event_kind: &input.event_kind,
            decision: input.decision.as_deref(),
            decision_bps: input.decision_bps,
            new_value: input.new_value,
            consensus_estimate: input.consensus_estimate,
            surprise: input.surprise,
            previous_value: input.previous_value,
            vote: input.vote.as_deref(),
            dot_plot: input.dot_plot.as_deref(),
            url: input.url.as_deref(),
            source: &input.source,
            content: input.content,
        },
    )
    .await?;
    Ok(Json(row.into()))
}

/// Batch upsert. Each item conflicts on (indicator_code, event_date);
/// the whole batch is wrapped in one transaction.
pub async fn batch_upsert(
    State(state): State<AppState>,
    Json(input): Json<MacroEventBatchIn>,
) -> ApiResult<Json<MacroEventBatchOut>> {
    validate_batch_size(input.items.len())?;
    for (i, item) in input.items.iter().enumerate() {
        if !item.content.is_object() {
            return Err(ApiError::BadRequest(format!(
                "items[{i}].content must be a JSON object keyed by locale"
            )));
        }
    }
    ensure_indicators_known(
        &state,
        input.items.iter().map(|i| i.indicator_code.as_str()),
    )
    .await?;
    let news: Vec<plutus_storage::queries::macro_events::NewMacroEvent<'_>> = input
        .items
        .iter()
        .map(|item| plutus_storage::queries::macro_events::NewMacroEvent {
            indicator_code: &item.indicator_code,
            event_date: &item.event_date,
            event_kind: &item.event_kind,
            decision: item.decision.as_deref(),
            decision_bps: item.decision_bps,
            new_value: item.new_value,
            consensus_estimate: item.consensus_estimate,
            surprise: item.surprise,
            previous_value: item.previous_value,
            vote: item.vote.as_deref(),
            dot_plot: item.dot_plot.as_deref(),
            url: item.url.as_deref(),
            source: &item.source,
            content: item.content.clone(),
        })
        .collect();
    let rows = plutus_storage::queries::macro_events::batch_upsert(&state.db, news).await?;
    Ok(Json(MacroEventBatchOut {
        count: rows.len(),
        items: rows.into_iter().map(Into::into).collect(),
    }))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    plutus_storage::queries::macro_events::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// App-level FK check: every `indicator_code` on an incoming event
/// must already exist in `macro_indicators`. Without this, an event
/// referencing an unknown code is invisible to the country-filter on
/// /macro-events (the country is resolved via the indicator), so it
/// silently disappears from the UI. A real DB-level FK would be
/// stronger but ships a migration; this catches the bug at the agent
/// boundary with a readable 400.
async fn ensure_indicators_known<'a>(
    state: &AppState,
    codes: impl IntoIterator<Item = &'a str>,
) -> ApiResult<()> {
    let indicators = plutus_storage::queries::macros::list_indicators(&state.db).await?;
    let known: HashSet<String> = indicators.into_iter().map(|i| i.code).collect();
    let mut missing: Vec<String> = codes
        .into_iter()
        .filter(|c| !known.contains(*c))
        .map(|c| c.to_string())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    missing.sort();
    missing.dedup();
    Err(ApiError::BadRequest(format!(
        "unknown indicator_code(s): {}. Register via POST /api/v1/macro/indicators before posting events.",
        missing.join(", ")
    )))
}
