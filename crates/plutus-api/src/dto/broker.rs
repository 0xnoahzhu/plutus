use serde::Serialize;
use utoipa::ToSchema;

use plutus_storage::models::Broker;

/// A broker reference row. Managed by admin via `/admin/brokers`; users
/// read via `/brokers` when picking the broker for a new account.
#[derive(Debug, Serialize, ToSchema)]
pub struct BrokerOut {
    /// Primary key.
    pub id: i64,
    /// Short code (e.g. `schwab`, `ibkr`, `futu`). Unique.
    pub code: String,
    /// Display name.
    pub name: String,
}

impl From<Broker> for BrokerOut {
    fn from(b: Broker) -> Self {
        Self {
            id: b.id,
            code: b.code,
            name: b.name,
        }
    }
}
