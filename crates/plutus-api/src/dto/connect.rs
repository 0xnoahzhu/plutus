use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{ConnectFlowDaily, ConnectHoldingsDaily};

#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectFlowOut {
    pub id: i64,
    pub market_code: String,
    pub direction: String,
    pub flow_date: String,
    #[schema(value_type = String)] pub net_buy: Decimal,
    pub net_buy_currency: String,
    #[schema(value_type = Option<String>)] pub total_buy: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub total_sell: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub quota_balance: Option<Decimal>,
    pub source: String,
}

impl From<ConnectFlowDaily> for ConnectFlowOut {
    fn from(f: ConnectFlowDaily) -> Self {
        Self {
            id: f.id, market_code: f.market_code, direction: f.direction,
            flow_date: f.flow_date, net_buy: f.net_buy, net_buy_currency: f.net_buy_currency,
            total_buy: f.total_buy, total_sell: f.total_sell,
            quota_balance: f.quota_balance, source: f.source,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnectFlowIn {
    pub market_code: String,
    pub direction: String,
    pub flow_date: String,
    #[schema(value_type = String)] pub net_buy: Decimal,
    pub net_buy_currency: String,
    #[schema(value_type = Option<String>)] pub total_buy: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub total_sell: Option<Decimal>,
    #[schema(value_type = Option<String>)] pub quota_balance: Option<Decimal>,
    #[serde(default = "default_source")] pub source: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectHoldingsOut {
    pub id: i64,
    pub stock_id: i64,
    pub direction: String,
    pub holding_date: String,
    #[schema(value_type = String)] pub shares: Decimal,
    #[schema(value_type = Option<String>)] pub value: Option<Decimal>,
    pub value_currency: Option<String>,
    #[schema(value_type = Option<String>)] pub pct_of_float: Option<Decimal>,
    pub source: String,
}

impl From<ConnectHoldingsDaily> for ConnectHoldingsOut {
    fn from(h: ConnectHoldingsDaily) -> Self {
        Self {
            id: h.id, stock_id: h.stock_id, direction: h.direction,
            holding_date: h.holding_date, shares: h.shares, value: h.value,
            value_currency: h.value_currency, pct_of_float: h.pct_of_float,
            source: h.source,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnectHoldingsIn {
    pub stock_id: i64,
    pub direction: String,
    pub holding_date: String,
    #[schema(value_type = String)] pub shares: Decimal,
    #[schema(value_type = Option<String>)] pub value: Option<Decimal>,
    pub value_currency: Option<String>,
    #[schema(value_type = Option<String>)] pub pct_of_float: Option<Decimal>,
    #[serde(default = "default_source")] pub source: String,
}

fn default_source() -> String { "agent".into() }
