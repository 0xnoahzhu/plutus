use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::WatchlistItem;

/// A single stock pinned on the user's watchlist. The watchlist is flat
/// (no folders). Scoped per-user — caller only sees their own items.
#[derive(Debug, Serialize, ToSchema)]
pub struct WatchlistItemOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`. Unique per user — adding the same stock twice
    /// returns the existing row instead of duplicating.
    pub stock_id: i64,
    /// RFC 3339 UTC timestamp the user added this stock.
    pub added_at: String,
    /// Free-form notes from the user.
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

/// `POST /watchlist/items` body. Adding a stock that's already on the
/// watchlist is a no-op (returns the existing item, not a 409).
#[derive(Debug, Deserialize, ToSchema)]
pub struct WatchlistItemIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// Optional free-form note attached at add-time.
    pub notes: Option<String>,
}
