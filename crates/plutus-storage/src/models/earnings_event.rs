//! Earnings calendar events. One row per stock per fiscal period. Natural key
//! (stock_id, fiscal_year, fiscal_period) enforced at the app layer via
//! upsert in queries.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "earnings_events"]
pub struct EarningsEvent {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub fiscal_year: i32,
    pub fiscal_period: String, // "Q1" / "Q2" / "Q3" / "Q4" / "FY" / "H1" / "H2"
    /// Precise announce timestamp (filled when confirmed).
    pub announce_at: Option<jiff::Timestamp>,
    /// ISO date — set even when time-of-day is unknown.
    pub announce_date: String,
    /// "bmo" (before market open) / "amc" (after market close) / "during" /
    /// "unknown". Independent of `announce_at` since calendars often publish
    /// just the date + session label.
    pub announce_timing: String,
    /// "scheduled" / "confirmed" / "released" / "postponed".
    pub status: String,
    pub eps_estimate: Option<Decimal>,
    pub eps_actual: Option<Decimal>,
    pub revenue_estimate: Option<Decimal>,
    pub revenue_actual: Option<Decimal>,
    pub currency: Option<String>,
    pub guidance_md: Option<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub source: String, // "agent" / "manual"
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
