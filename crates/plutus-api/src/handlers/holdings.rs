use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;
use plutus_core::cost_basis::CostBasisMethod;

use crate::dto::holding::HoldingOut;
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct HoldingsFilter {
    pub account_id: Option<i64>,
    pub method: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<HoldingsFilter>,
) -> ApiResult<Json<Vec<HoldingOut>>> {
    let user_id = require_user(&actor.0)?;
    let method = match f.method.as_deref().unwrap_or("fifo") {
        "fifo" => CostBasisMethod::Fifo,
        "lifo" => CostBasisMethod::Lifo,
        "average" => CostBasisMethod::Average,
        other => {
            return Err(ApiError::BadRequest(format!(
                "method must be fifo/lifo/average; got {other}"
            )))
        }
    };
    let mut rows = if let Some(account_id) = f.account_id {
        plutus_storage::queries::holdings::compute_for_account(&state.db, user_id, account_id, method)
            .await?
    } else {
        plutus_storage::queries::holdings::compute_all(&state.db, user_id, method).await?
    };
    // Sort by stock symbol so the response is stable across refreshes.
    // Holdings aren't a real table — they're computed by aggregating
    // transactions — so we can't ORDER BY in SQL. Doing it here keeps
    // the sort on the backend, which matters for future pagination.
    // For an unknown stock_id (deleted reference data, race condition)
    // we fall back to a string that sorts after real tickers.
    let symbols: HashMap<i64, String> = plutus_storage::queries::stocks::list(
        &state.db,
        "en",
        plutus_storage::queries::stocks::ListFilter {
            symbol: None,
            q: None,
            limit: None,
        },
    )
    .await?
    .into_iter()
    .map(|s| (s.id, s.symbol))
    .collect();
    rows.sort_by(|a, b| {
        let sa = symbols
            .get(&a.stock_id)
            .map(String::as_str)
            .unwrap_or("\u{FFFF}");
        let sb = symbols
            .get(&b.stock_id)
            .map(String::as_str)
            .unwrap_or("\u{FFFF}");
        sa.cmp(sb).then_with(|| a.stock_id.cmp(&b.stock_id))
    });
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
