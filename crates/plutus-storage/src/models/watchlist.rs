//! A named bucket of stocks the user is interested in. The user can have many.

#[derive(Debug, toasty::Model)]
#[table = "watchlists"]
pub struct Watchlist {
    #[key]
    #[auto]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: jiff::Timestamp,
}
