//! Regulatory filings (10-K / 10-Q / 8-K / HKEX disclosures / CSRC 公告 /
//! Form 4). Distinct from `news_items` so the schema can be stricter: every
//! filing belongs to exactly one stock and has a filing_type from a known set.

#[derive(Debug, toasty::Model)]
#[table = "filings"]
pub struct Filing {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub filing_type: String,
    // ^ "10-K" / "10-Q" / "8-K" / "6-K" / "20-F" / "S-1"
    //   "HKEX_ANNOUNCEMENT" / "CSRC_ANNOUNCEMENT" / "FORM_4" / "13F" / "13G"
    pub fiscal_year: Option<i32>,
    pub fiscal_period: Option<String>, // "FY" / "Q1" / "Q2" / "Q3" / "Q4" / "H1" / "H2"
    pub period_end: Option<String>,    // ISO date
    pub filed_at: jiff::Timestamp,
    pub url: String,
    pub title: String,
    pub content_md: Option<String>,
    pub source: String,
    pub created_at: jiff::Timestamp,
}
