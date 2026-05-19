//! Canonical stock listing. `(market_code, symbol)` is the natural key but we
//! also expose an auto-increment id so URLs and FKs stay short. Uniqueness on
//! the pair is enforced at the application layer for now (toasty 0.6 doesn't
//! support multi-column unique constraints in the derive).
//!
//! Translatable content (`name`, `description_md`) lives in the `content`
//! JSONB column on the DB side. Because toasty 0.6 doesn't speak JSONB, the
//! model omits that column entirely — raw `tokio_postgres` SQL in
//! `queries::stocks` handles read/write of localized content. The fields
//! declared here are the metadata columns toasty can manage (filtering,
//! indexing).

#[derive(Debug, toasty::Model)]
#[table = "stocks"]
pub struct Stock {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub market_code: String,
    #[index]
    pub symbol: String,
    pub isin: Option<String>,
    pub figi: Option<String>,
    pub currency: String,
    pub lot_size: Option<i32>,
    pub asset_class: String, // TransactionKind-style enum stored as text
    pub sector_code: Option<String>, // FK into sectors.code; enforced at app layer
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
