use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::catalysts::LocalizedCatalyst;

/// A scheduled or potential market-moving event tied to a specific stock,
/// sector, or country. The agent maintains this calendar so other workflows
/// (daily briefings, screener runs) can reason about upcoming events.
///
/// Returned by `GET /catalysts`, `GET /catalysts/{id}`,
/// `GET /stocks/{id}/catalysts`, `POST /catalysts`, `POST /catalysts/batch`.
/// Translatable fields (`title`, `summary_md`, `bull_case_md`, `bear_case_md`,
/// `notes`) are projected from the `content` JSONB using the request's
/// `?locale=` (fallback `en`).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CatalystOut {
    /// Primary key.
    pub id: i64,
    /// Stock this catalyst is anchored to. `null` for sector- or
    /// country-wide events (e.g. an FOMC meeting). Mutually optional with
    /// `sector_code` and `country`.
    pub stock_id: Option<i64>,
    /// Sector code (e.g. `"semiconductors"`). Set for sector-wide catalysts
    /// where no single stock is the focus.
    pub sector_code: Option<String>,
    /// ISO country code (`US` / `HK` / `CN`). Set for country-wide catalysts
    /// like central-bank decisions or holidays.
    pub country: Option<String>,
    /// Event kind — agent-defined string. Common values:
    /// `earnings`, `product_launch`, `regulatory`, `fomc`, `cpi`,
    /// `analyst_day`, `dividend`, `split`, `index_rebalance`, `holiday`.
    pub catalyst_kind: String,
    /// Localized title (from `content.<locale>.title`).
    pub title: Option<String>,
    /// Localized markdown summary (from `content.<locale>.summary_md`).
    pub summary_md: Option<String>,
    /// ISO date string `YYYY-MM-DD`. Stored as text, not a `date` type, so
    /// "TBD this quarter" can be represented as `2026-Q2` etc. if the agent
    /// extends the convention.
    pub catalyst_date: String,
    /// How firm `catalyst_date` is — one of `confirmed`, `scheduled`,
    /// `estimated`, `tentative`. Default: `scheduled`.
    pub date_confidence: String,
    /// Expected market impact — one of `low`, `medium`, `high`. Default:
    /// `medium`. Used by the Daily Briefing to filter the top-N catalysts.
    pub impact_level: String,
    /// Localized bull-case markdown.
    pub bull_case_md: Option<String>,
    /// Localized bear-case markdown.
    pub bear_case_md: Option<String>,
    /// Lifecycle — one of `upcoming`, `released`, `cancelled`. Default:
    /// `upcoming`. The agent flips this after the event occurs and may
    /// refresh the row with the realized outcome in `notes` / `summary_md`.
    pub status: String,
    /// Localized free-form notes (post-event color, links to coverage).
    pub notes: Option<String>,
    /// Canonical URL for the event (announcement, IR page, regulator notice).
    pub url: Option<String>,
    /// Provenance label — `agent`, `manual`, or a specific data vendor.
    /// PART OF THE NATURAL KEY: re-posting the same logical event with a
    /// different `source` creates a separate row.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp. Refreshed on every upsert.
    pub updated_at: String,
    /// RFC 3339 UTC timestamp when this user opened the item's detail
    /// page. `null` while the item is still unread.
    pub read_at: Option<String>,
}

impl From<LocalizedCatalyst> for CatalystOut {
    fn from(c: LocalizedCatalyst) -> Self {
        Self {
            id: c.id,
            stock_id: c.stock_id,
            sector_code: c.sector_code,
            country: c.country,
            catalyst_kind: c.catalyst_kind,
            title: c.title,
            summary_md: c.summary_md,
            catalyst_date: c.catalyst_date,
            date_confidence: c.date_confidence,
            impact_level: c.impact_level,
            bull_case_md: c.bull_case_md,
            bear_case_md: c.bear_case_md,
            status: c.status,
            notes: c.notes,
            url: c.url,
            source: c.source,
            created_at: c.created_at.to_string(),
            updated_at: c.updated_at.to_string(),
            read_at: None,
        }
    }
}

/// `POST /catalysts` body. Upserts against
/// `(user_id, catalyst_kind, catalyst_date, stock_id, sector_code, country, source)`
/// — re-posting the same key refreshes the mutable fields and bumps
/// `updated_at`. `NULL` columns in the key match each other (`NULLS NOT
/// DISTINCT`), so two country-level rows with the same other key parts will
/// collide as intended.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CatalystIn {
    /// Stock this catalyst attaches to. Leave `null` for sector / country
    /// events and set `sector_code` or `country` instead.
    pub stock_id: Option<i64>,
    /// Sector code (e.g. `"semiconductors"`). Use for sector-wide events.
    pub sector_code: Option<String>,
    /// ISO country code (`US` / `HK` / `CN`). Use for country-wide events.
    pub country: Option<String>,
    /// Event kind — see [`CatalystOut::catalyst_kind`] for the typical
    /// values. Free-form string; agree on a vocabulary inside the agent.
    pub catalyst_kind: String,
    /// ISO date `YYYY-MM-DD` (or `YYYY-Qn` for "TBD this quarter" if the
    /// agent agrees on the convention).
    pub catalyst_date: String,
    /// Date confidence: `confirmed` | `scheduled` | `estimated` |
    /// `tentative`. Default `scheduled`.
    #[serde(default = "default_confidence")]
    pub date_confidence: String,
    /// Impact: `low` | `medium` | `high`. Default `medium`.
    #[serde(default = "default_impact")]
    pub impact_level: String,
    /// Status: `upcoming` | `released` | `cancelled`. Default `upcoming`.
    #[serde(default = "default_status")]
    pub status: String,
    /// Canonical link (announcement, IR page, regulator notice).
    pub url: Option<String>,
    /// Provenance — `agent` (default), `manual`, or a vendor name. PART OF
    /// THE NATURAL KEY: distinct sources do NOT collapse into one row.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "title": "...", "summary_md": "...",
    /// "bull_case_md": "...", "bear_case_md": "...", "notes": "..." } }`.
    /// The whole object is stored verbatim; reads project the requested
    /// locale's fields to the top level.
    pub content: serde_json::Value,
}

fn default_confidence() -> String { "scheduled".into() }
fn default_impact() -> String { "medium".into() }
fn default_status() -> String { "upcoming".into() }
fn default_source() -> String { "agent".into() }

/// `POST /catalysts/batch` body. The handler caps `items` at 1000 and
/// rejects empty lists (400). The whole batch runs in one transaction —
/// any failing row rolls the rest back. Each item upserts against the same
/// natural key as the single-item endpoint.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CatalystBatchIn {
    pub items: Vec<CatalystIn>,
}

/// `POST /catalysts/batch` response. `count == items.len()` always — the
/// field is provided for quick caller-side validation against the request.
#[derive(Debug, Serialize, ToSchema)]
pub struct CatalystBatchOut {
    /// Number of rows persisted (== `items.len()`).
    pub count: usize,
    /// Persisted rows, in input order. New rows and refreshed-upsert rows
    /// are indistinguishable in the response — check `created_at` vs
    /// `updated_at` if you need to know.
    pub items: Vec<CatalystOut>,
}
