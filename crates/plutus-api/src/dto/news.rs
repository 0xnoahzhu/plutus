use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{NewsCountryLink, NewsMacroLink, NewsSectorLink, NewsStockLink};
use plutus_storage::queries::news::LocalizedNewsItem;

/// A news item the agent ingested or hand-added. Shared across users
/// (reference data). Related entities (stock / sector / macro indicator /
/// country) are attached via the link tables — `news_stock_links`,
/// `news_sector_links`, `news_macro_links`, `news_country_links`.
///
/// Translatable fields (`title`, `summary`, `content_md`,
/// `agent_summary_md`) come from `content.<locale>`.
#[derive(Debug, Serialize, ToSchema)]
pub struct NewsOut {
    /// Primary key.
    pub id: i64,
    /// Vendor / scraper id, if any. Useful for dedup before POSTing.
    pub external_id: Option<String>,
    /// Source URL (article page).
    pub url: String,
    /// Canonical URL after redirect resolution.
    pub canonical_url: Option<String>,
    /// Archive.org / cache snapshot if the source URL might rot.
    pub archive_url: Option<String>,
    /// HTTP status from the last verification (e.g. `200`, `404`, `403`).
    pub url_status: Option<i32>,
    /// RFC 3339 UTC timestamp of the last reachability check.
    pub last_verified_at: Option<String>,
    /// Localized title.
    pub title: Option<String>,
    /// Localized short summary (1-3 sentences, vendor-supplied).
    pub summary: Option<String>,
    /// Localized full article markdown (if the agent fetched and cleaned
    /// it).
    pub content_md: Option<String>,
    /// Localized agent-written summary — typically more opinionated than
    /// the vendor `summary`.
    pub agent_summary_md: Option<String>,
    /// Publisher / outlet (`bloomberg`, `reuters`, etc).
    pub source: String,
    /// `news` | `blog` | `regulator` | `social`. Default `news`.
    pub source_kind: String,
    /// `company` (single stock) | `sector` | `macro` | `market`.
    /// Default `company`.
    pub category: String,
    /// Geographic scope — `us` | `hk` | `cn` | `global`. Default `global`.
    pub region: String,
    /// RFC 3339 UTC timestamp the article was published.
    pub published_at: String,
    /// RFC 3339 UTC timestamp the agent fetched the article.
    pub fetched_at: String,
    /// `bullish` / `bearish` / `neutral` toward whatever the article is
    /// about. Mostly meaningful for `category=company`.
    pub sentiment: Option<String>,
    /// Numeric sentiment, usually `[-1, 1]`.
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// `low` | `medium` | `high` | `breaking`. Default `medium`.
    pub importance: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
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

/// `POST /news` body. Always inserts a new row; check via `external_id`
/// before posting if you want to dedup.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsIn {
    /// Vendor's id for the article. Optional but recommended.
    pub external_id: Option<String>,
    /// Article URL.
    pub url: String,
    /// Canonical URL after redirect resolution.
    pub canonical_url: Option<String>,
    /// Archive snapshot URL.
    pub archive_url: Option<String>,
    /// HTTP status from the last reachability check.
    pub url_status: Option<i32>,
    /// RFC 3339 UTC timestamp.
    pub last_verified_at: Option<String>,
    /// Outlet — `bloomberg`, `reuters`, etc.
    pub source: String,
    /// `news` (default) | `blog` | `regulator` | `social`.
    #[serde(default = "default_source_kind")]
    pub source_kind: String,
    /// `company` (default) | `sector` | `macro` | `market`.
    #[serde(default = "default_category")]
    pub category: String,
    /// `global` (default) | `US` | `HK` | `CN`. Case is normalized
    /// server-side so lowercase inputs still match, but the canonical
    /// form is uppercase for the country codes and lowercase for
    /// `global`. Unknown values return 400.
    #[serde(default = "default_region")]
    pub region: String,
    /// RFC 3339 UTC timestamp.
    pub published_at: String,
    /// RFC 3339 UTC timestamp. Defaults to server-side `now()` if omitted.
    pub fetched_at: Option<String>,
    /// Sentiment label.
    pub sentiment: Option<String>,
    /// Numeric sentiment.
    #[schema(value_type = Option<String>)]
    pub sentiment_score: Option<Decimal>,
    /// `low` | `medium` (default) | `high` | `breaking`.
    #[serde(default = "default_importance")]
    pub importance: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "title": "...", "summary": "...",
    /// "content_md": "...", "agent_summary_md": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source_kind() -> String { "news".into() }
fn default_category() -> String { "company".into() }
fn default_region() -> String { "global".into() }
fn default_importance() -> String { "medium".into() }

// ── Link DTOs ────────────────────────────────────────────────────────────

/// A many-to-many edge between a news item and a stock. Created via
/// `POST /news/{id}/stock-links`. Multiple stocks can link to the same
/// article (e.g. M&A coverage).
#[derive(Debug, Serialize, ToSchema)]
pub struct NewsStockLinkOut {
    /// Primary key.
    pub id: i64,
    /// FK to `news_items.id`.
    pub news_id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// `primary` (default) | `secondary` | `peer` | `competitor`.
    pub relation: String,
    /// Optional relevance score in `[0, 1]` — how central this stock is to
    /// the article.
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

/// `POST /news/{news_id}/stock-links` body. The parent `news_id` is in the
/// path. Multiple links per (news_id, stock_id) are allowed — useful if
/// you want to record more than one `relation`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsStockLinkIn {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// `primary` (default) | `secondary` | `peer` | `competitor`.
    #[serde(default = "default_relation")]
    pub relation: String,
    /// Relevance in `[0, 1]`. Optional.
    #[schema(value_type = Option<String>)]
    pub relevance: Option<Decimal>,
}

fn default_relation() -> String { "primary".into() }

/// Sector tag on a news item.
#[derive(Debug, Serialize, ToSchema)]
pub struct NewsSectorLinkOut {
    /// Primary key.
    pub id: i64,
    /// FK to `news_items.id`.
    pub news_id: i64,
    /// FK-ish to `sectors.code`.
    pub sector_code: String,
}

impl From<NewsSectorLink> for NewsSectorLinkOut {
    fn from(l: NewsSectorLink) -> Self {
        Self { id: l.id, news_id: l.news_id, sector_code: l.sector_code }
    }
}

/// `POST /news/{id}/sector-links` body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsSectorLinkIn {
    /// Sector code (e.g. `semiconductors`).
    pub sector_code: String,
}

/// Macro indicator tag on a news item — `news` tagged with `cpi_yoy`,
/// `fed_funds`, etc.
#[derive(Debug, Serialize, ToSchema)]
pub struct NewsMacroLinkOut {
    /// Primary key.
    pub id: i64,
    /// FK to `news_items.id`.
    pub news_id: i64,
    /// FK-ish to `macro_indicators.code`.
    pub indicator_code: String,
}

impl From<NewsMacroLink> for NewsMacroLinkOut {
    fn from(l: NewsMacroLink) -> Self {
        Self { id: l.id, news_id: l.news_id, indicator_code: l.indicator_code }
    }
}

/// `POST /news/{id}/macro-links` body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsMacroLinkIn {
    /// Macro indicator code.
    pub indicator_code: String,
}

/// Country tag on a news item.
#[derive(Debug, Serialize, ToSchema)]
pub struct NewsCountryLinkOut {
    /// Primary key.
    pub id: i64,
    /// FK to `news_items.id`.
    pub news_id: i64,
    /// ISO country code.
    pub country: String,
}

impl From<NewsCountryLink> for NewsCountryLinkOut {
    fn from(l: NewsCountryLink) -> Self {
        Self { id: l.id, news_id: l.news_id, country: l.country }
    }
}

/// `POST /news/{id}/country-links` body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsCountryLinkIn {
    /// ISO country code (`US` / `HK` / `CN`).
    pub country: String,
}
