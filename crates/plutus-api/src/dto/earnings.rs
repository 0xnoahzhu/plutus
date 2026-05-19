use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::EarningsEvent;

#[derive(Debug, Serialize, ToSchema)]
pub struct EarningsOut {
    pub id: i64,
    pub stock_id: i64,
    pub fiscal_year: i32,
    pub fiscal_period: String,
    pub announce_at: Option<String>,
    pub announce_date: String,
    pub announce_timing: String,
    pub status: String,
    #[schema(value_type = Option<String>)] pub eps_estimate: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub eps_actual: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub revenue_estimate: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub revenue_actual: Option<Decimal>,
    pub currency: Option<String>,
    pub guidance_md: Option<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<EarningsEvent> for EarningsOut {
    fn from(e: EarningsEvent) -> Self {
        Self {
            id: e.id,
            stock_id: e.stock_id,
            fiscal_year: e.fiscal_year,
            fiscal_period: e.fiscal_period,
            announce_at: e.announce_at.map(|t| t.to_string()),
            announce_date: e.announce_date,
            announce_timing: e.announce_timing,
            status: e.status,
            eps_estimate: e.eps_estimate,
            eps_actual: e.eps_actual,
            revenue_estimate: e.revenue_estimate,
            revenue_actual: e.revenue_actual,
            currency: e.currency,
            guidance_md: e.guidance_md,
            notes: e.notes,
            url: e.url,
            source: e.source,
            created_at: e.created_at.to_string(),
            updated_at: e.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EarningsIn {
    pub stock_id: i64,
    pub fiscal_year: i32,
    pub fiscal_period: String,
    /// RFC 3339 timestamp; optional when only the date is known.
    pub announce_at: Option<String>,
    /// ISO YYYY-MM-DD.
    pub announce_date: String,
    /// "bmo" / "amc" / "during" / "unknown".
    #[serde(default = "default_timing")]
    pub announce_timing: String,
    /// "scheduled" / "confirmed" / "released" / "postponed".
    #[serde(default = "default_status")]
    pub status: String,
    #[schema(value_type = Option<String>)] pub eps_estimate: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub eps_actual: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub revenue_estimate: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub revenue_actual: Option<Decimal>,
    pub currency: Option<String>,
    pub guidance_md: Option<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_timing() -> String { "unknown".into() }
fn default_status() -> String { "scheduled".into() }
fn default_source() -> String { "agent".into() }
