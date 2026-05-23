use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::market_briefs::LocalizedMarketBrief;

/// A pre- or post-market note for a country, written daily by the agent.
/// Upserts on `(user_id, country, kind, trade_date)`. `headline` and
/// `content_md` are projected from `content.<locale>` (with `en` fallback).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MarketBriefOut {
    /// Primary key.
    pub id: i64,
    /// ISO country code (`US` / `HK` / `CN`). Part of natural key.
    pub country: String,
    /// `pre_market` (before the open) or `post_market` (after the close).
    /// Part of natural key.
    pub kind: String,
    /// ISO date `YYYY-MM-DD` the brief is about. Part of natural key.
    pub trade_date: String,
    /// Localized headline.
    pub headline: Option<String>,
    /// Localized markdown body — typically 3-7 sentences.
    pub content_md: Option<String>,
    /// `bullish` / `bearish` / `neutral`. Pill in the UI.
    pub sentiment: Option<String>,
    /// Numeric sentiment, usually `[-1, 1]`.
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// Provenance.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
    /// RFC 3339 UTC timestamp when this user opened the item's detail
    /// page. `null` while the item is still unread.
    pub read_at: Option<String>,
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
            read_at: None,
        }
    }
}

/// `POST /market-briefs` body. Upserts against
/// `(user_id, country, kind, trade_date)`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MarketBriefIn {
    /// ISO country code.
    pub country: String,
    /// `pre_market` | `post_market`.
    pub kind: String,
    /// ISO date `YYYY-MM-DD`.
    pub trade_date: String,
    /// Sentiment label.
    pub sentiment: Option<String>,
    /// Numeric sentiment.
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "headline": "...", "content_md": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
