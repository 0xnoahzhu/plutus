//! Named set of stocks used as the basis for analytics like correlation matrices.
//! Stock membership stored as a JSON array of stock_id; for the scale we work at
//! that's plenty.

#[derive(Debug, toasty::Model)]
#[table = "universe_definitions"]
pub struct UniverseDefinition {
    #[key]
    #[auto]
    pub id: i64,
    #[unique]
    pub name: String, // "AI_CHIPS_2026Q2" / "HOLDINGS_2026-05" / etc.
    pub description_md: Option<String>,
    pub stock_ids: String, // JSON array of stock ids
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
