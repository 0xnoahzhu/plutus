use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Sector;

#[derive(Debug, Serialize, ToSchema)]
pub struct SectorOut {
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    pub scheme: String,
}

impl From<Sector> for SectorOut {
    fn from(s: Sector) -> Self {
        Self {
            code: s.code,
            name: s.name,
            parent_code: s.parent_code,
            scheme: s.scheme,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SectorIn {
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    #[serde(default = "default_scheme")]
    pub scheme: String,
}

fn default_scheme() -> String {
    "custom".into()
}
