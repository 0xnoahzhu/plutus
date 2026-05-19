//! Insider transactions: SEC Form 4 (US), HKEX 大股东权益披露 (HK), CSRC 大股东
//! 增减持公告 (CN), and 10%-holder filings.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "insider_transactions"]
pub struct InsiderTransaction {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub person_name: String,
    pub role: Option<String>, // "CEO" / "CFO" / "Director" / "10% Holder" / "General Counsel"
    pub txn_kind: String,
    // ^ "buy" / "sell" / "option_exercise" / "gift" / "automatic_sell"
    //   "indirect_buy" / "indirect_sell"
    pub shares: Decimal,
    pub price: Option<Decimal>,
    pub currency: Option<String>,
    pub executed_at: String, // ISO date
    pub filed_at: jiff::Timestamp,
    pub source: String,
    pub source_url: Option<String>,
    pub created_at: jiff::Timestamp,
}
