use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::WatchlistItem;

#[derive(Debug, Serialize, ToSchema)]
pub struct WatchlistItemOut {
    pub id: i64,
    pub stock_id: i64,
    pub added_at: String,
    pub notes: Option<String>,
}

impl From<WatchlistItem> for WatchlistItemOut {
    fn from(i: WatchlistItem) -> Self {
        Self {
            id: i.id,
            stock_id: i.stock_id,
            added_at: i.added_at.to_string(),
            notes: i.notes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WatchlistItemIn {
    pub stock_id: i64,
    pub notes: Option<String>,
}
