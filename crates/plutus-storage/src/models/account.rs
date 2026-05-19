//! A specific account at a broker. `account_number` is optional and stored
//! plain text — the user accepted this trade-off in Phase 0.

#[derive(Debug, toasty::Model)]
#[table = "accounts"]
pub struct Account {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub broker_id: i64,
    pub name: String,
    pub account_number: Option<String>,
    pub base_currency: String,
    pub created_at: jiff::Timestamp,
}
