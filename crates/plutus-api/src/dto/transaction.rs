use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Transaction;

/// One executed trade. The ledger is append-only — to correct a mistake,
/// post a compensating row (negative `quantity` for a buy, etc.). `holdings`
/// are derived purely from this table by the storage layer; there's no
/// `holdings` table.
///
/// Multi-currency: every monetary leg has its own currency column
/// (`trade_currency` for `price`, `commission_currency` for `commission`,
/// `tax_currency` for `tax`). `fx_rate_to_base` converts the trade_currency
/// to the user's base currency at execution time so `holdings` can roll
/// up in one currency.
#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionOut {
    /// Primary key.
    pub id: i64,
    /// FK to `accounts.id`. Bound to a user via the parent account.
    pub account_id: i64,
    /// FK to `stocks.id`. `null` for non-stock entries (cash deposits /
    /// withdrawals, dividends paid in cash).
    pub stock_id: Option<i64>,
    /// Transaction type — canonical form is `SCREAMING_SNAKE_CASE`. The
    /// API accepts these values case-insensitively: `BUY`, `SELL`,
    /// `DIVIDEND`, `FEE`, `INTEREST`, `DEPOSIT`, `WITHDRAWAL` (alias
    /// `WITHDRAW`), `FX`, `CORPORATE_ACTION`. Unknown values return 400.
    /// Stored canonically in upper form regardless of input.
    ///
    /// Only `BUY`, `SELL`, and `CORPORATE_ACTION` move share quantities
    /// (and roll up into `/holdings`); everything else is cash-only.
    pub kind: String,
    /// RFC 3339 UTC timestamp the trade settled / cash moved.
    pub executed_at: String,
    /// Shares (or cash amount for `DEPOSIT`/`WITHDRAWAL`). Decimal so
    /// fractional shares survive.
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// Per-share execution price (or `1.00` for cash entries).
    #[schema(value_type = String)]
    pub price: Decimal,
    /// ISO-4217 currency of `price`.
    pub trade_currency: String,
    /// Broker commission. Defaults to `0`.
    #[schema(value_type = String)]
    pub commission: Decimal,
    /// ISO-4217 currency of `commission`.
    pub commission_currency: String,
    /// Tax withheld. Defaults to `0`.
    #[schema(value_type = String)]
    pub tax: Decimal,
    /// ISO-4217 currency of `tax`.
    pub tax_currency: String,
    /// FX rate from `trade_currency` to the user's base currency at
    /// `executed_at`. Captures the conversion rate so historical holdings
    /// roll up correctly even after FX rates move.
    #[schema(value_type = String)]
    pub fx_rate_to_base: Decimal,
    /// Broker's confirmation id, if known. Useful for reconciling against
    /// statements.
    pub external_ref: Option<String>,
    /// Free-form notes from the user / agent.
    pub notes: Option<String>,
    /// `agent` (default), `manual`, broker name.
    pub source: String,
    /// JSON-stringified per-source metadata (e.g. broker-specific fields
    /// you don't want to model first-class).
    pub source_metadata: Option<String>,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

impl From<Transaction> for TransactionOut {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id,
            account_id: t.account_id,
            stock_id: t.stock_id,
            kind: t.kind,
            executed_at: t.executed_at.to_string(),
            quantity: t.quantity,
            price: t.price,
            trade_currency: t.trade_currency,
            commission: t.commission,
            commission_currency: t.commission_currency,
            tax: t.tax,
            tax_currency: t.tax_currency,
            fx_rate_to_base: t.fx_rate_to_base,
            external_ref: t.external_ref,
            notes: t.notes,
            source: t.source,
            source_metadata: t.source_metadata,
            created_at: t.created_at.to_string(),
            updated_at: t.updated_at.to_string(),
        }
    }
}

/// `POST /transactions` body. Always inserts a new row — there's no
/// upsert. To correct a mistake, POST a compensating transaction (e.g.
/// `quantity = -10` to undo a 10-share buy).
#[derive(Debug, Deserialize, ToSchema)]
pub struct TransactionIn {
    /// FK to `accounts.id`. The account must belong to the caller.
    pub account_id: i64,
    /// FK to `stocks.id`. `null` for cash entries.
    pub stock_id: Option<i64>,
    /// Transaction type, canonical `SCREAMING_SNAKE_CASE`. Accepts
    /// case-insensitively: `BUY`, `SELL`, `DIVIDEND`, `FEE`, `INTEREST`,
    /// `DEPOSIT`, `WITHDRAWAL` (alias `WITHDRAW`), `FX`,
    /// `CORPORATE_ACTION`. Unknown values return 400. Stored canonically
    /// in upper form regardless of input.
    ///
    /// Only `BUY`, `SELL`, and `CORPORATE_ACTION` roll up into
    /// `/holdings`; the others are cash-only.
    pub kind: String,
    /// RFC 3339 UTC timestamp.
    pub executed_at: String,
    /// Signed share count for share-moving kinds: positive for `BUY` /
    /// `CORPORATE_ACTION` (add) and negative for `SELL` (subtract from
    /// position). Cash amount for `DEPOSIT` / `WITHDRAWAL` /
    /// `DIVIDEND` / `FEE` / `INTEREST` / `FX`.
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// Per-share price or `1.00` for cash entries.
    #[schema(value_type = String)]
    pub price: Decimal,
    /// ISO-4217 currency of `price`.
    pub trade_currency: String,
    /// Defaults to `0`.
    #[serde(default)]
    #[schema(value_type = String)]
    pub commission: Decimal,
    /// ISO-4217 currency of `commission`.
    pub commission_currency: String,
    /// Defaults to `0`.
    #[serde(default)]
    #[schema(value_type = String)]
    pub tax: Decimal,
    /// ISO-4217 currency of `tax`.
    pub tax_currency: String,
    /// FX rate from `trade_currency` to the user's base currency at
    /// `executed_at`. Required even when `trade_currency` is already the
    /// base currency (send `1.0` in that case).
    #[schema(value_type = String)]
    pub fx_rate_to_base: Decimal,
    /// Broker confirmation id.
    pub external_ref: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Per-source metadata as a JSON object; the server stringifies.
    pub source_metadata: Option<serde_json::Value>,
}

fn default_source() -> String {
    "agent".into()
}
