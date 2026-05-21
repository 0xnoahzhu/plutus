//! Auto-record an `audit_log` row for every successful mutating
//! request. Lives in its own file because it leans on a different
//! state than the other auth middleware — it needs the typed
//! `AppState` so the recorder can hit storage.
//!
//! Skipped:
//! - Read methods (GET / HEAD / OPTIONS) — audit is for writes only.
//! - Non-2xx responses — failed writes didn't change anything.
//! - Auth + meta + docs paths — too noisy for the activity feed.
//! - Path with no extractable `entity_type` segment.
//!
//! Entity id resolution:
//! - `/<thing>/{id}` → entity_id = `{id}`
//! - `/<thing>/batch` → entity_id = `"batch"` (the action covers many
//!   rows; the diff is too large to inline).
//! - `/<thing>` POST → entity_id = `""` (we can't read it without
//!   buffering the response body, and that hurts streaming).

use axum::body::Body;
use axum::extract::Request;
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;

use plutus_core::audit::{Actor, AuditAction};

use crate::state::AppState;

/// Paths whose writes we deliberately don't record. Login, logout,
/// password changes, doc fetches, health probes, the audit list
/// itself. Matched against the path post-strip of `/api/v1`.
const SKIP_PREFIXES: &[&str] = &[
    "/auth/",
    "/healthz",
    "/openapi.json",
    "/docs",
    "/audit",
];

pub async fn record_writes(state: AppState, req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let mutating = matches!(method, Method::POST | Method::PATCH | Method::PUT | Method::DELETE);
    let skipped = SKIP_PREFIXES.iter().any(|p| path.starts_with(p));
    // Snapshot the actor now — the extension is still there, the
    // request hasn't been consumed yet.
    let actor = req.extensions().get::<Actor>().cloned();
    let request_id = generate_request_id();

    let response = next.run(req).await;

    // Decide AFTER the response so we can gate on the status.
    if !mutating || skipped || !response.status().is_success() {
        return response;
    }
    let Some(actor) = actor else {
        return response;
    };
    let Some(action) = method_to_action(&method) else {
        return response;
    };
    let (entity_type, entity_id) = parse_entity(&path);
    if entity_type.is_empty() {
        return response;
    }

    // Fire-and-forget. The audit insert shouldn't add latency to the
    // user's request, and failure to record an audit row should never
    // fail the user's operation. The handle is dropped immediately;
    // tokio keeps the task alive on the default runtime.
    let db = state.db.clone();
    tokio::spawn(async move {
        let entry = plutus_storage::queries::audit::RecordAudit {
            entity_type: &entity_type,
            entity_id,
            action,
            actor: &actor,
            before: None,
            after: None,
            request_id: &request_id,
        };
        if let Err(e) = plutus_storage::queries::audit::record(&db, entry).await {
            tracing::warn!("audit::record failed for {method} {path}: {e}");
        }
    });

    response
}

fn method_to_action(m: &Method) -> Option<AuditAction> {
    match *m {
        Method::POST => Some(AuditAction::Create),
        Method::PATCH | Method::PUT => Some(AuditAction::Update),
        Method::DELETE => Some(AuditAction::Delete),
        _ => None,
    }
}

/// Map a URL path to `(entity_type, entity_id)`.
///
/// Walks segments left-to-right. Each non-id segment replaces the
/// running entity (and clears the running id, since that id belonged
/// to the previous resource). Each id-like segment (digits, or the
/// sub-action keywords `batch` / `close`) attaches to the current
/// entity. The "deepest resource wins" rule means a POST to
/// `/stocks/4/ohlcv` audits as `ohlcv`, not `stock`.
///
/// Examples:
/// - `/catalysts/42`            → (`catalyst`, `42`)
/// - `/catalysts/batch`         → (`catalyst`, `batch`)
/// - `/catalysts`               → (`catalyst`, ``)
/// - `/stocks/4/ohlcv`          → (`ohlcv`, ``) — child resource
/// - `/macro/events/42`         → (`macro_event`, `42`)
fn parse_entity(path: &str) -> (String, String) {
    let parts: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return (String::new(), String::new());
    }
    let is_id = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    let is_keyword_id = |s: &str| matches!(s, "batch" | "close");

    let mut entity = String::new();
    let mut entity_id = String::new();
    for seg in &parts {
        if is_id(seg) || is_keyword_id(seg) {
            if !entity.is_empty() {
                entity_id = (*seg).to_string();
            }
        } else {
            entity = singularize(seg);
            entity_id.clear();
        }
    }
    // Namespacing fixups for two-segment logical entities. `events`
    // after `macro/` is a distinct table from any hypothetical generic
    // `event`; same for analyst sub-resources.
    if parts.len() >= 2 && parts[0] == "macro" && entity == "event" {
        entity = "macro_event".to_string();
    }
    if parts.len() >= 2 && parts[0] == "analyst" && (entity == "estimate" || entity == "rating") {
        entity = format!("analyst_{}", entity);
    }
    (entity, entity_id)
}

/// Naïve plural→singular: `categories` → `category`, `stocks` →
/// `stock`. Doesn't handle every irregular plural, but the API uses
/// regular plurals throughout.
fn singularize(s: &str) -> String {
    if s.ends_with("ies") && s.len() > 3 {
        format!("{}y", &s[..s.len() - 3])
    } else if s.ends_with('s') && s.len() > 1 {
        s[..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Per-request id, base64-url 22-char (16 bytes). Matches the
/// session-id helper's encoding style for consistency.
fn generate_request_id() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singularize_basic() {
        assert_eq!(singularize("stocks"), "stock");
        assert_eq!(singularize("categories"), "category");
        assert_eq!(singularize("watchlist"), "watchlist");
    }

    #[test]
    fn parse_entity_simple() {
        assert_eq!(parse_entity("/catalysts/42"), ("catalyst".into(), "42".into()));
        assert_eq!(parse_entity("/catalysts"), ("catalyst".into(), String::new()));
        assert_eq!(parse_entity("/catalysts/batch"), ("catalyst".into(), "batch".into()));
    }

    #[test]
    fn parse_entity_child() {
        // POST /stocks/4/ohlcv records an OHLCV bar, not a stock.
        assert_eq!(parse_entity("/stocks/4/ohlcv"), ("ohlcv".into(), String::new()));
    }

    #[test]
    fn parse_entity_namespaced() {
        assert_eq!(parse_entity("/macro/events/42"), ("macro_event".into(), "42".into()));
        assert_eq!(parse_entity("/analyst/estimates"), ("analyst_estimate".into(), String::new()));
    }
}
