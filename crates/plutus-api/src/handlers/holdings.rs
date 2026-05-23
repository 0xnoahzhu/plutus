use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use plutus_core::audit::Actor;
use plutus_core::cost_basis::CostBasisMethod;

use crate::dto::holding::{HoldingOut, HoldingStockMeta};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

const DEFAULT_PER_PAGE: i64 = 15;
const MAX_PER_PAGE: i64 = 500;

#[derive(Deserialize)]
pub struct HoldingsFilter {
    pub account_id: Option<i64>,
    pub method: Option<String>,
    /// ISO country (US/HK/CN). Filters by joined market_code.
    pub country: Option<String>,
    /// Case-insensitive substring match on stock symbol.
    pub q: Option<String>,
    /// 1-indexed page. When set, response carries X-Total-Count.
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<HoldingsFilter>,
) -> ApiResult<axum::response::Response> {
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
            sector_code: None,
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
    // Filter by query (symbol substring) and country before paginating
    // so page boundaries respect both. Both checks are case-insensitive
    // and skip rows whose stock metadata couldn't be resolved.
    let q_upper = f
        .q
        .as_deref()
        .map(|s| s.trim().to_ascii_uppercase())
        .filter(|s| !s.is_empty());
    let market_codes: Option<std::collections::HashSet<String>> =
        if let Some(country) = f.country.as_deref() {
            Some(
                plutus_storage::queries::markets::list_codes_by_country(&state.db, country)
                    .await?
                    .into_iter()
                    .collect(),
            )
        } else {
            None
        };
    let filtered: Vec<_> = rows
        .into_iter()
        .filter(|h| {
            let Some(meta) = stock_meta.get(&h.stock_id) else {
                return false;
            };
            if let Some(ref q) = q_upper {
                if !meta.symbol.to_ascii_uppercase().contains(q) {
                    return false;
                }
            }
            if let Some(ref codes) = market_codes {
                if !codes.contains(&meta.market_code) {
                    return false;
                }
            }
            true
        })
        .collect();

    // One OHLCV query for every held stock so we can fill in market
    // value and unrealized P&L. Stocks with no bar yet get None, which
    // serializes to null and surfaces as `—` in the UI.
    let latest_closes =
        plutus_storage::queries::ohlcv::latest_closes(&state.db, &stock_ids).await?;

    let total = filtered.len() as i64;
    let paginating = f.page.is_some();
    let page_slice: Vec<_> = if paginating {
        let per_page = f
            .per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE);
        let page = f.page.unwrap_or(1).max(1);
        let offset = ((page - 1) * per_page) as usize;
        filtered
            .into_iter()
            .skip(offset)
            .take(per_page as usize)
            .collect()
    } else {
        filtered
    };

    let out: Vec<HoldingOut> = page_slice
        .into_iter()
        .map(|h| {
            let close = latest_closes.get(&h.stock_id).copied();
            let meta = stock_meta.get(&h.stock_id).cloned();
            HoldingOut::from_holding(h, close, meta)
        })
        .collect();
    let mut headers = HeaderMap::new();
    if paginating {
        if let Ok(v) = HeaderValue::from_str(&total.to_string()) {
            headers.insert("X-Total-Count", v);
        }
    }
    Ok((headers, Json(out)).into_response())
}
