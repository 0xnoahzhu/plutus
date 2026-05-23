use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;

use std::str::FromStr;

use plutus_core::audit::Actor;
use plutus_core::transaction::TransactionKind;

use crate::dto::transaction::{TransactionIn, TransactionOut};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::state::AppState;

const DEFAULT_PER_PAGE: i64 = 15;
const MAX_PER_PAGE: i64 = 500;

#[derive(Deserialize)]
pub struct ListFilter {
    pub account_id: Option<i64>,
    pub stock_id: Option<i64>,
    /// ISO country (US/HK/CN). Filters by the joined stock's market.
    pub country: Option<String>,
    /// Case-insensitive substring match on the joined stock symbol.
    pub q: Option<String>,
    /// 1-indexed page. When set, response carries X-Total-Count.
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
) -> ApiResult<axum::response::Response> {
    let user_id = require_user(&actor.0)?;
    let rows = if let Some(account_id) = f.account_id {
        plutus_storage::queries::transactions::list_for_account(&state.db, user_id, account_id)
            .await?
    } else if let Some(stock_id) = f.stock_id {
        plutus_storage::queries::transactions::list_for_stock(&state.db, user_id, stock_id).await?
    } else {
        plutus_storage::queries::transactions::list(&state.db, user_id).await?
    };

    // Resolve symbol + market_code per touched stock_id for the q
    // and country filters. Cash-movement transactions (stock_id=null)
    // pass through the q filter only when q is unset.
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
    let stock_ids: Vec<i64> = rows.iter().filter_map(|r| r.stock_id).collect();
    let meta_map: HashMap<i64, (String, String)> = if !stock_ids.is_empty() {
        plutus_storage::queries::stocks::list(
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
        .map(|s| (s.id, (s.symbol, s.market_code)))
        .collect()
    } else {
        HashMap::new()
    };

    let filtered: Vec<_> = rows
        .into_iter()
        .filter(|r| {
            // q-filter: rows without a stock can never match a symbol
            // query, so they drop out when q is set.
            if let Some(ref q) = q_upper {
                let Some(stock_id) = r.stock_id else { return false };
                if !meta_map
                    .get(&stock_id)
                    .map(|(sym, _)| sym.to_ascii_uppercase().contains(q))
                    .unwrap_or(false)
                {
                    return false;
                }
            }
            // country filter: cash-movement rows pass through (no
            // market to filter by); stock rows must match the country.
            if let Some(ref codes) = market_codes {
                let Some(stock_id) = r.stock_id else { return true };
                let Some((_, market_code)) = meta_map.get(&stock_id) else {
                    return false;
                };
                if !codes.contains(market_code) {
                    return false;
                }
            }
            true
        })
        .collect();

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

    let mut headers = HeaderMap::new();
    if paginating {
        if let Ok(v) = HeaderValue::from_str(&total.to_string()) {
            headers.insert("X-Total-Count", v);
        }
    }
    let out: Vec<TransactionOut> = page_slice.into_iter().map(Into::into).collect();
    Ok((headers, Json(out)).into_response())
}

pub async fn get(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<Json<TransactionOut>> {
    let user_id = require_user(&actor.0)?;
    let row = plutus_storage::queries::transactions::get(&state.db, user_id, id).await?;
    Ok(Json(row.into()))
}

pub async fn create(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Json(input): Json<TransactionIn>,
) -> ApiResult<Json<TransactionOut>> {
    let user_id = require_user(&actor.0)?;
    // Validate `kind` against the canonical enum BEFORE writing — silently
    // accepting an unknown kind would store the row but break the holdings
    // rollup (which filters by `TransactionKind::from_str`). The parser is
    // case-insensitive and accepts the `withdraw` alias, so most agent
    // dialects work; anything else is rejected here with a clear 400.
    let canonical_kind = TransactionKind::from_str(&input.kind)
        .map_err(|_| {
            ApiError::BadRequest(format!(
                "kind must be one of BUY, SELL, DIVIDEND, FEE, INTEREST, \
                 DEPOSIT, WITHDRAWAL, FX, CORPORATE_ACTION (case-insensitive); \
                 got {:?}",
                input.kind
            ))
        })?
        .as_str();
    let executed_at: jiff::Timestamp = input
        .executed_at
        .parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("executed_at: {e}")))?;
    let metadata_str = match input.source_metadata {
        Some(v) => Some(
            serde_json::to_string(&v)
                .map_err(|e| ApiError::BadRequest(format!("source_metadata: {e}")))?,
        ),
        None => None,
    };
    let row = plutus_storage::queries::transactions::create(
        &state.db,
        plutus_storage::queries::transactions::NewTransaction {
            user_id,
            account_id: input.account_id,
            stock_id: input.stock_id,
            // Stored in the canonical SCREAMING_SNAKE_CASE form so the
            // database is consistent regardless of how the caller wrote it.
            kind: canonical_kind,
            executed_at,
            quantity: input.quantity,
            price: input.price,
            trade_currency: &input.trade_currency,
            commission: input.commission,
            commission_currency: &input.commission_currency,
            tax: input.tax,
            tax_currency: &input.tax_currency,
            fx_rate_to_base: input.fx_rate_to_base,
            external_ref: input.external_ref.as_deref(),
            notes: input.notes.as_deref(),
            source: &input.source,
            source_metadata: metadata_str.as_deref(),
        },
    )
    .await?;
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
) -> ApiResult<axum::http::StatusCode> {
    let user_id = require_user(&actor.0)?;
    plutus_storage::queries::transactions::delete(&state.db, user_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
