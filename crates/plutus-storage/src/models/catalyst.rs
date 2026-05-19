//! Forward-looking catalysts beyond what `earnings_events` and `macro_events`
//! capture. FDA approvals, product launches, court rulings, investor days,
//! IPOs, merger closes, lockup expiries, regulatory deadlines, elections.

#[derive(Debug, toasty::Model)]
#[table = "catalysts"]
pub struct Catalyst {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    // ── Mount points (at least one expected) ────────────────────────
    #[index]
    pub stock_id: Option<i64>,
    pub sector_code: Option<String>,
    pub country: Option<String>,
    // ── Event itself ────────────────────────────────────────────────
    pub catalyst_kind: String,
    // ^^ "fda_decision" / "product_launch" / "court_ruling" / "investor_day"
    //    / "shareholder_vote" / "trial_readout" / "merger_close" / "ipo"
    //    / "spinoff" / "license_expiry" / "lockup_expiry" / "election"
    //    / "trade_deadline" / "regulatory_deadline" / "trade_show"
    pub title: String,
    pub summary_md: Option<String>,
    #[index]
    pub catalyst_date: String,    // ISO date — when expected
    pub date_confidence: String,  // "confirmed" / "scheduled" / "expected" / "rumored"
    pub impact_level: String,     // "high" / "medium" / "low"
    pub bull_case_md: Option<String>,
    pub bear_case_md: Option<String>,
    pub status: String, // "upcoming" / "happened_positive" / "happened_negative"
                        //  / "happened_neutral" / "delayed" / "cancelled"
    pub notes: Option<String>,
    pub url: Option<String>,
    pub source: String,
    /// JSON map of locale → overrides for title / summary_md / bull_case_md /
    /// bear_case_md / notes.
    pub translations: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
