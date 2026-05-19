//! Many-to-many link tables from news_items to other entities. Each kept as a
//! separate table so the FK direction and semantic relation type can be
//! enforced cleanly.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "news_stock_links"]
pub struct NewsStockLink {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub news_id: i64,
    #[index]
    pub stock_id: i64,
    pub relation: String, // "primary" / "mentioned" / "peer" / "supplier" / "customer" / "competitor"
    pub relevance: Option<Decimal>, // 0.00 .. 1.00, agent confidence
}

#[derive(Debug, toasty::Model)]
#[table = "news_sector_links"]
pub struct NewsSectorLink {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub news_id: i64,
    #[index]
    pub sector_code: String,
}

#[derive(Debug, toasty::Model)]
#[table = "news_macro_links"]
pub struct NewsMacroLink {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub news_id: i64,
    #[index]
    pub indicator_code: String,
}

#[derive(Debug, toasty::Model)]
#[table = "news_country_links"]
pub struct NewsCountryLink {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub news_id: i64,
    pub country: String, // ISO alpha-2
}
