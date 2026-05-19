//! Daily OHLCV. Natural key (stock_id, trade_date) enforced at app layer.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "ohlcv_daily"]
pub struct OhlcvDaily {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub trade_date: String, // ISO date "YYYY-MM-DD" — toasty 0.6 has no native date type yet
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub adjusted_close: Option<Decimal>,
    pub volume: i64,
    pub source: String,
    pub created_at: jiff::Timestamp,
}
