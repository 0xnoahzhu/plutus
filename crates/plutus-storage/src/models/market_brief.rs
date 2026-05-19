//! Daily market briefs — pre-market analysis and post-market summary, one row
//! per (country, kind, trade_date). The natural key is enforced at the app
//! layer via the upsert query (toasty 0.6 doesn't model multi-column UNIQUE
//! in the derive).

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
    pub headline: String,
    pub content_md: Option<String>,
    pub sentiment: Option<String>, // "bullish" / "bearish" / "neutral"
    pub sentiment_score: Option<Decimal>,
    pub source: String,   // "agent" / "manual"
    pub language: String, // "en" / "zh-CN" — source language of the base columns
    /// JSON: `{ "zh-CN": { "headline": "…", "content_md": "…" } }`. Handlers
    /// merge these into the base columns when `?locale=` is set.
    pub translations: Option<String>,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
