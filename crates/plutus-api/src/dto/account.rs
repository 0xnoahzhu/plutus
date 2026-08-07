use rust_decimal::Decimal;
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
    /// Known cash on hand at `cash_as_of`, in `base_currency`.
    ///
    /// Cash is derived from the ledger like holdings are, but a ledger
    /// that starts mid-life has buys with no matching deposits and would
    /// roll up negative. This anchors it: actual cash is this figure plus
    /// every ledger cash flow after `cash_as_of`. Set it from a broker
    /// statement via `PATCH /accounts/{id}`; `0` with a null
    /// `cash_as_of` means "the ledger alone is the truth".
    #[schema(value_type = String)]
    pub cash_balance: Decimal,
    /// RFC 3339 UTC timestamp `cash_balance` was true, or `null` for no
    /// anchor. Flows at or before this instant are already baked into
    /// the balance and are not counted again.
    pub cash_as_of: Option<String>,
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
            cash_balance: a.cash_balance,
            cash_as_of: a.cash_as_of.map(|t| t.to_string()),
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

/// `PATCH /accounts/{id}` body. All fields optional.
///
/// `broker_id`, `base_currency` and `account_number` are deliberately
/// not patchable — changing them would re-denominate or re-key an
/// account that transactions already point at. Delete and recreate if
/// you really need that.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AccountPatch {
    /// New display label.
    pub name: Option<String>,
    /// Known cash on hand, in the account's `base_currency`. Pair it
    /// with `cash_as_of` — setting a balance without saying when it was
    /// true makes every prior trade double-count.
    #[schema(value_type = Option<String>)]
    pub cash_balance: Option<Decimal>,
    /// RFC 3339 timestamp the balance was true. Send `null` to drop the
    /// anchor and let the whole ledger count again.
    #[serde(default, deserialize_with = "double_option_str")]
    pub cash_as_of: Option<Option<String>>,
}

/// See the note on `dto::transaction::double_option` — plain
/// `#[serde(default)]` on `Option<Option<T>>` collapses an explicit
/// `null` into the outer `None`, which would make "clear the anchor"
/// indistinguishable from "leave it alone".
fn double_option_str<'de, D>(de: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(de).map(Some)
}
