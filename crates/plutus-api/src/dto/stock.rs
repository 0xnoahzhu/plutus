use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::stocks::LocalizedStock;

/// A tradable instrument — equity, ETF, fund. The reference table; every
/// per-stock entity (`catalysts`, `transactions`, `watchlist_items`,
/// `recommendations`, `screener_hits`, `correlation_pairs`, etc.) joins on
/// `stocks.id`.
///
/// **Lookup patterns** (`GET /stocks?...`):
/// - `?symbol=AAPL` — exact ticker, case-insensitive. Returns 0 or 1 row.
/// - `?q=Apple` — fuzzy substring over symbol + localized name. Up to
///   `limit` rows (default 50, max 200).
/// - `?country=US` — country filter via the market_code → MIC mapping.
/// - No filters — first `limit` rows, ordered by id.
///
/// Translatable fields (`name`, `description_md`) come from
/// `content.<locale>`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StockOut {
    /// Primary key — referenced by `stock_id` on most other tables.
    pub id: i64,
    /// MIC-flavored market code, lower-snake (`us`, `us_etf`, `hk`, `cn_a`,
    /// `cn_etf`). Maps to a country via the `markets` table.
    pub market_code: String,
    /// Ticker. Case as listed on the exchange (e.g. `AAPL`, `0700`).
    pub symbol: String,
    /// ISIN if known. Useful for cross-listing matches.
    pub isin: Option<String>,
    /// Bloomberg/OpenFIGI identifier if known.
    pub figi: Option<String>,
    /// ISO-4217 trading currency (`USD`, `HKD`, `CNY`).
    pub currency: String,
    /// Minimum lot size for HK / CN markets. `null` for US (single share).
    pub lot_size: Option<i32>,
    /// `common_stock` | `etf` | `mutual_fund` | `adr` | `gdr` | `index`.
    pub asset_class: String,
    /// Optional FK-ish sector code (matches `sectors.code` when present).
    pub sector_code: Option<String>,
    /// Localized company name (from `content.<locale>.name`).
    pub name: Option<String>,
    /// Localized markdown company description.
    pub description_md: Option<String>,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

impl From<LocalizedStock> for StockOut {
    fn from(s: LocalizedStock) -> Self {
        Self {
            id: s.id,
            market_code: s.market_code,
            symbol: s.symbol,
            isin: s.isin,
            figi: s.figi,
            currency: s.currency,
            lot_size: s.lot_size,
            asset_class: s.asset_class,
            sector_code: s.sector_code,
            name: s.name,
            description_md: s.description_md,
            created_at: s.created_at.to_string(),
            updated_at: s.updated_at.to_string(),
        }
    }
}

/// `POST /stocks` body. Inserts a new row; no upsert — to update a stock
/// use `PATCH /stocks/{id}` for `content` or fix the row directly in psql
/// for the immutable columns.
#[derive(Debug, Deserialize, ToSchema)]
pub struct StockIn {
    /// MIC-flavored market code (`us`, `hk`, etc).
    pub market_code: String,
    /// Ticker as listed.
    pub symbol: String,
    /// ISIN.
    pub isin: Option<String>,
    /// OpenFIGI id.
    pub figi: Option<String>,
    /// ISO-4217 trading currency.
    pub currency: String,
    /// HK/CN lot size.
    pub lot_size: Option<i32>,
    /// `common_stock` | `etf` | `mutual_fund` | `adr` | `gdr` | `index`.
    pub asset_class: String,
    /// Optional FK-ish sector code.
    pub sector_code: Option<String>,
    /// Multi-locale content blob:
    /// `{ "<locale>": { "name": "...", "description_md": "..." } }`. The
    /// server stores it verbatim; merge before sending if you want partial
    /// updates.
    pub content: serde_json::Value,
}

/// `PATCH /stocks/{id}` body. Only `content` is mutable through this
/// route — symbol/ISIN/etc are treated as immutable.
#[derive(Debug, Deserialize, ToSchema)]
pub struct StockPatch {
    /// New full multi-locale blob, replacing whatever is currently stored
    /// (no merge). The endpoint returns 400 if `content` is missing.
    pub content: Option<serde_json::Value>,
}
