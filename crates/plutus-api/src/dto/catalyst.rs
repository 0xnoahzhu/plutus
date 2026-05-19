use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Catalyst;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CatalystOut {
    pub id: i64,
    pub stock_id: Option<i64>,
    pub sector_code: Option<String>,
    pub country: Option<String>,
    pub catalyst_kind: String,
    pub title: String,
    pub summary_md: Option<String>,
    pub catalyst_date: String,
    pub date_confidence: String,
    pub impact_level: String,
    pub bull_case_md: Option<String>,
    pub bear_case_md: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub source: String,
    pub translations: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Catalyst> for CatalystOut {
    fn from(c: Catalyst) -> Self {
        Self {
            id: c.id, stock_id: c.stock_id, sector_code: c.sector_code, country: c.country,
            catalyst_kind: c.catalyst_kind, title: c.title, summary_md: c.summary_md,
            catalyst_date: c.catalyst_date, date_confidence: c.date_confidence,
            impact_level: c.impact_level, bull_case_md: c.bull_case_md,
            bear_case_md: c.bear_case_md, status: c.status, notes: c.notes,
            url: c.url, source: c.source,
            translations: c.translations,
            created_at: c.created_at.to_string(), updated_at: c.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CatalystIn {
    pub stock_id: Option<i64>,
    pub sector_code: Option<String>,
    pub country: Option<String>,
    pub catalyst_kind: String,
    pub title: String,
    pub summary_md: Option<String>,
    pub catalyst_date: String,
    #[serde(default = "default_confidence")]
    pub date_confidence: String,
    #[serde(default = "default_impact")]
    pub impact_level: String,
    pub bull_case_md: Option<String>,
    pub bear_case_md: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    pub notes: Option<String>,
    pub url: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    pub translations: Option<serde_json::Value>,
}

fn default_confidence() -> String { "scheduled".into() }
fn default_impact() -> String { "medium".into() }
fn default_status() -> String { "upcoming".into() }
fn default_source() -> String { "agent".into() }
