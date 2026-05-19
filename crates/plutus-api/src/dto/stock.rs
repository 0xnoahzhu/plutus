use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::stocks::LocalizedStock;

/// One stock with translatable text already projected for the caller's
/// locale by the storage layer. `name` and `description_md` come from the
/// row's `content` JSONB blob; if the requested locale is missing a
/// particular field the storage layer falls back to `en`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StockOut {
    pub id: i64,
    pub market_code: String,
    pub symbol: String,
    pub isin: Option<String>,
    pub figi: Option<String>,
    pub currency: String,
    pub lot_size: Option<i32>,
    pub asset_class: String,
    pub sector_code: Option<String>,
    pub name: Option<String>,
    pub description_md: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LocalizedStock> for StockOut {
    fn from(s: LocalizedStock) -> Self {
        Self {
            id: s.id,
            market_code: s.market_code,
            symbol: s.symbol,
            isin: s.isin,
            figi: s.figi,
            currency: s.currency,
            lot_size: s.lot_size,
            asset_class: s.asset_class,
            sector_code: s.sector_code,
            name: s.name,
            description_md: s.description_md,
            created_at: s.created_at.to_string(),
            updated_at: s.updated_at.to_string(),
        }
    }
}

/// Create input. `content` is the full multi-locale blob —
/// `{ "<locale>": { "name": "...", "description_md": "..." } }`. The
/// storage layer writes it verbatim; partial-locale updates are the
/// caller's responsibility to merge before sending.
#[derive(Debug, Deserialize, ToSchema)]
pub struct StockIn {
    pub market_code: String,
    pub symbol: String,
    pub isin: Option<String>,
    pub figi: Option<String>,
    pub currency: String,
    pub lot_size: Option<i32>,
    pub asset_class: String,
    pub sector_code: Option<String>,
    pub content: serde_json::Value,
}

/// PATCH input. Only `content` is mutable through this route — the rest of
/// the columns (market_code, symbol, isin, …) are immutable post-create.
#[derive(Debug, Deserialize, ToSchema)]
pub struct StockPatch {
    /// Full multi-locale content blob replacing whatever is currently
    /// stored. Partial-locale updates are the caller's responsibility to
    /// merge before sending.
    pub content: Option<serde_json::Value>,
}
