use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::macro_events::LocalizedMacroEvent;

/// A discrete macro event (one CPI print, one FOMC meeting, one rate
/// decision). Distinct from `macro_observations` which is the continuous
/// time series of a `macro_indicator`. Upserts on
/// `(indicator_code, event_date)`. Shared across users (reference data).
///
/// `title` and `summary_md` are projected from `content.<locale>`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MacroEventOut {
    /// Primary key.
    pub id: i64,
    /// FK-ish to `macro_indicators.code` (e.g. `cpi_yoy`, `fed_funds`).
    /// Part of the natural key.
    pub indicator_code: String,
    /// ISO date `YYYY-MM-DD` when the event happened or is scheduled. Part
    /// of the natural key.
    pub event_date: String,
    /// `release`, `decision`, `meeting`, `speech`, `cancellation`. Free-form.
    pub event_kind: String,
    /// Localized headline.
    pub title: Option<String>,
    /// Localized markdown commentary.
    pub summary_md: Option<String>,
    /// For `decision` events: `hike` | `hold` | `cut` (or whatever the
    /// agent uses).
    pub decision: Option<String>,
    /// Basis-points change for rate decisions (e.g. `25`, `-50`).
    pub decision_bps: Option<i32>,
    /// Realized value for the indicator (the print). Unit matches the
    /// indicator's convention.
    #[schema(value_type = Option<String>)] pub new_value: Option<Decimal>,
    /// Bloomberg / Reuters consensus going into the print.
    #[schema(value_type = Option<String>)] pub consensus_estimate: Option<Decimal>,
    /// `new_value − consensus_estimate`. Computed server-side if absent in
    /// the POST body.
    #[schema(value_type = Option<String>)] pub surprise: Option<Decimal>,
    /// Prior print, for context.
    #[schema(value_type = Option<String>)] pub previous_value: Option<Decimal>,
    /// FOMC-style vote tally as a JSON string (e.g.
    /// `"{\"for\":10,\"against\":2}"`).
    pub vote: Option<String>,
    /// JSON-stringified dot plot (agent picks the shape).
    pub dot_plot: Option<String>,
    /// Canonical URL — release page, statement, transcript.
    pub url: Option<String>,
    /// Provenance.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

impl From<LocalizedMacroEvent> for MacroEventOut {
    fn from(e: LocalizedMacroEvent) -> Self {
        Self {
            id: e.id,
            indicator_code: e.indicator_code,
            event_date: e.event_date,
            event_kind: e.event_kind,
            title: e.title,
            summary_md: e.summary_md,
            decision: e.decision,
            decision_bps: e.decision_bps,
            new_value: e.new_value,
            consensus_estimate: e.consensus_estimate,
            surprise: e.surprise,
            previous_value: e.previous_value,
            vote: e.vote,
            dot_plot: e.dot_plot,
            url: e.url,
            source: e.source,
            created_at: e.created_at.to_string(),
            updated_at: e.updated_at.to_string(),
        }
    }
}

/// `POST /macro/events` body. Upserts against
/// `(indicator_code, event_date)` — re-posting the same key refreshes the
/// row (so a `scheduled → released → revised` flow is one event id).
#[derive(Debug, Deserialize, ToSchema)]
pub struct MacroEventIn {
    /// FK-ish to `macro_indicators.code`. Part of the natural key.
    pub indicator_code: String,
    /// ISO date `YYYY-MM-DD`. Part of the natural key.
    pub event_date: String,
    /// `release` | `decision` | `meeting` | `speech` | `cancellation`.
    pub event_kind: String,
    /// `hike` / `hold` / `cut` for decisions.
    pub decision: Option<String>,
    /// Basis-points change (`25`, `-50`).
    pub decision_bps: Option<i32>,
    /// Realized print value.
    #[schema(value_type = Option<String>)] pub new_value: Option<Decimal>,
    /// Pre-release consensus.
    #[schema(value_type = Option<String>)] pub consensus_estimate: Option<Decimal>,
    /// `new_value − consensus_estimate`. Computed server-side when omitted
    /// and both inputs are present.
    #[schema(value_type = Option<String>)] pub surprise: Option<Decimal>,
    /// Prior print.
    #[schema(value_type = Option<String>)] pub previous_value: Option<Decimal>,
    /// Vote tally as a JSON string. The agent picks the shape.
    pub vote: Option<String>,
    /// Dot plot as a JSON string.
    pub dot_plot: Option<String>,
    /// Canonical URL.
    pub url: Option<String>,
    /// Provenance. Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "title": "...", "summary_md": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }

/// `POST /macro/events/batch` body. Caps at 1000 items; one transaction.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct MacroEventBatchIn {
    pub items: Vec<MacroEventIn>,
}

/// `POST /macro/events/batch` response.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct MacroEventBatchOut {
    /// Number persisted (`== items.len()`).
    pub count: usize,
    /// Persisted rows in input order. Refresh vs insert is invisible —
    /// check `created_at == updated_at` for new rows.
    pub items: Vec<MacroEventOut>,
}
