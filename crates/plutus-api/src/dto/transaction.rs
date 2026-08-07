use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Transaction;
use plutus_storage::queries::transactions::StockSummary;

/// One executed trade. `holdings` are derived purely from this table by
/// the storage layer; there's no `holdings` table.
///
/// Two ways to fix a row. A posting that genuinely happened gets reversed
/// with a compensating entry (negative `quantity` for a buy, etc.) so the
/// history stays honest. A row that was simply mistyped gets corrected in
/// place with `PATCH /transactions/{id}`.
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
/// upsert. Two ways to fix a bad row: `PATCH /transactions/{id}` when it
/// was a typo, or a compensating entry (`quantity = -10` to undo a
/// 10-share buy) when the original posting was itself a real event you
/// want to keep in the history.
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

/// `PATCH /transactions/{id}` body. Every field is optional — omit a key
/// to leave the column alone. Nullable columns (`stock_id`,
/// `external_ref`, `notes`, `source_metadata`) accept an explicit `null`
/// to clear them, which is why they're `Option<Option<T>>`.
///
/// Editing is safe against the derived views: `/holdings`,
/// `/portfolio/value-series` and the per-stock summary all recompute
/// from this table on every read, so a corrected row shows up
/// immediately with nothing to invalidate.
#[derive(Debug, Deserialize, ToSchema)]
pub struct TransactionPatch {
    /// Move the row to a different account. Must belong to the caller.
    pub account_id: Option<i64>,
    /// Re-point at another stock, or send `null` to turn the row into a
    /// cash-only entry.
    #[serde(default, deserialize_with = "double_option")]
    pub stock_id: Option<Option<i64>>,
    /// Same vocabulary and case-insensitivity as `POST` — `BUY`, `SELL`,
    /// `DIVIDEND`, `FEE`, `INTEREST`, `DEPOSIT`, `WITHDRAWAL` (alias
    /// `WITHDRAW`), `FX`, `CORPORATE_ACTION`. Stored canonically upper.
    pub kind: Option<String>,
    /// RFC 3339 UTC timestamp.
    pub executed_at: Option<String>,
    #[schema(value_type = Option<String>)]
    pub quantity: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub price: Option<Decimal>,
    /// ISO-4217 currency of `price`.
    pub trade_currency: Option<String>,
    #[schema(value_type = Option<String>)]
    pub commission: Option<Decimal>,
    /// ISO-4217 currency of `commission`.
    pub commission_currency: Option<String>,
    #[schema(value_type = Option<String>)]
    pub tax: Option<Decimal>,
    /// ISO-4217 currency of `tax`.
    pub tax_currency: Option<String>,
    #[schema(value_type = Option<String>)]
    pub fx_rate_to_base: Option<Decimal>,
    /// Broker confirmation id; send `null` to clear.
    #[serde(default, deserialize_with = "double_option")]
    pub external_ref: Option<Option<String>>,
    /// Free-form notes; send `null` to clear.
    #[serde(default, deserialize_with = "double_option")]
    pub notes: Option<Option<String>>,
    /// `agent` / `manual` / broker name.
    pub source: Option<String>,
    /// Per-source metadata as a JSON object; the server stringifies.
    /// Send `null` to clear.
    #[serde(default, deserialize_with = "double_option")]
    pub source_metadata: Option<Option<serde_json::Value>>,
}

/// serde's `Option<Option<T>>` handling collapses a literal `null` to the
/// outer `None` under `#[serde(default)]` alone, which would make
/// "clear this column" indistinguishable from "leave it alone". Routing
/// the field through an explicit deserializer keeps the two apart:
/// key absent → outer `None`, key present as `null` → `Some(None)`.
fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Option::<T>::deserialize(de).map(Some)
}

/// Everything the ledger knows about one stock, rolled up — the numbers
/// behind the "Transactions" panel on a stock's detail page.
///
/// Derived on read from the same rows `GET /transactions?stock_id=`
/// returns; there is no summary table. The open-position fields
/// (`quantity`, `avg_cost_trade`, `cost_base`, `realized_pnl_base`)
/// agree exactly with the matching `/holdings` row for the same
/// `method`.
#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionSummaryOut {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Echoes the `account_id` filter, or `null` for a rollup across
    /// every account the caller owns.
    pub account_id: Option<i64>,
    /// Cost-basis method the position fields were computed with:
    /// `fifo` (default) | `lifo` | `average`.
    pub method: String,
    /// Every ledger row touching this stock — including the cash-only
    /// kinds (dividends, fees) that leave the share count alone.
    pub transaction_count: i64,
    /// Rows that added shares / removed shares respectively.
    pub buy_count: i64,
    pub sell_count: i64,
    /// Gross shares acquired and disposed over the whole history. Not
    /// netted — a buy-then-sell round trip counts in both.
    #[schema(value_type = String)]
    pub buy_quantity: Decimal,
    #[schema(value_type = String)]
    pub sell_quantity: Decimal,
    /// Gross cash in / out in the trade currency, before commission.
    /// Only meaningful when `trade_currency` below is non-null; when the
    /// history mixes currencies these sum unlike units, so prefer the
    /// `_base` pair.
    #[schema(value_type = String)]
    pub buy_amount_trade: Decimal,
    #[schema(value_type = String)]
    pub sell_amount_trade: Decimal,
    /// The same gross amounts with each row's `fx_rate_to_base` applied,
    /// so they're always comparable.
    #[schema(value_type = String)]
    pub buy_amount_base: Decimal,
    #[schema(value_type = String)]
    pub sell_amount_base: Decimal,
    /// Lifetime commission and tax across every row, in base currency.
    /// Both use the trade row's `fx_rate_to_base` — the ledger carries no
    /// separate rate for the commission and tax legs.
    #[schema(value_type = String)]
    pub commission_total_base: Decimal,
    #[schema(value_type = String)]
    pub tax_total_base: Decimal,
    /// Net open shares right now (positive = long).
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// Weighted-average cost per open share, in trade currency.
    #[schema(value_type = String)]
    pub avg_cost_trade: Decimal,
    /// Cost basis of the open shares, in base currency.
    #[schema(value_type = String)]
    pub cost_base: Decimal,
    /// Realized P&L from closed legs, base currency, lifetime.
    #[schema(value_type = String)]
    pub realized_pnl_base: Decimal,
    /// RFC 3339 UTC timestamps bounding the history, or `null` when the
    /// stock has no transactions at all.
    pub first_executed_at: Option<String>,
    pub last_executed_at: Option<String>,
    /// The single ISO-4217 code shared by every contributing row, or
    /// `null` when the history spans more than one currency.
    pub trade_currency: Option<String>,
}

impl TransactionSummaryOut {
    /// Flatten the storage-layer rollup, splicing the nested `position`
    /// up to the top level so callers read one flat object.
    pub fn from_summary(s: StockSummary, method: &str) -> Self {
        Self {
            stock_id: s.stock_id,
            account_id: s.account_id,
            method: method.to_string(),
            transaction_count: s.transaction_count,
            buy_count: s.buy_count,
            sell_count: s.sell_count,
            buy_quantity: s.buy_quantity,
            sell_quantity: s.sell_quantity,
            buy_amount_trade: s.buy_amount_trade,
            sell_amount_trade: s.sell_amount_trade,
            buy_amount_base: s.buy_amount_base,
            sell_amount_base: s.sell_amount_base,
            commission_total_base: s.commission_total_base,
            tax_total_base: s.tax_total_base,
            quantity: s.position.quantity,
            avg_cost_trade: s.position.avg_cost_trade,
            cost_base: s.position.cost_base,
            realized_pnl_base: s.position.realized_pnl_base,
            first_executed_at: s.first_executed_at.map(|t| t.to_string()),
            last_executed_at: s.last_executed_at.map(|t| t.to_string()),
            trade_currency: s.trade_currency,
        }
    }
}
