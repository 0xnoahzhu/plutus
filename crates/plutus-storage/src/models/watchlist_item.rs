//! The user's watchlist — a single flat list of stocks. Unique on stock_id
//! so the same ticker can't show up twice.

#[derive(Debug, toasty::Model)]
#[table = "watchlist_items"]
pub struct WatchlistItem {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    #[index]
    pub stock_id: i64,
    pub added_at: jiff::Timestamp,
    pub notes: Option<String>,
}
