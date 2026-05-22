use rust_decimal::Decimal;
use serde::Serialize;
use utoipa::ToSchema;

use plutus_storage::queries::holdings::Holding;

/// A derived open position, rolled up from `transactions`. There's no
/// `holdings` table — the storage layer computes positions on every read
/// using weighted-average cost basis.
///
/// `GET /holdings` returns one row per `(stock_id, account_id)` for the
/// caller's accounts. Closed positions (quantity == 0) are omitted.
#[derive(Debug, Serialize, ToSchema)]
pub struct HoldingOut {
    /// FK to `stocks.id`.
    pub stock_id: i64,
    /// FK to `accounts.id`. `null` for cross-account rollups (a future
    /// query mode not currently exposed).
    pub account_id: Option<i64>,
    /// Net open shares (positive = long).
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// Weighted-average cost per share in the trade currency.
    #[schema(value_type = String)]
    pub avg_cost_trade: Decimal,
    /// Total cost basis converted to the account's `base_currency` using
    /// each transaction's `fx_rate_to_base`.
    #[schema(value_type = String)]
    pub cost_base: Decimal,
    /// Realized P&L from closed legs, in base currency. Lifetime
    /// (resets only on full close). 0 means no sells have happened
    /// against this position yet (commissions make true zero break-even
    /// vanishingly rare); the UI renders 0 as `—`.
    #[schema(value_type = String)]
    pub realized_pnl_base: Decimal,
    /// Current market value of the open position in base currency,
    /// computed as `quantity * latest_close`. `null` when no OHLCV bar
    /// has ever been recorded for the stock (the UI then renders `—`
    /// rather than 0). FX is treated as 1 — base currency tracks the
    /// trade currency for the current data set; cross-currency
    /// precision can come later if positions diversify.
    #[schema(value_type = Option<String>)]
    pub market_value_base: Option<Decimal>,
    /// Unrealized P&L = `market_value_base - cost_base`. `null` when
    /// `market_value_base` is null. Sign tracks gain/loss.
    #[schema(value_type = Option<String>)]
    pub unrealized_pnl_base: Option<Decimal>,

    /// Stock ticker. Inlined from `stocks` so the UI doesn't need a
    /// second round trip just to resolve symbols. `null` if the stock
    /// row was deleted (rare; reference data is shared, not user data).
    pub symbol: Option<String>,
    /// Market code (MIC). Inlined from `stocks`.
    pub market_code: Option<String>,
    /// Trade currency. Inlined from `stocks`.
    pub currency: Option<String>,
}

/// Subset of stock metadata the holdings handler joins onto each row.
/// Owned strings so the closure doesn't borrow from the outer map.
#[derive(Debug, Clone)]
pub struct HoldingStockMeta {
    pub symbol: String,
    pub market_code: String,
    pub currency: String,
}

impl HoldingOut {
    /// Build a row, layering on the latest-close-derived fields and
    /// the inlined stock metadata. Pass `latest_close = None` when no
    /// OHLCV bar is on file (market_value / unrealized then surface
    /// as `null`); pass `stock = None` when the stock row is gone
    /// (symbol / market / currency surface as `null`).
    pub fn from_holding(
        h: Holding,
        latest_close: Option<Decimal>,
        stock: Option<HoldingStockMeta>,
    ) -> Self {
        let market_value_base = latest_close.map(|c| h.position.quantity * c);
        let unrealized_pnl_base = market_value_base.map(|mv| mv - h.position.cost_base);
        Self {
            stock_id: h.stock_id,
            account_id: h.account_id,
            quantity: h.position.quantity,
            avg_cost_trade: h.position.avg_cost_trade,
            cost_base: h.position.cost_base,
            realized_pnl_base: h.position.realized_pnl_base,
            market_value_base,
            unrealized_pnl_base,
            symbol: stock.as_ref().map(|s| s.symbol.clone()),
            market_code: stock.as_ref().map(|s| s.market_code.clone()),
            currency: stock.map(|s| s.currency),
        }
    }
}
