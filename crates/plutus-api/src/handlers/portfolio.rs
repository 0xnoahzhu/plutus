use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use jiff::ToSpan;

use plutus_core::audit::Actor;
use plutus_core::cost_basis::CostBasisMethod;

use crate::dto::portfolio::{DailyValueOut, PortfolioSummaryOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

/// Hard cap on the window width, whichever form asked for it. The
/// series re-folds every lot per day, so an unbounded range is an
/// unbounded fold. Ten years is far past any chart anyone reads and
/// still bounds the work.
const MAX_SPAN_DAYS: i64 = 3653;
/// Cap on the `?days=` shorthand specifically. Kept at the original 365
/// so the existing contract is unchanged — callers wanting more now
/// have `from`/`to`, which says what they mean.
const MAX_DAYS: i64 = 365;
const DEFAULT_DAYS: i64 = 30;

#[derive(Debug, Deserialize)]
pub struct ValueSeriesQuery {
    /// Trailing window ending today. Ignored when `from` is present.
    pub days: Option<i64>,
    /// Inclusive window start, `YYYY-MM-DD`. Enables ranges a trailing
    /// day count can't express — month-to-date, a fixed quarter, the
    /// whole history.
    pub from: Option<String>,
    /// Inclusive window end, `YYYY-MM-DD`. Defaults to today.
    pub to: Option<String>,
}

/// `GET /portfolio/value-series?days=N` or `?from=&to=`
///
/// Returns one row per calendar day in the window. Each row has the
/// user's market value, cost basis, cash and total assets as of that
/// day, computed from `transactions` (FIFO cost basis) and
/// `ohlcv_daily` (latest close on or before the date, carried forward
/// across weekends / holidays). Empty user history → `[]`.
///
/// `from`/`to` wins over `days` when both are sent — it's the more
/// specific request, and silently ignoring the explicit one would be
/// the surprising choice.
pub async fn value_series(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(q): Query<ValueSeriesQuery>,
) -> ApiResult<Json<Vec<DailyValueOut>>> {
    let user_id = require_user(&actor.0)?;
    let today = jiff::Zoned::now().date();

    let end = match q.to.as_deref() {
        Some(s) => parse_date(s, "to")?,
        None => today,
    };
    let start = match q.from.as_deref() {
        Some(s) => parse_date(s, "from")?,
        None => {
            let days = match q.days {
                Some(n) if n <= 0 => {
                    return Err(ApiError::BadRequest("days must be > 0".into()));
                }
                Some(n) if n > MAX_DAYS => {
                    return Err(ApiError::BadRequest(format!(
                        "days must be ≤ {MAX_DAYS}; use from/to for a wider window"
                    )));
                }
                Some(n) => n,
                None => DEFAULT_DAYS,
            };
            end.checked_sub((days - 1).days())
                .map_err(|e| ApiError::BadRequest(format!("days: {e}")))?
        }
    };

    if start > end {
        return Err(ApiError::BadRequest(
            "from must be on or before to".into(),
        ));
    }
    // `days_until` is exclusive of the start, so +1 counts both ends.
    let span = i64::from(start.until(end).map_err(|e| {
        ApiError::BadRequest(format!("date range: {e}"))
    })?.get_days()) + 1;
    if span > MAX_SPAN_DAYS {
        return Err(ApiError::BadRequest(format!(
            "range spans {span} days; maximum is {MAX_SPAN_DAYS}"
        )));
    }

    let rows =
        plutus_storage::queries::portfolio::value_series(&state.db, user_id, start, end).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

fn parse_date(raw: &str, field: &str) -> ApiResult<jiff::civil::Date> {
    raw.parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("{field}: {e}")))
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
