use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{CorrelationPair, UniverseDefinition};
use plutus_storage::queries::correlations::LocalizedCorrelationRun;

/// A named set of stocks the user (or agent) wants to study together —
/// "S&P 500 megacaps", "my HK watch", "EV supply chain". Used as the input
/// for `correlation_runs`. Unique by `(user_id, name)`; POSTing the same
/// name updates the existing row.
#[derive(Debug, Serialize, ToSchema)]
pub struct UniverseOut {
    /// Primary key.
    pub id: i64,
    /// Display name. Unique within a user's universe collection.
    pub name: String,
    /// Optional markdown explaining what this universe represents and why
    /// these stocks belong together.
    pub description_md: Option<String>,
    /// JSON-stringified array of `stocks.id` values (e.g. `"[1,2,3]"`).
    /// Stored as text; parse client-side. Order is meaningful only if the
    /// agent treats it so — the server doesn't enforce uniqueness within.
    pub stock_ids: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

impl From<UniverseDefinition> for UniverseOut {
    fn from(u: UniverseDefinition) -> Self {
        Self {
            id: u.id, name: u.name, description_md: u.description_md,
            stock_ids: u.stock_ids,
            created_at: u.created_at.to_string(), updated_at: u.updated_at.to_string(),
        }
    }
}

/// `POST /universes` body. Upserts by `(user_id, name)`. Re-posting the
/// same name replaces `stock_ids` and `description_md` entirely (no merge).
#[derive(Debug, Deserialize, ToSchema)]
pub struct UniverseIn {
    /// Universe name. Treated as a natural-key string — choose carefully.
    pub name: String,
    /// Optional markdown description.
    pub description_md: Option<String>,
    /// Array of `stocks.id` values. The server JSON-stringifies for
    /// storage; clients always send a real array.
    pub stock_ids: Vec<i64>,
}

/// A pairwise-correlation analysis over a universe. The agent picks a
/// `method` (`pearson` default), a `lookback_days` window, then computes
/// pair correlations and POSTs them to `/correlation-runs/{id}/pairs`.
///
/// `summary_md` is projected from `content.<locale>.summary_md`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CorrelationRunOut {
    /// Primary key.
    pub id: i64,
    /// Run flavor — agent-defined. Examples: `weekly`, `event`, `regime`.
    pub kind: String,
    /// ISO date `YYYY-MM-DD`.
    pub run_date: String,
    /// FK to `universe_definitions.id`. The set of stocks the pairs cover.
    pub universe_id: i64,
    /// Trailing window the correlations were computed over.
    pub lookback_days: i32,
    /// Method — `pearson` (default) | `spearman` | `kendall` | anything the
    /// agent supports.
    pub method: String,
    /// Localized markdown read-out — clusters, surprises, regime call.
    pub summary_md: Option<String>,
    /// JSON-stringified per-run metrics (e.g.
    /// `{"avg_corr":0.34,"top_cluster":"semis"}`).
    pub metrics: Option<String>,
    /// Provenance — `agent` (default).
    pub source: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
    /// RFC 3339 UTC timestamp when this user opened the item's detail
    /// page. `null` while the item is still unread.
    pub read_at: Option<String>,
}

impl From<LocalizedCorrelationRun> for CorrelationRunOut {
    fn from(r: LocalizedCorrelationRun) -> Self {
        Self {
            id: r.id,
            kind: r.kind,
            run_date: r.run_date,
            universe_id: r.universe_id,
            lookback_days: r.lookback_days,
            method: r.method,
            summary_md: r.summary_md,
            metrics: r.metrics,
            source: r.source,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
            read_at: None,
        }
    }
}

/// `POST /correlation-runs` body. Creates a new run row; no upsert (delete
/// the old run via `DELETE /correlation-runs/{id}` if you need to re-run).
#[derive(Debug, Deserialize, ToSchema)]
pub struct CorrelationRunIn {
    /// Run flavor — see [`CorrelationRunOut::kind`].
    pub kind: String,
    /// ISO date `YYYY-MM-DD`.
    pub run_date: String,
    /// FK to `universe_definitions.id`.
    pub universe_id: i64,
    /// Trailing window in days.
    pub lookback_days: i32,
    /// Default `pearson`.
    #[serde(default = "default_method")]
    pub method: String,
    /// Run-level metrics as a JSON object (stringified server-side).
    pub metrics: Option<serde_json::Value>,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "summary_md": "..." } }`.
    pub content: serde_json::Value,
}

/// One pair-correlation row inside a correlation run. The pairs are
/// stored canonically — the server orders `(stock_a_id, stock_b_id)` so
/// `a <= b` to avoid double-counting `(AAPL, MSFT)` and `(MSFT, AAPL)`.
#[derive(Debug, Serialize, ToSchema)]
pub struct CorrelationPairOut {
    /// Primary key.
    pub id: i64,
    /// FK to `correlation_runs.id`.
    pub run_id: i64,
    /// Lower-id member of the pair (canonical order).
    pub stock_a_id: i64,
    /// Higher-id member of the pair (canonical order).
    pub stock_b_id: i64,
    /// Pairwise correlation, typically in `[-1, 1]`. Decimal so trailing
    /// precision survives the round-trip.
    #[schema(value_type = String)]
    pub correlation: Decimal,
}

impl From<CorrelationPair> for CorrelationPairOut {
    fn from(p: CorrelationPair) -> Self {
        Self {
            id: p.id, run_id: p.run_id,
            stock_a_id: p.stock_a_id, stock_b_id: p.stock_b_id,
            correlation: p.correlation,
        }
    }
}

/// `POST /correlation-runs/{id}/pairs` body. The server canonicalizes the
/// stock ordering so `(stock_a_id, stock_b_id)` always has `a <= b` —
/// callers don't need to pre-sort.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CorrelationPairIn {
    /// FK to `stocks.id`. Order doesn't matter; server canonicalizes.
    pub stock_a_id: i64,
    /// FK to `stocks.id`.
    pub stock_b_id: i64,
    /// Pairwise correlation, typically `[-1, 1]`.
    #[schema(value_type = String)]
    pub correlation: Decimal,
}

fn default_method() -> String { "pearson".into() }
fn default_source() -> String { "agent".into() }
