//! Shared helpers for batch-insert endpoints.
//!
//! All `/<entity>/batch` routes (catalysts, macro-events, earnings,
//! ohlcv) follow the same contract:
//!   - Request body: `{ items: [<EntityIn>, ...] }`
//!   - Empty list  → 400
//!   - More than `MAX_BATCH` items → 413
//!   - Otherwise   → all-or-nothing transaction at the storage layer
//!
//! Validation that's specific to one entity stays in that entity's
//! handler (e.g. JSONB content shape checks); this module is just the
//! size guard so the rules don't drift between handlers.

use crate::error::{ApiError, ApiResult};

/// Hard cap per request. Picked to fit comfortably inside Postgres's
/// 65535-parameter ceiling for any of our table shapes (the widest is
/// catalysts at 13 columns × 1000 = 13000 params) and to keep a single
/// transaction bounded enough that a partial-failure rollback isn't
/// catastrophic.
pub const MAX_BATCH: usize = 1000;

/// 0 → 400 Bad Request, > MAX_BATCH → 413 Payload Too Large, otherwise OK.
pub fn validate_batch_size(n: usize) -> ApiResult<()> {
    if n == 0 {
        return Err(ApiError::BadRequest(
            "items must not be empty".to_string(),
        ));
    }
    if n > MAX_BATCH {
        return Err(ApiError::PayloadTooLarge(format!(
            "batch size {n} exceeds max {MAX_BATCH}"
        )));
    }
    Ok(())
}
