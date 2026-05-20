use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::FundamentalsQuarterly;

/// Per-quarter fundamentals snapshot for one stock — the as-reported
/// income statement / balance sheet / cash flow numbers. Shared across
/// users. All monetary fields use `currency`; the agent typically picks
/// the company's reporting currency (e.g. USD for US, HKD/RMB for HK/CN).
#[derive(Debug, Serialize, ToSchema)]
pub struct FundamentalsOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// 4-digit fiscal year.
    pub fiscal_year: i32,
    /// `Q1` / `Q2` / `Q3` / `Q4` / `FY`.
    pub fiscal_period: String,
    /// ISO date `YYYY-MM-DD` end of the period.
    pub period_end: String,
    /// ISO-4217 reporting currency for all monetary fields below.
    pub currency: String,
    /// Top-line revenue.
    #[schema(value_type = Option<String>)] pub revenue: Option<Decimal>,
    /// Gross profit (revenue − COGS).
    #[schema(value_type = Option<String>)] pub gross_profit: Option<Decimal>,
    /// Operating income (gross profit − operating expenses).
    #[schema(value_type = Option<String>)] pub operating_income: Option<Decimal>,
    /// Net income attributable to common shareholders.
    #[schema(value_type = Option<String>)] pub net_income: Option<Decimal>,
    /// Basic EPS.
    #[schema(value_type = Option<String>)] pub eps_basic: Option<Decimal>,
    /// Diluted EPS.
    #[schema(value_type = Option<String>)] pub eps_diluted: Option<Decimal>,
    /// Cash + equivalents at period end.
    #[schema(value_type = Option<String>)] pub cash: Option<Decimal>,
    /// Total interest-bearing debt at period end.
    #[schema(value_type = Option<String>)] pub total_debt: Option<Decimal>,
    /// Total shareholders' equity at period end.
    #[schema(value_type = Option<String>)] pub total_equity: Option<Decimal>,
    /// Cash from operations for the period.
    #[schema(value_type = Option<String>)] pub operating_cf: Option<Decimal>,
    /// Free cash flow (operating_cf − capex).
    #[schema(value_type = Option<String>)] pub free_cf: Option<Decimal>,
    /// Weighted-average diluted shares outstanding.
    #[schema(value_type = Option<String>)] pub shares_outstanding: Option<Decimal>,
    /// Provenance.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
}

impl From<FundamentalsQuarterly> for FundamentalsOut {
    fn from(f: FundamentalsQuarterly) -> Self {
        Self {
            id: f.id,
            stock_id: f.stock_id,
            fiscal_year: f.fiscal_year,
            fiscal_period: f.fiscal_period,
            period_end: f.period_end,
            currency: f.currency,
            revenue: f.revenue,
            gross_profit: f.gross_profit,
            operating_income: f.operating_income,
            net_income: f.net_income,
            eps_basic: f.eps_basic,
            eps_diluted: f.eps_diluted,
            cash: f.cash,
            total_debt: f.total_debt,
            total_equity: f.total_equity,
            operating_cf: f.operating_cf,
            free_cf: f.free_cf,
            shares_outstanding: f.shares_outstanding,
            source: f.source,
            created_at: f.created_at.to_string(),
        }
    }
}

/// `POST /fundamentals` body. Upserts on
/// `(stock_id, fiscal_year, fiscal_period)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct FundamentalsIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// `Q1`/`Q2`/`Q3`/`Q4`/`FY`.
    pub fiscal_period: String,
    /// ISO date `YYYY-MM-DD`.
    pub period_end: String,
    /// ISO-4217 reporting currency.
    pub currency: String,
    #[schema(value_type = Option<String>)] pub revenue: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub gross_profit: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub operating_income: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub net_income: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub eps_basic: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub eps_diluted: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub cash: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub total_debt: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub total_equity: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub operating_cf: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub free_cf: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub shares_outstanding: Option<Decimal>,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String { "agent".into() }
