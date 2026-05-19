use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::MarketBrief;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MarketBriefOut {
    pub id: i64,
    pub country: String,
    pub kind: String,
    pub trade_date: String,
    pub headline: String,
    pub content_md: Option<String>,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    pub source: String,
    pub language: String,
    /// Raw JSON: `{ "zh-CN": { "headline": "...", "content_md": "..." } }`.
    /// Returned alongside the (possibly localized) base columns so the agent
    /// can read/update the full translation set.
    pub translations: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<MarketBrief> for MarketBriefOut {
    fn from(b: MarketBrief) -> Self {
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
            language: b.language,
            translations: b.translations,
            created_at: b.created_at.to_string(),
            updated_at: b.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarketBriefIn {
    pub country: String,
    /// "pre_market" or "post_market".
    pub kind: String,
    /// ISO YYYY-MM-DD.
    pub trade_date: String,
    pub headline: String,
    pub content_md: Option<String>,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default = "default_language")]
    pub language: String,
    pub translations: Option<serde_json::Value>,
}

fn default_source() -> String { "agent".into() }
fn default_language() -> String { "en".into() }
