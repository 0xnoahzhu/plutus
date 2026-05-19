//! Every buy/sell/dividend/fee/etc. Stored as an immutable ledger; holdings
//! are derived. Currency-specific amounts are paired with their currency code.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "transactions"]
pub struct Transaction {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub account_id: i64,
    #[index]
    pub stock_id: Option<i64>, // null for cash-only entries (deposit, withdrawal, fx)
    pub kind: String,          // TransactionKind serialized as SCREAMING_SNAKE_CASE
    pub executed_at: jiff::Timestamp,
    pub quantity: Decimal,
    pub price: Decimal,
    pub trade_currency: String,
    pub commission: Decimal,
    pub commission_currency: String,
    pub tax: Decimal,
    pub tax_currency: String,
    pub fx_rate_to_base: Decimal,
    pub external_ref: Option<String>,
    pub notes: Option<String>,
    pub source: String,
    pub source_metadata: Option<String>, // JSON blob
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
