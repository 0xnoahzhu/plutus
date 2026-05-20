use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Filing;

/// A regulatory filing — SEC 10-K / 10-Q / 8-K / S-1, HKEX announcements,
/// CSRC submissions. Shared across users. The `content_md` is optional;
/// the agent typically stores a cleaned-up summary, not the full filing
/// (which can be hundreds of pages).
#[derive(Debug, Serialize, ToSchema)]
pub struct FilingOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Filing type — `10-K` | `10-Q` | `8-K` | `S-1` | `13D` | `13G` |
    /// HKEX / CSRC codes. Free-form text.
    pub filing_type: String,
    /// Fiscal year covered (when applicable).
    pub fiscal_year: Option<i32>,
    /// `Q1` / `Q2` / `Q3` / `Q4` / `FY` (when applicable — `8-K`s have no
    /// fiscal period).
    pub fiscal_period: Option<String>,
    /// ISO date `YYYY-MM-DD` end of the reporting period.
    pub period_end: Option<String>,
    /// RFC 3339 UTC timestamp the filing was submitted.
    pub filed_at: String,
    /// Canonical URL to the filing on the regulator's site.
    pub url: String,
    /// Filing title / headline.
    pub title: String,
    /// Cleaned-up English markdown of the filing's substance. Not
    /// multi-locale.
    pub content_md: Option<String>,
    /// Provenance.
    pub source: String,
    /// RFC 3339 UTC timestamp.
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

/// `POST /filings` body. Always inserts.
#[derive(Debug, Deserialize, ToSchema)]
pub struct FilingIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Filing type code.
    pub filing_type: String,
    /// Fiscal year.
    pub fiscal_year: Option<i32>,
    /// Fiscal period.
    pub fiscal_period: Option<String>,
    /// ISO date `YYYY-MM-DD`.
    pub period_end: Option<String>,
    /// RFC 3339 UTC timestamp.
    pub filed_at: String,
    /// Filing URL.
    pub url: String,
    /// Headline / title.
    pub title: String,
    /// Optional cleaned-up English markdown.
    pub content_md: Option<String>,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String { "agent".into() }
