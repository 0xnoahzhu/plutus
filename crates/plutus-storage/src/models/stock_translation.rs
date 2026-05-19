//! i18n metadata for stocks. Natural composite key is (stock_id, locale); we
//! synthesize a surrogate id for portability across toasty's PK conventions.

#[derive(Debug, toasty::Model)]
#[table = "stock_translations"]
pub struct StockTranslation {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub stock_id: i64,
    pub locale: String, // "en", "zh-CN", "zh-TW"
    pub name: String,
    pub description_md: Option<String>,
    pub updated_at: jiff::Timestamp,
}
