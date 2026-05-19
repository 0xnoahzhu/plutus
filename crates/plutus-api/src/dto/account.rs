use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Account;

#[derive(Debug, Serialize, ToSchema)]
pub struct AccountOut {
    pub id: i64,
    pub broker_id: i64,
    pub name: String,
    pub account_number: Option<String>,
    pub base_currency: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct AccountIn {
    pub broker_id: i64,
    pub name: String,
    pub account_number: Option<String>,
    pub base_currency: String,
}
