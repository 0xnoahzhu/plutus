use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::queries::portfolio::{AccountCash, DailyValue, PortfolioSummary};

/// One day's portfolio rollup, returned by `GET /portfolio/value-series`.
/// The series gives the agent + the home-page chart enough to plot a
/// portfolio-equity curve over time without each caller re-deriving it
/// from `/transactions` + `/holdings` + per-stock `/stocks/:id/ohlcv`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DailyValueOut {
    /// ISO date `YYYY-MM-DD`. Calendar day, not trading day — weekends
    /// + holidays carry the previous trading day's close.
    pub date: String,
    /// Sum of `quantity * close_on_that_day` for every open position
    /// on that date. Uses adjusted close when present (split- /
    /// dividend-corrected). Stocks with no recorded price fall back to
    /// the position's cost basis so the series doesn't dip on missing
    /// data.
    #[schema(value_type = String)]
    pub market_value: Decimal,
    /// Sum of FIFO cost basis for every open position on that date.
    #[schema(value_type = String)]
    pub cost_basis: Decimal,
    /// Cash held that day, walked back from each account's anchor
    /// through the ledger. Days before an anchor are computed by backing
    /// the intervening flows out of it.
    #[schema(value_type = String)]
    pub cash: Decimal,
    /// `cash + market_value` — net worth on that date.
    #[schema(value_type = String)]
    pub total_assets: Decimal,
}

impl From<DailyValue> for DailyValueOut {
    fn from(v: DailyValue) -> Self {
        // Round to four decimal places to match the precision policy
        // used in /holdings — display layers don't have to re-format.
        Self {
            date: v.date,
            market_value: v.market_value.round_dp(4),
            cost_basis: v.cost_basis.round_dp(4),
            cash: v.cash.round_dp(4),
            total_assets: v.total_assets.round_dp(4),
        }
    }
}

/// Cash for one account, and how it was arrived at. Exposing the anchor
/// and the flow separately rather than just the total matters: when the
/// number looks wrong, the split says immediately whether the anchor is
/// stale or the ledger is incomplete.
#[derive(Debug, Serialize, ToSchema)]
pub struct AccountCashOut {
    pub account_id: i64,
    pub account_name: String,
    /// ISO-4217 code the figures below are denominated in.
    pub base_currency: String,
    /// The balance the user pinned via `PATCH /accounts/{id}`.
    #[schema(value_type = String)]
    pub anchor: Decimal,
    /// RFC 3339 timestamp the anchor was true, or `null` when there's no
    /// anchor and `flow` therefore covers the entire ledger.
    pub anchor_as_of: Option<String>,
    /// Net cash the ledger moved after `anchor_as_of`.
    #[schema(value_type = String)]
    pub flow: Decimal,
    /// `anchor + flow` — cash actually on hand.
    #[schema(value_type = String)]
    pub cash: Decimal,
}

impl From<AccountCash> for AccountCashOut {
    fn from(a: AccountCash) -> Self {
        Self {
            account_id: a.account_id,
            account_name: a.account_name,
            base_currency: a.base_currency,
            anchor: a.anchor,
            anchor_as_of: a.anchor_as_of.map(|t| t.to_string()),
            flow: a.flow,
            cash: a.cash,
        }
    }
}

/// `GET /portfolio/summary` — net worth as this ledger sees it.
///
/// `total_assets = cash + market_value`. Cash comes from each account's
/// anchor plus the ledger flows after it; market value from the open
/// positions priced at their latest close.
#[derive(Debug, Serialize, ToSchema)]
pub struct PortfolioSummaryOut {
    /// Sum of every account's cash, base currency.
    #[schema(value_type = String)]
    pub cash: Decimal,
    /// Market value of open positions.
    #[schema(value_type = String)]
    pub market_value: Decimal,
    /// FIFO (or requested method) cost basis of those positions.
    #[schema(value_type = String)]
    pub cost_basis: Decimal,
    /// `market_value - cost_basis`.
    #[schema(value_type = String)]
    pub unrealized_pnl: Decimal,
    /// Lifetime realized P&L across every closed leg.
    #[schema(value_type = String)]
    pub realized_pnl: Decimal,
    /// `cash + market_value` — the headline number.
    #[schema(value_type = String)]
    pub total_assets: Decimal,
    /// Cost-basis method used: `fifo` | `lifo` | `average`.
    pub method: String,
    /// Open positions counted.
    pub position_count: i64,
    /// How many of those had no OHLCV bar and were valued at cost basis
    /// instead. Non-zero means `market_value` mixes market and book
    /// values — surface that rather than presenting it as exact.
    pub unpriced_count: i64,
    /// Per-account cash breakdown.
    pub accounts: Vec<AccountCashOut>,
}

impl PortfolioSummaryOut {
    pub fn from_summary(s: PortfolioSummary, method: &str) -> Self {
        Self {
            cash: s.cash,
            market_value: s.market_value,
            cost_basis: s.cost_basis,
            unrealized_pnl: s.unrealized_pnl,
            realized_pnl: s.realized_pnl,
            total_assets: s.total_assets,
            method: method.to_string(),
            position_count: s.position_count,
            unpriced_count: s.unpriced_count,
            accounts: s.accounts.into_iter().map(Into::into).collect(),
        }
    }
}
