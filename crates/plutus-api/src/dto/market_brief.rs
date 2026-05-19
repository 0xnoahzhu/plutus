use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::market_briefs::LocalizedMarketBrief;

/// One market brief, with translatable text already projected for the
/// caller's locale by the storage layer. `headline` / `content_md` come from
/// the row's `content` JSONB blob; if the requested locale is missing a
/// particular field the storage layer falls back to `en`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MarketBriefOut {
    pub id: i64,
    pub country: String,
    pub kind: String,
    pub trade_date: String,
    pub headline: Option<String>,
    pub content_md: Option<String>,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LocalizedMarketBrief> for MarketBriefOut {
    fn from(b: LocalizedMarketBrief) -> Self {
        Self {
            id: b.id,
            country: b.country,
            kind: b.kind,
            trade_date: b.trade_date,
            headline: b.headline,
            content_md: b.content_md,
            sentiment: b.sentiment,
            sentiment_score: b.sentiment_score,
            source: b.source,
            created_at: b.created_at.to_string(),
            updated_at: b.updated_at.to_string(),
        }
    }
}

/// Upsert input. `content` is the full multi-locale blob —
/// `{ "<locale>": { "headline": "...", "content_md": "..." } }`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MarketBriefIn {
    pub country: String,
    /// "pre_market" or "post_market".
    pub kind: String,
    /// ISO YYYY-MM-DD.
    pub trade_date: String,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    #[serde(default = "default_source")]
    pub source: String,
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
