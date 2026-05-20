use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::InsiderTransaction;

/// One insider trade as reported in a Form 4 / equivalent. Shared across
/// users. Append-only — corrections are filed as new rows.
#[derive(Debug, Serialize, ToSchema)]
pub struct InsiderTxnOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Insider's full name.
    pub person_name: String,
    /// Role at the company — `ceo` | `cfo` | `director` | `10pct_owner` |
    /// etc. Free-form.
    pub role: Option<String>,
    /// `buy` | `sell` | `option_exercise` | `grant` | `gift`.
    pub txn_kind: String,
    /// Shares traded (positive for buy / exercise / grant; negative for
    /// sell / gift, by convention).
    #[schema(value_type = String)] pub shares: Decimal,
    /// Per-share price if known.
    #[schema(value_type = Option<String>)] pub price: Option<Decimal>,
    /// ISO-4217 currency.
    pub currency: Option<String>,
    /// ISO date `YYYY-MM-DD` the trade actually happened.
    pub executed_at: String,
    /// RFC 3339 UTC timestamp the form was filed (different from
    /// `executed_at` by a few days typically).
    pub filed_at: String,
    /// Provenance.
    pub source: String,
    /// Link to the filing (e.g. SEC EDGAR URL).
    pub source_url: Option<String>,
}

impl From<InsiderTransaction> for InsiderTxnOut {
    fn from(t: InsiderTransaction) -> Self {
        Self {
            id: t.id, stock_id: t.stock_id, person_name: t.person_name,
            role: t.role, txn_kind: t.txn_kind, shares: t.shares,
            price: t.price, currency: t.currency,
            executed_at: t.executed_at, filed_at: t.filed_at.to_string(),
            source: t.source, source_url: t.source_url,
        }
    }
}

/// `POST /insider/transactions` body. Always inserts.
#[derive(Debug, Deserialize, ToSchema)]
pub struct InsiderTxnIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Insider's full name.
    pub person_name: String,
    /// Role string.
    pub role: Option<String>,
    /// `buy` | `sell` | `option_exercise` | `grant` | `gift`.
    pub txn_kind: String,
    /// Shares (sign by convention).
    #[schema(value_type = String)] pub shares: Decimal,
    /// Per-share price.
    #[schema(value_type = Option<String>)] pub price: Option<Decimal>,
    /// ISO-4217 currency.
    pub currency: Option<String>,
    /// ISO date `YYYY-MM-DD`.
    pub executed_at: String,
    /// RFC 3339 UTC timestamp.
    pub filed_at: String,
    /// Default `agent`.
    #[serde(default = "default_source")] pub source: String,
    /// Source URL.
    pub source_url: Option<String>,
}

fn default_source() -> String { "agent".into() }
