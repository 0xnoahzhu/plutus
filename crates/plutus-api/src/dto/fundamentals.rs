use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::FundamentalsQuarterly;

#[derive(Debug, Serialize, ToSchema)]
pub struct FundamentalsOut {
    pub id: i64,
    pub stock_id: i64,
    pub fiscal_year: i32,
    pub fiscal_period: String,
    pub period_end: String,
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
    pub source: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct FundamentalsIn {
    pub stock_id: i64,
    pub fiscal_year: i32,
    pub fiscal_period: String,
    pub period_end: String,
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
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String { "agent".into() }
