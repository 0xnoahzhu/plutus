//! News / blog / research note / social post entries.
//!
//! `external_id` and `url` are both unique (when present) so the same article
//! pulled from two sources isn't stored twice.
//!
//! Translatable content (`title`, `summary`, `content_md`, `agent_summary_md`)
//! lives in the `content` JSONB column on the DB side. Because toasty 0.6
//! doesn't speak JSONB, the model omits that column entirely — raw
//! `tokio_postgres` SQL in `queries::news` handles read/write of localized
//! content. The fields declared here are the metadata columns toasty can
//! manage (filtering, indexing).

#[derive(Debug, toasty::Model)]
#[table = "news_items"]
pub struct NewsItem {
    #[key]
    #[auto]
    pub id: i64,
    pub external_id: Option<String>,
    #[unique]
    pub url: String,
    pub canonical_url: Option<String>,
    pub archive_url: Option<String>,
    pub url_status: Option<i32>, // last observed HTTP code
    pub last_verified_at: Option<jiff::Timestamp>,
    pub source: String,   // "Reuters" / "Bloomberg" / "财新"
    pub source_kind: String, // "news" / "filing" / "research_note" / "blog" / "social"
    pub category: String, // "company" / "macro" / "regulatory" / "industry" / "earnings" / "ma"
    pub region: String,   // "US" / "HK" / "CN" / "global"
    #[index]
    pub published_at: jiff::Timestamp,
    pub fetched_at: jiff::Timestamp,
    pub sentiment: Option<String>, // "positive" / "negative" / "neutral"
    pub sentiment_score: Option<rust_decimal::Decimal>, // -1.0 .. 1.0
    pub importance: String, // "high" / "medium" / "low"
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
