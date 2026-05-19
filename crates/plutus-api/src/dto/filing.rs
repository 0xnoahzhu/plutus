use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Filing;

#[derive(Debug, Serialize, ToSchema)]
pub struct FilingOut {
    pub id: i64,
    pub stock_id: i64,
    pub filing_type: String,
    pub fiscal_year: Option<i32>,
    pub fiscal_period: Option<String>,
    pub period_end: Option<String>,
    pub filed_at: String,
    pub url: String,
    pub title: String,
    pub content_md: Option<String>,
    pub source: String,
    pub created_at: String,
}

impl From<Filing> for FilingOut {
    fn from(f: Filing) -> Self {
        Self {
            id: f.id,
            stock_id: f.stock_id,
            filing_type: f.filing_type,
            fiscal_year: f.fiscal_year,
            fiscal_period: f.fiscal_period,
            period_end: f.period_end,
            filed_at: f.filed_at.to_string(),
            url: f.url,
            title: f.title,
            content_md: f.content_md,
            source: f.source,
            created_at: f.created_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FilingIn {
    pub stock_id: i64,
    pub filing_type: String,
    pub fiscal_year: Option<i32>,
    pub fiscal_period: Option<String>,
    pub period_end: Option<String>,
    /// RFC 3339 timestamp.
    pub filed_at: String,
    pub url: String,
    pub title: String,
    pub content_md: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String { "agent".into() }
