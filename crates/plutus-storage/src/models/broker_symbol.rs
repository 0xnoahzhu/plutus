//! Maps a broker-specific symbol (e.g. "AAPL.US" at Moomoo) to the canonical
//! stock row keyed by (market_code, symbol).

#[derive(Debug, toasty::Model)]
#[table = "broker_symbols"]
pub struct BrokerSymbol {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub broker_id: i64,
    #[index]
    pub stock_id: i64,
    pub broker_symbol: String,
}
