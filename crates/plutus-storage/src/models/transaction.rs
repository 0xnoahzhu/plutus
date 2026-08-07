//! Every buy/sell/dividend/fee/etc. The ledger every derived view reads
//! from — holdings, the portfolio value series, and the per-stock summary
//! are all recomputed from these rows rather than stored. Currency-specific
//! amounts are paired with their currency code.
//!
//! Rows are normally append-only: a posting that really happened gets
//! reversed by a compensating entry, not erased. `queries::transactions::update`
//! is the escape hatch for the other case — a typo in a hand-entered row.
//! It's safe precisely because nothing caches the rollup.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "transactions"]
pub struct Transaction {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
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
