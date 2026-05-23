use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::screeners::{LocalizedScreenerHit, LocalizedScreenerRun};

/// One execution of a stock-screener pipeline. The agent owns the schedule
/// (e.g. "weekly value screen", "daily insider-buying check") and posts a
/// row per (name, kind, run_date). Each run has its own `hits` collection
/// addressed via `/screener-runs/{id}/hits`.
///
/// `description_md` and `summary_md` come from `content.<locale>` —
/// `description_md` is meant to capture the screener's intent (criteria
/// rationale), `summary_md` the post-run reading of the results.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScreenerRunOut {
    /// Primary key.
    pub id: i64,
    /// Screener identifier — slug-like, stable across runs. Forms part of
    /// the natural key: re-running the same `(name, kind, run_date)` upserts
    /// the row instead of duplicating it.
    pub name: String,
    /// Screener category — agent-defined. Examples: `value`, `momentum`,
    /// `insider`, `analyst_upgrade`, `event_driven`. Used for filtering and
    /// (together with `name` + `run_date`) forms the natural key.
    pub kind: String,
    /// ISO date `YYYY-MM-DD` of the run. Combined with `name` + `kind` to
    /// dedup re-runs.
    pub run_date: String,
    /// Universe scanned — free-form label (e.g. `sp500`, `us_smallcap`,
    /// `hk_main`). Not a FK to `universe_definitions`; that's a separate
    /// model used by `correlation_runs`.
    pub universe: String,
    /// Number of candidates the screener considered before filtering.
    pub universe_size: Option<i32>,
    /// JSON-stringified criteria (e.g. `{"pe<": 15, "fcf_yield>": 0.05}`).
    /// Stored as text to keep the criteria DSL agent-private.
    pub criteria: Option<String>,
    /// Localized markdown explaining what this screener looks for and why.
    pub description_md: Option<String>,
    /// Localized markdown reading of the results — top picks, themes,
    /// notable absences.
    pub summary_md: Option<String>,
    /// Overall flavor — `bullish` / `bearish` / `neutral`. Optional; the
    /// summary_md is the canonical narrative.
    pub sentiment: Option<String>,
    /// Provenance — `agent` (default), `manual`, or a vendor.
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp. Refreshed on every upsert.
    pub updated_at: String,
    /// RFC 3339 UTC timestamp when this user opened the item's detail
    /// page. `null` while the item is still unread.
    pub read_at: Option<String>,
}

impl From<LocalizedScreenerRun> for ScreenerRunOut {
    fn from(r: LocalizedScreenerRun) -> Self {
        Self {
            id: r.id,
            name: r.name,
            kind: r.kind,
            run_date: r.run_date,
            universe: r.universe,
            universe_size: r.universe_size,
            criteria: r.criteria,
            description_md: r.description_md,
            summary_md: r.summary_md,
            sentiment: r.sentiment,
            source: r.source,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
            read_at: None,
        }
    }
}

/// `POST /screener-runs` body. Upserts against `(user_id, name, kind,
/// run_date)`. To re-screen with different parameters under the same name,
/// either change `run_date` (preserves history) or delete the old row first.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ScreenerRunIn {
    /// Screener identifier — slug-like, stable across runs.
    pub name: String,
    /// Screener category — see [`ScreenerRunOut::kind`] for common values.
    pub kind: String,
    /// ISO date `YYYY-MM-DD` of the run.
    pub run_date: String,
    /// Universe label (e.g. `sp500`, `us_smallcap`).
    pub universe: String,
    /// Candidates considered before filtering.
    pub universe_size: Option<i32>,
    /// Criteria as a JSON object; the server stringifies it for storage.
    pub criteria: Option<serde_json::Value>,
    /// Sentiment hint — `bullish` / `bearish` / `neutral`.
    pub sentiment: Option<String>,
    /// Provenance. Default: `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "description_md": "...", "summary_md": "..." } }`.
    pub content: serde_json::Value,
}

/// One stock surfaced by a screener run. Belongs to a parent
/// `screener_run`; deleting the run cascades the hits via
/// `DELETE /screener-runs/{id}`.
///
/// `rationale_md` is projected from `content.<locale>.rationale_md`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScreenerHitOut {
    /// Primary key.
    pub id: i64,
    /// FK to `screener_runs.id`. Scoped to the same user as the parent run.
    pub run_id: i64,
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// 1-based rank within this run. Optional — some screeners are
    /// unordered ("everything that passes a hard filter").
    pub rank: Option<i32>,
    /// Composite score from the screener's ranking function.
    #[schema(value_type = Option<String>)]
    pub score: Option<Decimal>,
    /// Localized markdown explaining why this stock surfaced.
    pub rationale_md: Option<String>,
    /// JSON-stringified per-metric values (e.g. `{"pe":12.3,"roe":0.18}`).
    pub metrics: Option<String>,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
}

impl From<LocalizedScreenerHit> for ScreenerHitOut {
    fn from(h: LocalizedScreenerHit) -> Self {
        Self {
            id: h.id,
            run_id: h.run_id,
            stock_id: h.stock_id,
            rank: h.rank,
            score: h.score,
            rationale_md: h.rationale_md,
            metrics: h.metrics,
            created_at: h.created_at.to_string(),
        }
    }
}

/// `POST /screener-runs/{id}/hits` body. The parent `run_id` comes from the
/// path; this struct intentionally omits it. No upsert here — each POST
/// inserts a new row.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ScreenerHitIn {
    /// FK to `stocks.id`. The handler does NOT verify the stock exists
    /// before insert; the row is orphaned (but harmless) if the id is wrong.
    pub stock_id: i64,
    /// Rank within the run. Optional.
    pub rank: Option<i32>,
    /// Composite ranking score.
    #[schema(value_type = Option<String>)]
    pub score: Option<Decimal>,
    /// Per-metric values as a JSON object; the server stringifies.
    pub metrics: Option<serde_json::Value>,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "rationale_md": "..." } }`.
    pub content: serde_json::Value,
}

fn default_source() -> String { "agent".into() }
