//! Daily market briefs — pre-market analysis and post-market summary, one row
//! per (user_id, country, kind, trade_date). The natural key is enforced at
//! the app layer via the upsert query (toasty 0.6 doesn't model multi-column
//! UNIQUE in the derive).
//!
//! Translatable content (headline / content_md) lives in the `content` JSONB
//! column on the DB side. Because toasty 0.6 doesn't speak JSONB, the model
//! omits that column entirely — raw `tokio_postgres` SQL in
//! `queries::market_briefs` handles read/write of localized content. The
//! fields declared here are the metadata columns toasty can manage.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "market_briefs"]
pub struct MarketBrief {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    #[index]
    pub country: String, // "US" / "HK" / "CN" / "global"
    #[index]
    pub kind: String, // "pre_market" / "post_market"
    pub trade_date: String, // ISO "YYYY-MM-DD"
    pub sentiment: Option<String>, // "bullish" / "bearish" / "neutral"
    pub sentiment_score: Option<Decimal>,
    pub source: String, // "agent" / "manual"
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
