use rust_decimal::Decimal;

use crate::db::{Db, Result};
use crate::models::FundamentalsQuarterly;

pub async fn list_for_stock(db: &Db, stock_id: i64) -> Result<Vec<FundamentalsQuarterly>> {
    db.with(async |d| {
        FundamentalsQuarterly::all()
            .filter(FundamentalsQuarterly::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub struct NewFundamentals<'a> {
    pub stock_id: i64,
    pub fiscal_year: i32,
    pub fiscal_period: &'a str,
    pub period_end: &'a str,
    pub currency: &'a str,
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
    pub source: &'a str,
}

pub async fn insert(db: &Db, input: NewFundamentals<'_>) -> Result<FundamentalsQuarterly> {
    let stock_id = input.stock_id;
    let fiscal_year = input.fiscal_year;
    let fiscal_period = input.fiscal_period.to_string();
    let period_end = input.period_end.to_string();
    let currency = input.currency.to_string();
    let revenue = input.revenue;
    let gross_profit = input.gross_profit;
    let operating_income = input.operating_income;
    let net_income = input.net_income;
    let eps_basic = input.eps_basic;
    let eps_diluted = input.eps_diluted;
    let cash = input.cash;
    let total_debt = input.total_debt;
    let total_equity = input.total_equity;
    let operating_cf = input.operating_cf;
    let free_cf = input.free_cf;
    let shares_outstanding = input.shares_outstanding;
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();

    let row = db
        .with(async |d| {
            toasty::create!(FundamentalsQuarterly {
                stock_id: stock_id,
                fiscal_year: fiscal_year,
                fiscal_period: fiscal_period,
                period_end: period_end,
                currency: currency,
                revenue: revenue,
                gross_profit: gross_profit,
                operating_income: operating_income,
                net_income: net_income,
                eps_basic: eps_basic,
                eps_diluted: eps_diluted,
                cash: cash,
                total_debt: total_debt,
                total_equity: total_equity,
                operating_cf: operating_cf,
                free_cf: free_cf,
                shares_outstanding: shares_outstanding,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
