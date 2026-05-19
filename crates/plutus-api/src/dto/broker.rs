use serde::Serialize;
use utoipa::ToSchema;

use plutus_storage::models::Broker;

#[derive(Debug, Serialize, ToSchema)]
pub struct BrokerOut {
    pub id: i64,
    pub code: String,
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
