use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::InsiderTransaction;

#[derive(Debug, Serialize, ToSchema)]
pub struct InsiderTxnOut {
    pub id: i64,
    pub stock_id: i64,
    pub person_name: String,
    pub role: Option<String>,
    pub txn_kind: String,
    #[schema(value_type = String)] pub shares: Decimal,
    #[schema(value_type = Option<String>)] pub price: Option<Decimal>,
    pub currency: Option<String>,
    pub executed_at: String,
    pub filed_at: String,
    pub source: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct InsiderTxnIn {
    pub stock_id: i64,
    pub person_name: String,
    pub role: Option<String>,
    pub txn_kind: String,
    #[schema(value_type = String)] pub shares: Decimal,
    #[schema(value_type = Option<String>)] pub price: Option<Decimal>,
    pub currency: Option<String>,
    pub executed_at: String,
    /// RFC 3339.
    pub filed_at: String,
    #[serde(default = "default_source")] pub source: String,
    pub source_url: Option<String>,
}

fn default_source() -> String { "agent".into() }
