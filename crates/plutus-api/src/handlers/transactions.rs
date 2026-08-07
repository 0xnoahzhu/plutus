use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;

use std::str::FromStr;

use plutus_core::audit::Actor;
use plutus_core::cost_basis::CostBasisMethod;
use plutus_core::transaction::TransactionKind;

use crate::dto::transaction::{
    TransactionIn, TransactionOut, TransactionPatch, TransactionSummaryOut,
};
use crate::error::{ApiError, ApiResult};
use crate::handlers::access::require_user;
use crate::handlers::pagination::{clamp_limit, clamp_offset, paginate_slice};
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
    /// 1-indexed page (used with `per_page`). When set, response
    /// carries X-Total-Count.
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    /// Direct slice via limit/offset — alternative to page/per_page.
    /// When either is set, response also carries X-Total-Count. If
    /// both forms are present, page/per_page wins.
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Query(f): Query<ListFilter>,
) -> ApiResult<axum::response::Response> {
    let user_id = require_user(&actor.0)?;
    // Pick whichever filter has an index behind it, then narrow in memory
    // for the other. The two used to be an if/else-if chain, which meant
    // `?account_id=1&stock_id=2` silently ignored the stock and returned
    // the whole account — a wrong answer rather than an error.
    let rows = match (f.account_id, f.stock_id) {
        (Some(account_id), stock_id) => {
            let rows = plutus_storage::queries::transactions::list_for_account(
                &state.db, user_id, account_id,
            )
            .await?;
            match stock_id {
                Some(s) => rows
                    .into_iter()
                    .filter(|r| r.stock_id == Some(s))
                    .collect(),
                None => rows,
            }
        }
        (None, Some(stock_id)) => {
            plutus_storage::queries::transactions::list_for_stock(&state.db, user_id, stock_id)
                .await?
        }
        (None, None) => plutus_storage::queries::transactions::list(&state.db, user_id).await?,
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
    // Same dual-pagination contract as /holdings — see that handler's
    // comment for the rationale.
    let paginating =
        f.page.is_some() || f.per_page.is_some() || f.limit.is_some() || f.offset.is_some();
    let page_slice: Vec<_> = if f.page.is_some() || f.per_page.is_some() {
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
    } else if f.limit.is_some() || f.offset.is_some() {
        let limit = clamp_limit(f.limit)?;
        let offset = clamp_offset(f.offset)?;
        paginate_slice(filtered, limit, offset)
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
    let canonical_kind = canonical_kind(&input.kind)?;
    let executed_at = parse_executed_at(&input.executed_at)?;
    let metadata_str = match input.source_metadata {
        Some(v) => Some(stringify_metadata(&v)?),
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

/// `PATCH /transactions/{id}` — correct a row in place.
///
/// The ledger used to be strictly append-only, with a compensating entry
/// as the only fix. That still works and is still right when the original
/// posting was a real event. This exists for the other case — a typo'd
/// price or a commission the statement disagrees with — where a
/// compensating pair just makes the history harder to read. Nothing
/// downstream caches the rollup, so an edit is visible on `/holdings` and
/// the per-stock summary on the very next read.
pub async fn update(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(id): Path<i64>,
    Json(patch): Json<TransactionPatch>,
) -> ApiResult<Json<TransactionOut>> {
    let user_id = require_user(&actor.0)?;
    // Same up-front validation as `create`: an unknown kind would store
    // fine but silently drop the row out of every derived view.
    let kind = match patch.kind.as_deref() {
        Some(k) => Some(canonical_kind(k)?),
        None => None,
    };
    let executed_at = match patch.executed_at.as_deref() {
        Some(s) => Some(parse_executed_at(s)?),
        None => None,
    };
    // Outer `Some` = the key was present. Inner `Some`/`None` = the value
    // to store / an explicit clear-to-NULL.
    let metadata_str: Option<Option<String>> = match patch.source_metadata {
        Some(Some(v)) => Some(Some(stringify_metadata(&v)?)),
        Some(None) => Some(None),
        None => None,
    };
    let row = plutus_storage::queries::transactions::update(
        &state.db,
        user_id,
        id,
        plutus_storage::queries::transactions::TransactionPatch {
            account_id: patch.account_id,
            stock_id: patch.stock_id,
            kind,
            executed_at,
            quantity: patch.quantity,
            price: patch.price,
            trade_currency: patch.trade_currency.as_deref(),
            commission: patch.commission,
            commission_currency: patch.commission_currency.as_deref(),
            tax: patch.tax,
            tax_currency: patch.tax_currency.as_deref(),
            fx_rate_to_base: patch.fx_rate_to_base,
            external_ref: patch.external_ref.as_ref().map(|o| o.as_deref()),
            notes: patch.notes.as_ref().map(|o| o.as_deref()),
            source: patch.source.as_deref(),
            source_metadata: metadata_str.as_ref().map(|o| o.as_deref()),
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

#[derive(Deserialize)]
pub struct StockScopedFilter {
    /// Narrow to one account. Omit for a rollup across every account.
    pub account_id: Option<i64>,
    /// `fifo` (default) | `lifo` | `average`. Summary only.
    pub method: Option<String>,
    /// Cap the number of rows returned, newest first. Summary figures are
    /// unaffected — they always span the full history.
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// `GET /stocks/{id}/transactions` — this stock's slice of the ledger,
/// newest first.
///
/// Same rows as `GET /transactions?stock_id={id}`; this nested form
/// exists so the stock detail page reads like every other per-stock
/// sub-resource (`/stocks/{id}/news`, `/stocks/{id}/earnings`, …).
pub async fn list_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
    Query(f): Query<StockScopedFilter>,
) -> ApiResult<axum::response::Response> {
    let user_id = require_user(&actor.0)?;
    let limit = clamp_limit(f.limit)?;
    let offset = clamp_offset(f.offset)?;
    let rows =
        plutus_storage::queries::transactions::list_for_stock(&state.db, user_id, stock_id).await?;
    let rows: Vec<_> = match f.account_id {
        Some(a) => rows.into_iter().filter(|r| r.account_id == a).collect(),
        None => rows,
    };
    let total = rows.len() as i64;
    let page_slice = paginate_slice(rows, limit, offset);
    let body: Vec<TransactionOut> = page_slice.into_iter().map(Into::into).collect();
    let mut headers = HeaderMap::new();
    if f.limit.is_some() || f.offset.is_some() {
        if let Ok(v) = HeaderValue::from_str(&total.to_string()) {
            headers.insert("X-Total-Count", v);
        }
    }
    Ok((headers, Json(body)).into_response())
}

/// `GET /stocks/{id}/transaction-summary` — the ledger rolled up for one
/// stock: gross bought / sold, lifetime commission and tax, and the open
/// position under the requested cost-basis method.
///
/// Always returns a body, even for a stock with no transactions — the
/// zero-filled shape lets the UI render its panel without special-casing
/// a 404.
pub async fn summary_for_stock(
    State(state): State<AppState>,
    actor: axum::extract::Extension<Actor>,
    Path(stock_id): Path<i64>,
    Query(f): Query<StockScopedFilter>,
) -> ApiResult<Json<TransactionSummaryOut>> {
    let user_id = require_user(&actor.0)?;
    let method_str = f.method.as_deref().unwrap_or("fifo");
    let method = parse_method(method_str)?;
    let summary = plutus_storage::queries::transactions::summarize_for_stock(
        &state.db,
        user_id,
        stock_id,
        f.account_id,
        method,
    )
    .await?;
    Ok(Json(TransactionSummaryOut::from_summary(summary, method_str)))
}

// ── Shared validation ────────────────────────────────────────────────────

/// Validate `kind` against the canonical enum BEFORE writing — silently
/// accepting an unknown kind would store the row but break the holdings
/// rollup (which filters by `TransactionKind::from_str`). The parser is
/// case-insensitive and accepts the `withdraw` alias, so most agent
/// dialects work; anything else is rejected here with a clear 400.
fn canonical_kind(raw: &str) -> ApiResult<&'static str> {
    TransactionKind::from_str(raw)
        .map(TransactionKind::as_str)
        .map_err(|_| {
            ApiError::BadRequest(format!(
                "kind must be one of BUY, SELL, DIVIDEND, FEE, INTEREST, \
                 DEPOSIT, WITHDRAWAL, FX, CORPORATE_ACTION (case-insensitive); \
                 got {raw:?}"
            ))
        })
}

fn parse_executed_at(raw: &str) -> ApiResult<jiff::Timestamp> {
    raw.parse()
        .map_err(|e: jiff::Error| ApiError::BadRequest(format!("executed_at: {e}")))
}

fn stringify_metadata(value: &serde_json::Value) -> ApiResult<String> {
    serde_json::to_string(value)
        .map_err(|e| ApiError::BadRequest(format!("source_metadata: {e}")))
}

/// Mirrors the `/holdings` handler's parsing so the same query string
/// means the same thing on both endpoints.
fn parse_method(raw: &str) -> ApiResult<CostBasisMethod> {
    match raw {
        "fifo" => Ok(CostBasisMethod::Fifo),
        "lifo" => Ok(CostBasisMethod::Lifo),
        "average" => Ok(CostBasisMethod::Average),
        other => Err(ApiError::BadRequest(format!(
            "method must be fifo/lifo/average; got {other}"
        ))),
    }
}
