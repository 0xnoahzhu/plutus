use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;

use crate::dto::macro_event::{MacroEventIn, MacroEventOut};
use crate::error::{ApiError, ApiResult};
use crate::i18n::{apply_overrides, LocaleQuery};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub indicator_code: Option<String>,
    pub event_kind: Option<String>,
    pub country: Option<String>, // resolved through macro_indicators.country
    pub from: Option<String>,
    pub to: Option<String>,
}

fn localize(out: &mut MacroEventOut, locale: &str) {
    let trans = out.translations.clone();
    apply_overrides(out, trans.as_deref(), locale);
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<ListFilter>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<Vec<MacroEventOut>>> {
    let mut rows = plutus_storage::queries::macro_events::list(
        &state.db,
        plutus_storage::queries::macro_events::ListFilter {
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

    rows.sort_by(|a, b| a.event_date.cmp(&b.event_date).then(a.indicator_code.cmp(&b.indicator_code)));
    let mut out: Vec<MacroEventOut> = rows.into_iter().map(Into::into).collect();
    for o in out.iter_mut() {
        localize(o, &l.locale);
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(l): Query<LocaleQuery>,
) -> ApiResult<Json<MacroEventOut>> {
    let row = plutus_storage::queries::macro_events::get(&state.db, id).await?;
    let mut out: MacroEventOut = row.into();
    localize(&mut out, &l.locale);
    Ok(Json(out))
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(input): Json<MacroEventIn>,
) -> ApiResult<Json<MacroEventOut>> {
    let translations = match input.translations {
        Some(v) => Some(serde_json::to_string(&v)
            .map_err(|e| ApiError::BadRequest(format!("translations: {e}")))?),
        None => None,
    };
    let row = plutus_storage::queries::macro_events::upsert(
        &state.db,
        plutus_storage::queries::macro_events::NewMacroEvent {
            indicator_code: &input.indicator_code,
            event_date: &input.event_date,
            event_kind: &input.event_kind,
            title: &input.title,
            summary_md: input.summary_md.as_deref(),
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
    plutus_storage::queries::macro_events::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
