//! Quarterly fundamentals snapshot per stock. One row per (stock, fiscal_year,
//! fiscal_period). Nullable fields tolerate sources that only report a subset.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "fundamentals_quarterly"]
pub struct FundamentalsQuarterly {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub fiscal_year: i32,
    pub fiscal_period: String, // "Q1" / "Q2" / "Q3" / "Q4" / "FY" / "H1" / "H2"
    pub period_end: String,    // ISO date
    pub currency: String,
    pub revenue: Option<Decimal>,
    pub gross_profit: Option<Decimal>,
    pub operating_income: Option<Decimal>,
    pub net_income: Option<Decimal>,
    pub eps_basic: Option<Decimal>,
    pub eps_diluted: Option<Decimal>,
    pub cash: Option<Decimal>,
    pub total_debt: Option<Decimal>,
    pub total_equity: Option<Decimal>,
    pub operating_cf: Option<Decimal>,
    pub free_cf: Option<Decimal>,
    pub shares_outstanding: Option<Decimal>,
    pub source: String,
    pub created_at: jiff::Timestamp,
}
