//! Shared pagination chrome for list endpoints.
//!
//! Most `/stocks/:id/<subresource>` GETs and the `/<entity>` index
//! endpoints accept `?limit=N&offset=M`. When the caller supplies
//! `limit` (or `offset`) we paginate and send back an `X-Total-Count`
//! header so the caller can compute page counts. Without those
//! params, the endpoint returns the full result set — the way agent
//! bulk-fetches need to work.
//!
//! Why headers and not a wrapper body: `Vec<T>` stays the response
//! shape no matter the mode, so existing agent code that just
//! consumes the JSON array doesn't break.

use axum::http::{HeaderMap, HeaderValue};
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};

/// Hard upper bound on `?limit=`. A single request can never pull
/// more than 1000 rows in one shot regardless of what the caller
/// passes; bulk imports use the natural-key upsert path instead.
const MAX_LIMIT: i64 = 1000;

#[derive(Debug, Deserialize)]
pub struct PaginationFilter {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl PaginationFilter {
    /// Whether the caller wants the paginated path (any limit or
    /// offset set → set the X-Total-Count header).
    pub fn is_paginating(&self) -> bool {
        self.limit.is_some() || self.offset.is_some()
    }
}

/// Validate and clamp `limit` to `[1, MAX_LIMIT]`. None passes
/// through unchanged so the storage layer treats it as "no cap".
/// Negative / zero values reject with 400.
pub fn clamp_limit(limit: Option<i64>) -> ApiResult<Option<usize>> {
    match limit {
        None => Ok(None),
        Some(n) if n <= 0 => Err(ApiError::BadRequest("limit must be > 0".into())),
        Some(n) if n > MAX_LIMIT => Err(ApiError::BadRequest(format!(
            "limit must be ≤ {MAX_LIMIT}"
        ))),
        Some(n) => Ok(Some(n as usize)),
    }
}

/// Validate offset (non-negative). Returns None when input is None.
pub fn clamp_offset(offset: Option<i64>) -> ApiResult<Option<usize>> {
    match offset {
        None => Ok(None),
        Some(n) if n < 0 => Err(ApiError::BadRequest("offset must be ≥ 0".into())),
        Some(n) => Ok(Some(n as usize)),
    }
}

/// Build a HeaderMap with `X-Total-Count: <total>`. The handler
/// composes this with the JSON body via `(headers, Json(body)).into_response()`.
pub fn paginated_response_headers(total: i64) -> HeaderMap {
    let mut h = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&total.to_string()) {
        h.insert("X-Total-Count", v);
    }
    h
}
