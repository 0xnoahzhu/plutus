use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{Watchlist, WatchlistItem};

/// Tri-state PATCH field: field absent (`None`) means "leave alone";
/// field set to JSON `null` (`Some(None)`) means "clear"; field set to a
/// value (`Some(Some(v))`) means "write this value".
fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WatchlistOut {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
}

impl From<Watchlist> for WatchlistOut {
    fn from(w: Watchlist) -> Self {
        Self {
            id: w.id,
            name: w.name,
            description: w.description,
            sort_order: w.sort_order,
            created_at: w.created_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WatchlistIn {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WatchlistPatch {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<String>)]
    pub description: Option<Option<String>>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WatchlistItemOut {
    pub watchlist_id: i64,
    pub stock_id: i64,
    pub added_at: String,
    pub notes: Option<String>,
}

impl From<WatchlistItem> for WatchlistItemOut {
    fn from(i: WatchlistItem) -> Self {
        Self {
            watchlist_id: i.watchlist_id,
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

/// Cross-watchlist stock view: a deduplicated stock plus the IDs of every
/// watchlist that contains it. Used by `GET /watchlists/stocks`.
#[derive(Debug, Serialize, ToSchema)]
pub struct WatchlistStockOut {
    pub id: i64,
    pub market_code: String,
    pub symbol: String,
    pub currency: String,
    pub asset_class: String,
    pub sector_code: Option<String>,
    /// Watchlist IDs that contain this stock, in ascending order.
    pub watchlist_ids: Vec<i64>,
}
