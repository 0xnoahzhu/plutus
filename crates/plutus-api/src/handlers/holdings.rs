use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;
use plutus_core::cost_basis::CostBasisMethod;

use crate::dto::holding::{HoldingOut, HoldingStockMeta};
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
    // Fetch ONLY the held stocks (not the full catalog). With a 5k+
    // catalog the old "fetch all then build a symbol map" pattern
    // was O(N catalog) instead of O(N positions). The id-filter
    // SQL bypasses the LIMIT cap on stocks::list automatically.
    let stock_ids: Vec<i64> = rows.iter().map(|h| h.stock_id).collect();
    let stock_meta: HashMap<i64, HoldingStockMeta> = plutus_storage::queries::stocks::list(
        &state.db,
        "en",
        plutus_storage::queries::stocks::ListFilter {
            symbol: None,
            q: None,
            ids: Some(&stock_ids),
            limit: None,
            offset: None,
        },
    )
    .await?
    .into_iter()
    .map(|s| {
        (
            s.id,
            HoldingStockMeta {
                symbol: s.symbol,
                market_code: s.market_code,
                currency: s.currency,
            },
        )
    })
    .collect();
    // Stable sort by symbol. Holdings aren't a real table (computed
    // from transactions), so the sort happens here, not in SQL. An
    // unknown stock_id falls back to a string that sorts after real
    // tickers so orphans cluster predictably at the end.
    rows.sort_by(|a, b| {
        let sa = stock_meta
            .get(&a.stock_id)
            .map(|m| m.symbol.as_str())
            .unwrap_or("\u{FFFF}");
        let sb = stock_meta
            .get(&b.stock_id)
            .map(|m| m.symbol.as_str())
            .unwrap_or("\u{FFFF}");
        sa.cmp(sb).then_with(|| a.stock_id.cmp(&b.stock_id))
    });
    // One OHLCV query for every held stock so we can fill in market
    // value and unrealized P&L. Stocks with no bar yet get None, which
    // serializes to null and surfaces as `—` in the UI.
    let latest_closes =
        plutus_storage::queries::ohlcv::latest_closes(&state.db, &stock_ids).await?;
    let out: Vec<HoldingOut> = rows
        .into_iter()
        .map(|h| {
            let close = latest_closes.get(&h.stock_id).copied();
            let meta = stock_meta.get(&h.stock_id).cloned();
            HoldingOut::from_holding(h, close, meta)
        })
        .collect();
    Ok(Json(out))
}
