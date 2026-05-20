use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::EarningsEvent;

/// An earnings event for a stock — one per `(stock_id, fiscal_year,
/// fiscal_period)`. Carries the scheduled announce time plus estimate /
/// actual numbers as they become known. Shared across users (reference
/// data).
///
/// No `content` JSONB here — the human-readable fields (`guidance_md`,
/// `notes`) are stored directly as text. They're not multi-locale.
#[derive(Debug, Serialize, ToSchema)]
pub struct EarningsOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`. Part of the natural key.
    pub stock_id: i64,
    /// 4-digit fiscal year (e.g. `2026`). Calendar year for most US
    /// issuers; non-calendar for some (Apple's FY ends in September —
    /// careful when joining to `fundamentals_quarterly`).
    pub fiscal_year: i32,
    /// `Q1` / `Q2` / `Q3` / `Q4` / `FY`. Part of the natural key.
    pub fiscal_period: String,
    /// RFC 3339 timestamp if a precise minute is known. `null` when only
    /// the date is published.
    pub announce_at: Option<String>,
    /// ISO date `YYYY-MM-DD`. Always set.
    pub announce_date: String,
    /// `bmo` (before market open), `amc` (after market close), `during`,
    /// `unknown`.
    pub announce_timing: String,
    /// Lifecycle — `scheduled` | `confirmed` | `released` | `postponed`.
    pub status: String,
    /// Consensus EPS (currency = `currency` field below).
    #[schema(value_type = Option<String>)] pub eps_estimate: Option<Decimal>,
    /// Reported EPS.
    #[schema(value_type = Option<String>)] pub eps_actual: Option<Decimal>,
    /// Consensus revenue.
    #[schema(value_type = Option<String>)] pub revenue_estimate: Option<Decimal>,
    /// Reported revenue.
    #[schema(value_type = Option<String>)] pub revenue_actual: Option<Decimal>,
    /// ISO-4217 reporting currency. Falls back to the stock's trading
    /// currency when missing.
    pub currency: Option<String>,
    /// Free-form markdown — forward guidance, key callouts. English only;
    /// not multi-locale.
    pub guidance_md: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// IR release URL.
    pub url: Option<String>,
    /// Provenance.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
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

/// `POST /earnings` body. Upserts against
/// `(stock_id, fiscal_year, fiscal_period)`. Typical flow: schedule
/// (status=`scheduled`, only `announce_date` known) → confirm
/// (status=`confirmed`, exact `announce_at` + `eps_estimate`/`revenue_estimate`)
/// → release (status=`released`, all four numbers filled).
#[derive(Debug, Deserialize, ToSchema)]
pub struct EarningsIn {
    /// FK to `stocks.id`. Part of the natural key.
    pub stock_id: i64,
    /// 4-digit fiscal year. Part of the natural key.
    pub fiscal_year: i32,
    /// `Q1`/`Q2`/`Q3`/`Q4`/`FY`. Part of the natural key.
    pub fiscal_period: String,
    /// RFC 3339 timestamp; optional until the exact minute is known.
    pub announce_at: Option<String>,
    /// ISO date `YYYY-MM-DD`.
    pub announce_date: String,
    /// `bmo` / `amc` / `during` / `unknown`. Default `unknown`.
    #[serde(default = "default_timing")]
    pub announce_timing: String,
    /// Default `scheduled`.
    #[serde(default = "default_status")]
    pub status: String,
    /// Consensus EPS.
    #[schema(value_type = Option<String>)] pub eps_estimate: Option<Decimal>,
    /// Reported EPS.
    #[schema(value_type = Option<String>)] pub eps_actual: Option<Decimal>,
    /// Consensus revenue.
    #[schema(value_type = Option<String>)] pub revenue_estimate: Option<Decimal>,
    /// Reported revenue.
    #[schema(value_type = Option<String>)] pub revenue_actual: Option<Decimal>,
    /// ISO-4217 reporting currency.
    pub currency: Option<String>,
    /// Forward guidance markdown (English only).
    pub guidance_md: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// IR release URL.
    pub url: Option<String>,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_timing() -> String { "unknown".into() }
fn default_status() -> String { "scheduled".into() }
fn default_source() -> String { "agent".into() }

/// `POST /earnings/batch` body. Caps at 1000 items; one transaction.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct EarningsBatchIn {
    pub items: Vec<EarningsIn>,
}

/// `POST /earnings/batch` response.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct EarningsBatchOut {
    /// Number persisted (`== items.len()`).
    pub count: usize,
    /// Rows in input order.
    pub items: Vec<EarningsOut>,
}
