use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;
use plutus_core::cost_basis::CostBasisMethod;

use crate::dto::portfolio::{DailyValueOut, PortfolioSummaryOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

/// Hard cap on `?days=` so a misconfigured agent can't ask for the
/// whole history and bog the server down. 365 covers a year of trading
/// days — plenty for the home-page chart.
const MAX_DAYS: i64 = 365;
const DEFAULT_DAYS: i64 = 30;

#[derive(Debug, Deserialize)]
pub struct ValueSeriesQuery {
    pub days: Option<i64>,
}

/// `GET /portfolio/value-series?days=N`
///
/// Returns one row per calendar day in the lookback window. Each row
/// has the user's total market value and cost basis as of that day,
/// computed from `transactions` (FIFO cost basis) and `ohlcv_daily`
/// (latest close on or before the date, carried forward across
/// weekends / holidays). Empty user history → `[]`.
pub async fn value_series(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(q): Query<ValueSeriesQuery>,
) -> ApiResult<Json<Vec<DailyValueOut>>> {
    let user_id = require_user(&actor.0)?;
    let days = match q.days {
        Some(n) if n <= 0 => {
            return Err(ApiError::BadRequest("days must be > 0".into()));
        }
        Some(n) if n > MAX_DAYS => {
            return Err(ApiError::BadRequest(format!("days must be ≤ {MAX_DAYS}")));
        }
        Some(n) => n,
        None => DEFAULT_DAYS,
    };
    let rows = plutus_storage::queries::portfolio::value_series(&state.db, user_id, days).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

#[derive(Debug, Deserialize)]
pub struct SummaryQuery {
    /// `fifo` (default) | `lifo` | `average`. Affects cost basis and
    /// realized P&L; cash and market value are method-independent.
    pub method: Option<String>,
}

/// `GET /portfolio/summary`
///
/// Net worth: `total_assets = cash + market_value`.
///
/// Cash is each account's anchor (`accounts.cash_balance` as of
/// `cash_as_of`) plus every ledger cash flow after it — buys and fees
/// out, sells, deposits, dividends and interest in. Market value is the
/// open positions at their latest close, falling back to cost basis for
/// any stock with no OHLCV bar (`unpriced_count` says how many, so the
/// caller can flag the total as an estimate).
pub async fn summary(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(q): Query<SummaryQuery>,
) -> ApiResult<Json<PortfolioSummaryOut>> {
    let user_id = require_user(&actor.0)?;
    let method_str = q.method.as_deref().unwrap_or("fifo");
    let method = match method_str {
        "fifo" => CostBasisMethod::Fifo,
        "lifo" => CostBasisMethod::Lifo,
        "average" => CostBasisMethod::Average,
        other => {
            return Err(ApiError::BadRequest(format!(
                "method must be fifo/lifo/average; got {other}"
            )))
        }
    };
    let s = plutus_storage::queries::portfolio::summary(&state.db, user_id, method).await?;
    Ok(Json(PortfolioSummaryOut::from_summary(s, method_str)))
}
