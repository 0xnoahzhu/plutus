//! Named set of stocks used as the basis for analytics like correlation matrices.
//! Stock membership stored as a JSON array of stock_id; for the scale we work at
//! that's plenty. Uniqueness on `(user_id, name)` is enforced by a post-migrate
//! index — the previous single-tenant `#[unique]` on `name` alone was retired.

#[derive(Debug, toasty::Model)]
#[table = "universe_definitions"]
pub struct UniverseDefinition {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    pub name: String, // "AI_CHIPS_2026Q2" / "HOLDINGS_2026-05" / etc.
    pub description_md: Option<String>,
    pub stock_ids: String, // JSON array of stock ids
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
