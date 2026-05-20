use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{ConnectFlowDaily, ConnectHoldingsDaily};

/// HK Stock Connect (Shanghai/Shenzhen ↔ Hong Kong) daily aggregate flow.
/// Used by the agent to track north-bound / south-bound buying pressure
/// from mainland Chinese investors as a sentiment indicator. Shared
/// across users.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectFlowOut {
    /// Primary key.
    pub id: i64,
    /// Market the flow is **into** — `hk` for southbound, `cn_a` for
    /// northbound. Combined with `direction` to identify the channel.
    pub market_code: String,
    /// `northbound` (mainland buying HK-listed Stock Connect names — wait,
    /// no: northbound is HK money going into Shanghai/Shenzhen) |
    /// `southbound` (mainland money going into HK). Agent picks the
    /// convention.
    pub direction: String,
    /// ISO date `YYYY-MM-DD`.
    pub flow_date: String,
    /// Net buy (buy − sell). Positive = inflow.
    #[schema(value_type = String)] pub net_buy: Decimal,
    /// ISO-4217 currency of `net_buy`.
    pub net_buy_currency: String,
    /// Gross buys.
    #[schema(value_type = Option<String>)] pub total_buy: Option<Decimal>,
    /// Gross sells.
    #[schema(value_type = Option<String>)] pub total_sell: Option<Decimal>,
    /// Remaining daily quota in the channel.
    #[schema(value_type = Option<String>)] pub quota_balance: Option<Decimal>,
    /// Provenance.
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

/// `POST /connect/flow` body. Upserts on
/// `(market_code, direction, flow_date)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnectFlowIn {
    /// Target market code.
    pub market_code: String,
    /// `northbound` | `southbound`.
    pub direction: String,
    /// ISO date `YYYY-MM-DD`.
    pub flow_date: String,
    /// Net buy.
    #[schema(value_type = String)] pub net_buy: Decimal,
    /// ISO-4217 currency.
    pub net_buy_currency: String,
    /// Gross buy.
    #[schema(value_type = Option<String>)] pub total_buy: Option<Decimal>,
    /// Gross sell.
    #[schema(value_type = Option<String>)] pub total_sell: Option<Decimal>,
    /// Remaining daily quota.
    #[schema(value_type = Option<String>)] pub quota_balance: Option<Decimal>,
    /// Default `agent`.
    #[serde(default = "default_source")] pub source: String,
}

/// Per-stock Stock Connect cumulative holdings as of a day. Tracks how
/// much of each Connect-eligible name is held via the channel.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectHoldingsOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// `northbound` | `southbound`.
    pub direction: String,
    /// ISO date `YYYY-MM-DD`.
    pub holding_date: String,
    /// Cumulative shares held via Connect.
    #[schema(value_type = String)] pub shares: Decimal,
    /// Market value (shares × price).
    #[schema(value_type = Option<String>)] pub value: Option<Decimal>,
    /// ISO-4217 currency of `value`.
    pub value_currency: Option<String>,
    /// Percent of free float held via Connect.
    #[schema(value_type = Option<String>)] pub pct_of_float: Option<Decimal>,
    /// Provenance.
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

/// `POST /connect/holdings` body. Upserts on
/// `(stock_id, direction, holding_date)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnectHoldingsIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// `northbound` | `southbound`.
    pub direction: String,
    /// ISO date.
    pub holding_date: String,
    /// Cumulative shares.
    #[schema(value_type = String)] pub shares: Decimal,
    /// Market value.
    #[schema(value_type = Option<String>)] pub value: Option<Decimal>,
    /// ISO-4217 currency.
    pub value_currency: Option<String>,
    /// Percent of free float.
    #[schema(value_type = Option<String>)] pub pct_of_float: Option<Decimal>,
    /// Default `agent`.
    #[serde(default = "default_source")] pub source: String,
}

fn default_source() -> String { "agent".into() }
