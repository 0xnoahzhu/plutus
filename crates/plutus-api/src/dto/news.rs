use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{NewsCountryLink, NewsMacroLink, NewsSectorLink, NewsStockLink};
use plutus_storage::queries::news::LocalizedNewsItem;

/// One news item with translatable text already projected for the caller's
/// locale by the storage layer. `title` / `summary` / `content_md` /
/// `agent_summary_md` come from the row's `content` JSONB blob; if the
/// requested locale is missing a particular field the storage layer falls
/// back to `en`.
#[derive(Debug, Serialize, ToSchema)]
pub struct NewsOut {
    pub id: i64,
    pub external_id: Option<String>,
    pub url: String,
    pub canonical_url: Option<String>,
    pub archive_url: Option<String>,
    pub url_status: Option<i32>,
    pub last_verified_at: Option<String>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub content_md: Option<String>,
    pub agent_summary_md: Option<String>,
    pub source: String,
    pub source_kind: String,
    pub category: String,
    pub region: String,
    pub published_at: String,
    pub fetched_at: String,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    pub importance: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LocalizedNewsItem> for NewsOut {
    fn from(n: LocalizedNewsItem) -> Self {
        Self {
            id: n.id,
            external_id: n.external_id,
            url: n.url,
            canonical_url: n.canonical_url,
            archive_url: n.archive_url,
            url_status: n.url_status,
            last_verified_at: n.last_verified_at.map(|t| t.to_string()),
            title: n.title,
            summary: n.summary,
            content_md: n.content_md,
            agent_summary_md: n.agent_summary_md,
            source: n.source,
            source_kind: n.source_kind,
            category: n.category,
            region: n.region,
            published_at: n.published_at.to_string(),
            fetched_at: n.fetched_at.to_string(),
            sentiment: n.sentiment,
            sentiment_score: n.sentiment_score,
            importance: n.importance,
            created_at: n.created_at.to_string(),
            updated_at: n.updated_at.to_string(),
        }
    }
}

/// Create input. `content` is the full multi-locale blob —
/// `{ "<locale>": { "title": "...", "summary": "...", "content_md": "...",
///                   "agent_summary_md": "..." } }`. The storage layer
/// writes it verbatim; partial-locale updates are the caller's
/// responsibility to merge before sending.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsIn {
    pub external_id: Option<String>,
    pub url: String,
    pub canonical_url: Option<String>,
    pub archive_url: Option<String>,
    pub url_status: Option<i32>,
    pub last_verified_at: Option<String>, // RFC 3339
    pub source: String,
    #[serde(default = "default_source_kind")]
    pub source_kind: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_region")]
    pub region: String,
    /// RFC 3339 timestamp.
    pub published_at: String,
    pub fetched_at: Option<String>,
    pub sentiment: Option<String>,
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    #[serde(default = "default_importance")]
    pub importance: String,
    pub content: serde_json::Value,
}

fn default_source_kind() -> String { "news".into() }
fn default_category() -> String { "company".into() }
fn default_region() -> String { "global".into() }
fn default_importance() -> String { "medium".into() }

// ── Link DTOs ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct NewsStockLinkOut {
    pub id: i64,
    pub news_id: i64,
    pub stock_id: i64,
    pub relation: String,
    #[schema(value_type = Option<String>)]
    pub relevance: Option<Decimal>,
}

impl From<NewsStockLink> for NewsStockLinkOut {
    fn from(l: NewsStockLink) -> Self {
        Self {
            id: l.id,
            news_id: l.news_id,
            stock_id: l.stock_id,
            relation: l.relation,
            relevance: l.relevance,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsStockLinkIn {
    pub stock_id: i64,
    #[serde(default = "default_relation")]
    pub relation: String,
    #[schema(value_type = Option<String>)]
    pub relevance: Option<Decimal>,
}

fn default_relation() -> String { "primary".into() }

#[derive(Debug, Serialize, ToSchema)]
pub struct NewsSectorLinkOut {
    pub id: i64,
    pub news_id: i64,
    pub sector_code: String,
}

impl From<NewsSectorLink> for NewsSectorLinkOut {
    fn from(l: NewsSectorLink) -> Self {
        Self { id: l.id, news_id: l.news_id, sector_code: l.sector_code }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsSectorLinkIn { pub sector_code: String }

#[derive(Debug, Serialize, ToSchema)]
pub struct NewsMacroLinkOut {
    pub id: i64,
    pub news_id: i64,
    pub indicator_code: String,
}

impl From<NewsMacroLink> for NewsMacroLinkOut {
    fn from(l: NewsMacroLink) -> Self {
        Self { id: l.id, news_id: l.news_id, indicator_code: l.indicator_code }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsMacroLinkIn { pub indicator_code: String }

#[derive(Debug, Serialize, ToSchema)]
pub struct NewsCountryLinkOut {
    pub id: i64,
    pub news_id: i64,
    pub country: String,
}

impl From<NewsCountryLink> for NewsCountryLinkOut {
    fn from(l: NewsCountryLink) -> Self {
        Self { id: l.id, news_id: l.news_id, country: l.country }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsCountryLinkIn { pub country: String }
