use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::Sector;

/// A sector taxonomy node. The agent typically uses a single scheme
/// (`custom` by default) but the table supports multiple side-by-side
/// (`gics`, `icb`, `custom`). Sectors form a tree via `parent_code`.
#[derive(Debug, Serialize, ToSchema)]
pub struct SectorOut {
    /// Primary key — the sector's short code (e.g. `semiconductors`).
    /// Unique within a `scheme`.
    pub code: String,
    /// Display name.
    pub name: String,
    /// FK-ish to the parent sector's `code`. `null` at the top of the
    /// tree.
    pub parent_code: Option<String>,
    /// Taxonomy this row belongs to — `custom` (default) | `gics` | `icb`.
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

/// `POST /sectors` body. Upserts by `code` within the `scheme`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SectorIn {
    /// Sector code.
    pub code: String,
    /// Display name.
    pub name: String,
    /// Parent code (must already exist in the same scheme).
    pub parent_code: Option<String>,
    /// Taxonomy. Default `custom`.
    #[serde(default = "default_scheme")]
    pub scheme: String,
}

fn default_scheme() -> String {
    "custom".into()
}
