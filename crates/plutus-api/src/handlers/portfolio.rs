use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;

use crate::dto::portfolio::DailyValueOut;
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
