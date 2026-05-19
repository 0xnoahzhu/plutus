use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Transaction;

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionOut {
    pub id: i64,
    pub account_id: i64,
    pub stock_id: Option<i64>,
    pub kind: String,
    pub executed_at: String,
    #[schema(value_type = String)]
    pub quantity: Decimal,
    #[schema(value_type = String)]
    pub price: Decimal,
    pub trade_currency: String,
    #[schema(value_type = String)]
    pub commission: Decimal,
    pub commission_currency: String,
    #[schema(value_type = String)]
    pub tax: Decimal,
    pub tax_currency: String,
    #[schema(value_type = String)]
    pub fx_rate_to_base: Decimal,
    pub external_ref: Option<String>,
    pub notes: Option<String>,
    pub source: String,
    pub source_metadata: Option<String>,
    pub created_at: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct TransactionIn {
    pub account_id: i64,
    pub stock_id: Option<i64>,
    pub kind: String,
    /// RFC 3339 timestamp.
    pub executed_at: String,
    #[schema(value_type = String)]
    pub quantity: Decimal,
    #[schema(value_type = String)]
    pub price: Decimal,
    pub trade_currency: String,
    #[serde(default)]
    #[schema(value_type = String)]
    pub commission: Decimal,
    pub commission_currency: String,
    #[serde(default)]
    #[schema(value_type = String)]
    pub tax: Decimal,
    pub tax_currency: String,
    #[schema(value_type = String)]
    pub fx_rate_to_base: Decimal,
    pub external_ref: Option<String>,
    pub notes: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    pub source_metadata: Option<serde_json::Value>,
}

fn default_source() -> String {
    "agent".into()
}
