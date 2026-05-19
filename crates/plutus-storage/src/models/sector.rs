//! Industry / sector classification. Hierarchy via `parent_code`; top-level
//! entries have `parent_code = None`. `scheme` lets us mix GICS with custom or
//! exchange-specific taxonomies later (default seed is GICS).

#[derive(Debug, toasty::Model)]
#[table = "sectors"]
pub struct Sector {
    #[key]
    pub code: String, // e.g. "45" (GICS Information Technology) or "45.10" (Semis)
    pub name: String,
    pub parent_code: Option<String>,
    pub scheme: String, // "GICS" / "SSE" / "custom"
}
