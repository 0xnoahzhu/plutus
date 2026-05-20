use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Account;

/// A brokerage account belonging to the user. Transactions and pending
/// orders attach to an account; holdings are derived per `(stock_id,
/// account_id)`.
#[derive(Debug, Serialize, ToSchema)]
pub struct AccountOut {
    /// Primary key.
    pub id: i64,
    /// FK to `brokers.id`. The user picks brokers from a list managed by
    /// the admin via `/admin/brokers`.
    pub broker_id: i64,
    /// User-chosen account label (e.g. "Schwab Roth IRA").
    pub name: String,
    /// Broker-side account number, optionally masked by the user before
    /// storing.
    pub account_number: Option<String>,
    /// ISO-4217 base currency for this account. All transactions get
    /// their `fx_rate_to_base` computed against this code.
    pub base_currency: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
}

impl From<Account> for AccountOut {
    fn from(a: Account) -> Self {
        Self {
            id: a.id,
            broker_id: a.broker_id,
            name: a.name,
            account_number: a.account_number,
            base_currency: a.base_currency,
            created_at: a.created_at.to_string(),
        }
    }
}

/// `POST /accounts` body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AccountIn {
    /// FK to `brokers.id`.
    pub broker_id: i64,
    /// User-chosen label.
    pub name: String,
    /// Optional account number.
    pub account_number: Option<String>,
    /// ISO-4217 currency code.
    pub base_currency: String,
}
