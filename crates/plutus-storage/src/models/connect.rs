//! Stock Connect flows: daily net flow totals and per-stock holdings.
//! `direction = "southbound"` is HK shares bought via the Connect from
//! mainland; `direction = "northbound"` is A-shares bought from HK.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "connect_flow_daily"]
pub struct ConnectFlowDaily {
    #[key]
    #[auto]
    pub id: i64,
    pub market_code: String, // "XHKG" for southbound, "XSHG"/"XSHE" for northbound
    pub direction: String,   // "southbound" / "northbound"
    pub flow_date: String,   // ISO date
    pub net_buy: Decimal,
    pub net_buy_currency: String, // "HKD" / "CNY"
    pub total_buy: Option<Decimal>,
    pub total_sell: Option<Decimal>,
    pub quota_balance: Option<Decimal>,
    pub source: String,
    pub created_at: jiff::Timestamp,
}

#[derive(Debug, toasty::Model)]
#[table = "connect_holdings_daily"]
pub struct ConnectHoldingsDaily {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub direction: String, // "southbound" / "northbound"
    pub holding_date: String,
    pub shares: Decimal,
    pub value: Option<Decimal>,
    pub value_currency: Option<String>,
    pub pct_of_float: Option<Decimal>,
    pub source: String,
    pub created_at: jiff::Timestamp,
}
