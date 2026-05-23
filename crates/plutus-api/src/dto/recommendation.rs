use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::recommendations::LocalizedRecommendation;

/// A buy/sell recommendation the agent issues for a stock or sector. Each
/// row goes through a lifecycle: `open` → close via
/// `POST /recommendations/{id}/close` with status `closed_correct` /
/// `closed_wrong` / `closed_neutral` / `expired` plus the realized
/// `pnl_pct`. This history feeds self-exams.
///
/// `rationale_md` and `outcome_md` are projected from `content.<locale>`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecommendationOut {
    /// Primary key.
    pub id: i64,
    /// FK to `stocks.id`. `null` for sector-level recs (then `sector_code`
    /// is set).
    pub stock_id: Option<i64>,
    /// Sector code for sector-level recs. Mutually optional with `stock_id`.
    pub sector_code: Option<String>,
    /// Direction — `buy` | `sell` | `hold` | `trim` | `add`. Free-form
    /// string; the agent picks a vocabulary.
    pub action: String,
    /// Confidence in `[0, 1]`. Conventional but not enforced.
    #[schema(value_type = Option<String>)]
    pub confidence: Option<Decimal>,
    /// Localized markdown — the thesis. Required-ish in practice (no
    /// rationale = no recommendation).
    pub rationale_md: Option<String>,
    /// Optional price target in `target_currency`.
    #[schema(value_type = Option<String>)]
    pub target_price: Option<Decimal>,
    /// ISO-4217 currency for `target_price`.
    pub target_currency: Option<String>,
    /// Time horizon — `1w` | `1m` | `3m` | `6m` | `12m` | `open`. Default
    /// `open` (no preset expiry).
    pub target_horizon: String,
    /// RFC 3339 UTC timestamp when the rec was issued.
    pub issued_at: String,
    /// Lifecycle — `open` | `closed_correct` | `closed_wrong` |
    /// `closed_neutral` | `expired`. New recs start at `open`; flip via the
    /// `/close` endpoint.
    pub status: String,
    /// Localized post-close markdown — what happened, lessons learned.
    pub outcome_md: Option<String>,
    /// Realized P&L percent at close time (sign matches `action`).
    #[schema(value_type = Option<String>)]
    pub pnl_pct: Option<Decimal>,
    /// RFC 3339 UTC timestamp when the rec was closed. `null` while open.
    pub closed_at: Option<String>,
    /// Provenance — `agent` (default), `manual`, vendor.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
    /// RFC 3339 UTC timestamp when this user opened the item's detail
    /// page. `null` while the item is still unread.
    pub read_at: Option<String>,
}

impl From<LocalizedRecommendation> for RecommendationOut {
    fn from(r: LocalizedRecommendation) -> Self {
        Self {
            id: r.id,
            stock_id: r.stock_id,
            sector_code: r.sector_code,
            action: r.action,
            confidence: r.confidence,
            rationale_md: r.rationale_md,
            target_price: r.target_price,
            target_currency: r.target_currency,
            target_horizon: r.target_horizon,
            issued_at: r.issued_at.to_string(),
            status: r.status,
            outcome_md: r.outcome_md,
            pnl_pct: r.pnl_pct,
            closed_at: r.closed_at.map(|t| t.to_string()),
            source: r.source,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
            read_at: None,
        }
    }
}

/// `POST /recommendations` body. Always creates a new row (initial status
/// `open`); no upsert. To close a rec, use `POST /recommendations/{id}/close`
/// rather than POSTing again.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RecommendationIn {
    /// FK to `stocks.id` for single-stock recs. Leave `null` for
    /// sector-level recs.
    pub stock_id: Option<i64>,
    /// Sector code for sector-level recs.
    pub sector_code: Option<String>,
    /// `buy` | `sell` | `hold` | `trim` | `add`.
    pub action: String,
    /// Confidence in `[0, 1]`. Not validated.
    #[schema(value_type = Option<String>)]
    pub confidence: Option<Decimal>,
    /// Price target.
    #[schema(value_type = Option<String>)]
    pub target_price: Option<Decimal>,
    /// ISO-4217 currency code.
    pub target_currency: Option<String>,
    /// `1w` / `1m` / `3m` / `6m` / `12m` / `open`. Default `open`.
    #[serde(default = "default_horizon")]
    pub target_horizon: String,
    /// RFC 3339 timestamp; defaults to server-side `now()` when omitted.
    pub issued_at: Option<String>,
    /// Default: `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "rationale_md": "...", "outcome_md": "..." } }`.
    /// `outcome_md` is usually empty at creation and filled via the
    /// `/close` endpoint.
    pub content: serde_json::Value,
}

/// `POST /recommendations/{id}/close` body. Closes an `open` rec.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RecommendationClosePatch {
    /// `closed_correct` / `closed_wrong` / `closed_neutral` / `expired`.
    pub status: String,
    /// Optional close note (English at the top level — the localized blob
    /// stays whatever was POSTed initially). Use a PATCH on `content` if you
    /// need multi-locale outcome text.
    pub outcome_md: Option<String>,
    /// Realized P&L percent (e.g. `0.12` for +12%).
    #[schema(value_type = Option<String>)]
    pub pnl_pct: Option<Decimal>,
    /// RFC 3339 timestamp; defaults to server-side `now()` when omitted.
    pub closed_at: Option<String>,
}

fn default_horizon() -> String { "open".into() }
fn default_source() -> String { "agent".into() }
