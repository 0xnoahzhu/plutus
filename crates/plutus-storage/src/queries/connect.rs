use rust_decimal::Decimal;

use crate::db::{Db, Result};
use crate::models::{ConnectFlowDaily, ConnectHoldingsDaily};

// ── Flow ─────────────────────────────────────────────────────────────────

pub async fn list_flow(db: &Db) -> Result<Vec<ConnectFlowDaily>> {
    db.with(async |d| ConnectFlowDaily::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub struct NewFlow<'a> {
    pub market_code: &'a str,
    pub direction: &'a str,
    pub flow_date: &'a str,
    pub net_buy: Decimal,
    pub net_buy_currency: &'a str,
    pub total_buy: Option<Decimal>,
    pub total_sell: Option<Decimal>,
    pub quota_balance: Option<Decimal>,
    pub source: &'a str,
}

pub async fn insert_flow(db: &Db, input: NewFlow<'_>) -> Result<ConnectFlowDaily> {
    let market_code = input.market_code.to_string();
    let direction = input.direction.to_string();
    let flow_date = input.flow_date.to_string();
    let net_buy = input.net_buy;
    let net_buy_currency = input.net_buy_currency.to_string();
    let total_buy = input.total_buy;
    let total_sell = input.total_sell;
    let quota_balance = input.quota_balance;
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();

    let row = db
        .with(async |d| {
            toasty::create!(ConnectFlowDaily {
                market_code: market_code,
                direction: direction,
                flow_date: flow_date,
                net_buy: net_buy,
                net_buy_currency: net_buy_currency,
                total_buy: total_buy,
                total_sell: total_sell,
                quota_balance: quota_balance,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

// ── Holdings ─────────────────────────────────────────────────────────────

pub async fn list_holdings_for_stock(
    db: &Db,
    stock_id: i64,
) -> Result<Vec<ConnectHoldingsDaily>> {
    db.with(async |d| {
        ConnectHoldingsDaily::all()
            .filter(ConnectHoldingsDaily::fields().stock_id().eq(stock_id))
            .exec(d)
            .await
    })
    .await
    .map_err(Into::into)
}

pub struct NewHoldings<'a> {
    pub stock_id: i64,
    pub direction: &'a str,
    pub holding_date: &'a str,
    pub shares: Decimal,
    pub value: Option<Decimal>,
    pub value_currency: Option<&'a str>,
    pub pct_of_float: Option<Decimal>,
    pub source: &'a str,
}

pub async fn insert_holdings(db: &Db, input: NewHoldings<'_>) -> Result<ConnectHoldingsDaily> {
    let stock_id = input.stock_id;
    let direction = input.direction.to_string();
    let holding_date = input.holding_date.to_string();
    let shares = input.shares;
    let value = input.value;
    let value_currency = input.value_currency.map(str::to_string);
    let pct_of_float = input.pct_of_float;
    let source = input.source.to_string();
    let now = jiff::Timestamp::now();

    let row = db
        .with(async |d| {
            toasty::create!(ConnectHoldingsDaily {
                stock_id: stock_id,
                direction: direction,
                holding_date: holding_date,
                shares: shares,
                value: value,
                value_currency: value_currency,
                pct_of_float: pct_of_float,
                source: source,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}
