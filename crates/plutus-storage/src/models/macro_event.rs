//! Discrete macro / policy events: FOMC decisions, CPI releases, LPR
//! announcements, etc. Unlike `macro_observations` (continuous time series),
//! these carry rich structured fields per event: decision direction, vote,
//! dot-plot blob, and the consensus-vs-actual surprise.
//!
//! Natural key (indicator_code, event_date) — one event per indicator per day.
//!
//! Translatable content (title, summary_md) lives in the `content` JSONB
//! column on the DB side. Because toasty 0.6 doesn't speak JSONB, the model
//! omits that column entirely — raw `tokio_postgres` SQL in
//! `queries::macro_events` handles read/write of localized content.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "macro_events"]
pub struct MacroEvent {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub indicator_code: String,
    pub event_date: String, // ISO "YYYY-MM-DD"
    /// "fomc_decision" / "ecb_decision" / "cpi_release" / "lpr_decision" /
    /// "pmi_release" / "nfp_release" / "gdp_release" / "boj_decision" / ...
    pub event_kind: String,
    /// For policy decisions: "hike" / "cut" / "hold". null for data releases.
    pub decision: Option<String>,
    /// Magnitude in basis points. +25 = hike 25bps, -50 = cut 50bps, 0 = hold.
    pub decision_bps: Option<i32>,
    /// New value: new policy rate, or actual data point (CPI / PPI / etc).
    pub new_value: Option<Decimal>,
    pub consensus_estimate: Option<Decimal>,
    /// new_value − consensus_estimate. Agent computes; we store for query speed.
    pub surprise: Option<Decimal>,
    pub previous_value: Option<Decimal>,
    /// Vote string like "11-1" or "9-3 with dissents".
    pub vote: Option<String>,
    /// Dot plot / projection blob — agent picks its own JSON shape, stored as text.
    pub dot_plot: Option<String>,
    pub url: Option<String>,
    pub source: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
