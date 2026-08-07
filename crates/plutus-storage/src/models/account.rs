//! A specific account at a broker. `account_number` is optional and stored
//! plain text — the user accepted this trade-off in Phase 0.
//!
//! ## The cash anchor
//!
//! Cash is *derived*, like holdings — but unlike holdings it can't be
//! derived from nothing. A ledger that starts mid-life (the common case:
//! you begin recording trades on a funded account) has buys with no
//! matching deposits, so a pure rollup lands deeply negative.
//!
//! `cash_balance` + `cash_as_of` fix that with an anchor: "this account
//! held X at time T". Cash from then on is `X` plus every ledger cash
//! flow after `T`. Set the anchor once from your broker's statement and
//! ongoing deposits/withdrawals/trades keep it current on their own. A
//! `cash_as_of` of `None` means "count the whole ledger", which is the
//! right setting for an account recorded from day one.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "accounts"]
pub struct Account {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    #[index]
    pub broker_id: i64,
    pub name: String,
    pub account_number: Option<String>,
    pub base_currency: String,
    /// Known cash on hand at `cash_as_of`, in `base_currency`. Defaults
    /// to 0, which combined with a null `cash_as_of` means "cash is
    /// exactly what the ledger says".
    pub cash_balance: Decimal,
    /// When `cash_balance` was true. Ledger flows at or before this
    /// instant are already baked into it and must not be counted again.
    /// `None` = no anchor, count everything.
    pub cash_as_of: Option<jiff::Timestamp>,
    pub created_at: jiff::Timestamp,
}
