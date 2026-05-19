use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::macro_events::LocalizedMacroEvent;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MacroEventOut {
    pub id: i64,
    pub indicator_code: String,
    pub event_date: String,
    pub event_kind: String,
    pub title: Option<String>,
    pub summary_md: Option<String>,
    pub decision: Option<String>,
    pub decision_bps: Option<i32>,
    #[schema(value_type = Option<String>)] pub new_value: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub consensus_estimate: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub surprise: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub previous_value: Option<Decimal>,
    pub vote: Option<String>,
    pub dot_plot: Option<String>,
    pub url: Option<String>,
    pub source: String,
    pub created_at: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct MacroEventIn {
    pub indicator_code: String,
    pub event_date: String,
    pub event_kind: String,
    pub decision: Option<String>,
    pub decision_bps: Option<i32>,
    #[schema(value_type = Option<String>)] pub new_value: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub consensus_estimate: Option<Decimal>,
    /// Optional — computed automatically as new_value − consensus_estimate if absent.
    #[schema(value_type = Option<String>)] pub surprise: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub previous_value: Option<Decimal>,
    pub vote: Option<String>,
    /// JSON string; agent picks its own shape.
    pub dot_plot: Option<String>,
    pub url: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob — `{ "<locale>": { "title": "...",
    /// "summary_md": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
