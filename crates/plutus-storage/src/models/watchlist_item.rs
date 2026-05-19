//! Membership of a stock in a watchlist. Natural key (watchlist_id, stock_id)
//! enforced at app layer.

#[derive(Debug, toasty::Model)]
#[table = "watchlist_items"]
pub struct WatchlistItem {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub watchlist_id: i64,
    #[index]
    pub stock_id: i64,
    pub added_at: jiff::Timestamp,
    pub notes: Option<String>,
}
