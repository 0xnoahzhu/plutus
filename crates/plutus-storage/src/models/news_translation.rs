//! Translations of news items. Mirrors the language-specific fields on
//! `news_items` (title / summary / content_md / agent_summary_md). The
//! original text stays on `news_items` itself; this table holds alternatives
//! the agent can write into per locale.

#[derive(Debug, toasty::Model)]
#[table = "news_translations"]
pub struct NewsTranslation {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub news_id: i64,
    pub locale: String, // "en" / "zh-CN" / "zh-TW" / ...
    pub title: String,
    pub summary: Option<String>,
    pub content_md: Option<String>,
    pub agent_summary_md: Option<String>,
    /// "agent" / "human" / "machine_translation" / "external"
    pub translator: String,
    pub updated_at: jiff::Timestamp,
}
