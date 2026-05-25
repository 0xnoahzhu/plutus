//! OpenAPI 3.1 spec assembly.
//!
//! Strategy: enumerate every DTO in [`Schemas`] via `#[derive(utoipa::OpenApi)]`
//! so utoipa generates the JSON-Schema component objects from the typed
//! definitions in `dto/*.rs`. The path inventory stays hand-written here
//! because axum's router macros don't carry the rich metadata utoipa needs
//! to derive paths automatically — instead each operation references the
//! relevant component via `$ref`.
//!
//! A handful of small structs (`LoginIn`, `LoginOut`, `MeOut`, `AuditEntryOut`)
//! live in handler files and don't derive `ToSchema`; their request/response
//! shapes are inlined directly in the JSON below.

use serde_json::{json, Map, Value};
use utoipa::OpenApi;

use crate::dto::{
    account::{AccountIn, AccountOut},
    analyst::{
        AnalystEstimateBatchIn, AnalystEstimateBatchOut, AnalystEstimateIn,
        AnalystEstimateOut, AnalystRatingBatchIn, AnalystRatingBatchOut,
        AnalystRatingIn, AnalystRatingOut,
    },
    broker::BrokerOut,
    catalyst::{CatalystBatchIn, CatalystBatchOut, CatalystIn, CatalystOut},
    connect::{ConnectFlowIn, ConnectFlowOut, ConnectHoldingsIn, ConnectHoldingsOut},
    correlation::{
        CorrelationPairIn, CorrelationPairOut, CorrelationRunIn, CorrelationRunOut, UniverseIn,
        UniverseOut,
    },
    earnings::{EarningsBatchIn, EarningsBatchOut, EarningsIn, EarningsOut},
    filing::{FilingIn, FilingOut},
    fundamentals::{FundamentalsIn, FundamentalsOut},
    fx::{FxIn, FxOut},
    holding::HoldingOut,
    portfolio::DailyValueOut,
    insider::{InsiderTxnBatchIn, InsiderTxnBatchOut, InsiderTxnIn, InsiderTxnOut},
    macro_event::{MacroEventBatchIn, MacroEventBatchOut, MacroEventIn, MacroEventOut},
    macros::{MacroIndicatorIn, MacroIndicatorOut, MacroObservationIn, MacroObservationOut},
    market::MarketOut,
    market_brief::{MarketBriefIn, MarketBriefOut},
    news::{
        NewsCountryLinkIn, NewsCountryLinkOut, NewsIn, NewsMacroLinkIn, NewsMacroLinkOut, NewsOut,
        NewsPatch, NewsSectorLinkIn, NewsSectorLinkOut, NewsStockLinkIn, NewsStockLinkOut,
    },
    ohlcv::{OhlcvBatchIn, OhlcvBatchOut, OhlcvIn, OhlcvOut},
    pending_order::{PendingOrderIn, PendingOrderOut, PendingOrderPatch},
    portfolio_review::{PortfolioReviewIn, PortfolioReviewOut},
    recommendation::{RecommendationClosePatch, RecommendationIn, RecommendationOut},
    screener::{ScreenerHitIn, ScreenerHitOut, ScreenerRunIn, ScreenerRunOut},
    sector::{SectorIn, SectorOut},
    self_exam::{SelfExamIn, SelfExamOut},
    stock::{StockIn, StockOut, StockPatch},
    token::{TokenCreatedOut, TokenIn, TokenOut},
    trade_plan::{
        TradePlanIn, TradePlanLevelIn, TradePlanLevelOut, TradePlanLevelPatch, TradePlanOut,
        TradePlanPatch,
    },
    transaction::{TransactionIn, TransactionOut},
    user::{
        AdminCreateUserIn, AdminResetPasswordIn, AdminUpdateCountriesIn, ChangePasswordIn, UserOut,
    },
    watchlist::{WatchlistItemIn, WatchlistItemOut},
    watchlist_report::{WatchlistReportIn, WatchlistReportOut},
};
use crate::handlers::admin::brokers::{AdminCreateBrokerIn, AdminUpdateBrokerIn};

/// Marker type for `utoipa` derive. The struct itself is never instantiated;
/// only its `OpenApi` impl is used, which exposes the registered component
/// schemas for every DTO listed below.
#[derive(OpenApi)]
#[openapi(components(schemas(
    AccountIn, AccountOut,
    AdminCreateBrokerIn, AdminUpdateBrokerIn,
    AnalystEstimateIn, AnalystEstimateOut, AnalystEstimateBatchIn, AnalystEstimateBatchOut,
    AnalystRatingIn, AnalystRatingOut, AnalystRatingBatchIn, AnalystRatingBatchOut,
    BrokerOut,
    CatalystIn, CatalystOut, CatalystBatchIn, CatalystBatchOut,
    ConnectFlowIn, ConnectFlowOut,
    ConnectHoldingsIn, ConnectHoldingsOut,
    CorrelationPairIn, CorrelationPairOut,
    CorrelationRunIn, CorrelationRunOut,
    UniverseIn, UniverseOut,
    EarningsIn, EarningsOut, EarningsBatchIn, EarningsBatchOut,
    FilingIn, FilingOut,
    FundamentalsIn, FundamentalsOut,
    FxIn, FxOut,
    HoldingOut,
    DailyValueOut,
    InsiderTxnIn, InsiderTxnOut, InsiderTxnBatchIn, InsiderTxnBatchOut,
    MacroEventIn, MacroEventOut, MacroEventBatchIn, MacroEventBatchOut,
    MacroIndicatorIn, MacroIndicatorOut,
    MacroObservationIn, MacroObservationOut,
    MarketOut,
    MarketBriefIn, MarketBriefOut,
    NewsIn, NewsOut, NewsPatch,
    NewsStockLinkIn, NewsStockLinkOut,
    NewsSectorLinkIn, NewsSectorLinkOut,
    NewsMacroLinkIn, NewsMacroLinkOut,
    NewsCountryLinkIn, NewsCountryLinkOut,
    OhlcvIn, OhlcvOut, OhlcvBatchIn, OhlcvBatchOut,
    PendingOrderIn, PendingOrderOut, PendingOrderPatch,
    PortfolioReviewIn, PortfolioReviewOut,
    RecommendationIn, RecommendationOut, RecommendationClosePatch,
    ScreenerHitIn, ScreenerHitOut,
    ScreenerRunIn, ScreenerRunOut,
    SectorIn, SectorOut,
    SelfExamIn, SelfExamOut,
    StockIn, StockOut, StockPatch,
    TokenIn, TokenOut, TokenCreatedOut,
    TradePlanIn, TradePlanOut, TradePlanPatch,
    TradePlanLevelIn, TradePlanLevelOut, TradePlanLevelPatch,
    TransactionIn, TransactionOut,
    UserOut, AdminCreateUserIn, AdminResetPasswordIn, AdminUpdateCountriesIn, ChangePasswordIn,
    WatchlistItemIn, WatchlistItemOut,
    WatchlistReportIn, WatchlistReportOut,
)))]
struct Schemas;

pub fn spec() -> Value {
    // Pull the derived component schemas as JSON so we can splice them into
    // the hand-written paths spec without losing field-level information.
    let derived = <Schemas as OpenApi>::openapi();
    let schemas_value = serde_json::to_value(&derived.components)
        .ok()
        .and_then(|mut c| c.as_object_mut().and_then(|o| o.remove("schemas")))
        .unwrap_or(Value::Object(Map::new()));

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Plutus API",
            "version": "0.1.0",
            "description": "Personal investment data store. The hermes AI agent writes data via this API; the web UI is the human-side viewer.\n\n## Mounting and auth\n\nAll routes live under `/api/v1`. Auth is optional by default (`PLUTUS_API_REQUIRE_AUTH=false`) — flip the env var to require either a session cookie (`plutus_session`) or a bearer token on every call. Agents should always send a bearer token.\n\n## Identity model\n\n- **Regular users** — rows in the `users` table, Argon2-hashed passwords. Each user has an `allowed_countries` allowlist (subset of `[US, HK, CN]`); list endpoints that scope by country honor it.\n- **Admin** — env-only (`PLUTUS_ADMIN_USERNAME` / `PLUTUS_ADMIN_PASSWORD`). Not a row in `users`. Manages users via `/admin/*`; cannot access per-user data routes (those return 403).\n- **Session cookie** — `plutus_session` is set by `POST /auth/login` and lasts 30 days. The cookie value is a 32-byte random id; the server-side row in `web_sessions` carries the identity.\n- **Bearer token** — long-lived API token minted via `POST /tokens`. Plaintext is shown once at creation AND stored alongside the hash so the listing UI can show masked tokens with a copy button. Set `Authorization: Bearer <token>` on every call.\n\n## Translatable text — the `content JSONB` pattern\n\nEvery entity with human-readable text uses a single `content` column shaped as:\n\n```json\n{ \"<locale>\": { \"title\": \"...\", \"summary_md\": \"...\" } }\n```\n\n**Writes** — POST/PATCH bodies always include the full multi-locale blob. The server stores it verbatim. There is no separate \"set just English\" endpoint; build the whole object on the client.\n\n**Reads** — list/get endpoints accept `?locale=en` (or `zh-CN`) and return the localized fields flattened to top-level: `title`, `summary_md`, `headline`, `description_md`, `bull_case_md`, `bear_case_md`, `notes`, `rationale_md`, etc. Locale falls back to `en` for missing keys. The flattened shape is purely a read-time projection — the source of truth is still the full `content` blob in the DB.\n\n## Per-user isolation\n\nEntities that store agent outputs or user decisions (`catalysts`, `screener_runs`, `portfolio_reviews`, `recommendations`, `self_exams`, `correlation_runs`, `universe_definitions`, `trade_plans`, `pending_orders`, `transactions`, `accounts`, `watchlist_items`, `watchlist_reports`) carry a `user_id` column. Reads filter by the authenticated caller's `user_id`. Cross-user reads are not exposed.\n\nReference data (`stocks`, `markets`, `brokers`, `sectors`, `news_items`, `macro_*`, `earnings_events`, `market_briefs`, `ohlcv_daily`, `filings`, `fundamentals_quarterly`, `analyst_*`, `insider_transactions`, `connect_*`) is shared across all users.\n\n## Idempotent writes (\"upsert\" / dedup)\n\nSeveral entities have a natural unique key and accept idempotent writes:\n\n| Entity | Conflict key | Endpoint |\n|---|---|---|\n| `catalysts` | `(user_id, catalyst_kind, catalyst_date, stock_id, sector_code, country, source)` | `POST /catalysts`, `POST /catalysts/batch` |\n| `earnings_events` | `(stock_id, fiscal_year, fiscal_period)` | `POST /earnings`, `POST /earnings/batch` |\n| `macro_events` | `(indicator_code, event_date)` | `POST /macro/events`, `POST /macro/events/batch` |\n| `macro_observations` | `(indicator_code, obs_date)` | `POST /macro/observations` |\n| `ohlcv_daily` | `(stock_id, trade_date)` | `POST /ohlcv/batch` |\n| `screener_runs` | `(user_id, name, kind, run_date)` | `POST /screener-runs` |\n| `portfolio_reviews` | `(user_id, kind, period_start)` | `POST /portfolio-reviews` |\n| `self_exams` | `(user_id, kind, period_start)` | `POST /self-exams` |\n| `watchlist_reports` | `(user_id, kind, period_start)` | `POST /watchlist/reports` |\n| `market_briefs` | `(user_id, country, kind, trade_date)` | `POST /market-briefs` |\n\nRe-POSTing the same natural key refreshes the mutable fields and bumps `updated_at`. The `source` column on `catalysts` discriminates provenance: a row added by `source=\"agent\"` and one by `source=\"manual\"` for the same nominal event coexist as distinct rows.\n\n## Batch writes\n\n`POST /<entity>/batch` accepts `{ \"items\": [...] }`, validates the whole batch up front, and runs all upserts in one transaction. Caps at 1000 items; empty list returns 400. A single bad row rolls everything back.\n\n## Stock search\n\n`GET /stocks` is also a search endpoint:\n\n- `?symbol=AAPL` — exact ticker match, case-insensitive. Returns 0 or 1 row.\n- `?q=Apple` — substring search across `symbol` and `content.<locale>.name`, case-insensitive, ranked by `symbol` priority. Returns up to `limit` rows (default 50, max 200).\n- `?country=US` — country filter (post-DB; matches `stocks.market_code` through the country → MIC mapping).\n\nAll three can be combined.\n\n## Audit log\n\nEvery write through this API is recorded in `audit_log` with actor, route, status, and a diff. Read via `GET /audit` (admin sees everything; users see their own writes)."
        },
        "servers": [{ "url": "/api/v1" }],
        "tags": tags(),
        "paths": paths(),
        "components": {
            "schemas": schemas_value,
            "securitySchemes": {
                "bearer": {
                    "type": "http",
                    "scheme": "bearer",
                    "description": "API tokens minted via `POST /tokens`. Token is shown once at creation."
                },
                "session": {
                    "type": "apiKey",
                    "in": "cookie",
                    "name": "plutus_session",
                    "description": "Set by `POST /auth/login`. Cleared by `POST /auth/logout`."
                }
            },
            "parameters": {
                "Locale": {
                    "name": "locale",
                    "in": "query",
                    "schema": { "type": "string", "enum": ["en", "zh-CN"] },
                    "description": "Resolved language. Translatable fields fall back to English when missing."
                },
                "Country": {
                    "name": "country",
                    "in": "query",
                    "schema": { "type": "string", "enum": ["US", "HK", "CN"] }
                },
                "PathId": {
                    "name": "id",
                    "in": "path",
                    "required": true,
                    "schema": { "type": "integer", "format": "int64" }
                }
            }
        }
    })
}

fn tags() -> Value {
    json!([
        { "name": "meta", "description": "Liveness, root index, this spec." },
        { "name": "auth", "description": "Username + password login, session cookie, self-service password change." },
        { "name": "admin", "description": "Admin-only endpoints — manage user accounts. Reachable when authenticated as PLUTUS_ADMIN_USERNAME." },
        { "name": "tokens", "description": "Long-lived bearer tokens scoped to one user." },
        { "name": "stocks", "description": "Tradable instruments + metadata + translations." },
        { "name": "watchlists", "description": "The user's watchlist — a flat list of stocks plus daily / weekly reports." },
        { "name": "transactions", "description": "Trade ledger; holdings are derived from this." },
        { "name": "holdings", "description": "Derived open positions per cost basis." },
        { "name": "trade-plans", "description": "Per-user, per-stock plans with buy / stop-loss / take-profit / trim price points." },
        { "name": "pending-orders", "description": "Per-user limit orders the user has placed with their broker (or intends to place)." },
        { "name": "news", "description": "Articles + per-entity link tables + translations." },
        { "name": "calendar", "description": "Briefs, earnings, macro events, catalysts." },
        { "name": "macros", "description": "Macro indicators + observations + events." },
        { "name": "analyst", "description": "External estimates + ratings." },
        { "name": "insider", "description": "Insider transactions." },
        { "name": "filings", "description": "SEC / HKEX / CSRC filings." },
        { "name": "fundamentals", "description": "Per-period fundamentals snapshots." },
        { "name": "connect", "description": "HK Stock Connect flow + holdings." },
        { "name": "agent-outputs", "description": "Screener runs, recommendations, reviews, self-exams, correlations." },
        { "name": "unread", "description": "Per-user unread state. Detail GETs on event-like entities mark the item read as a side effect; counts power the sidebar badges." },
        { "name": "audit", "description": "Server-side write log." },
        { "name": "reference", "description": "Markets, brokers, accounts, FX, sectors." }
    ])
}

// ── shared parameter refs ────────────────────────────────────────────────
fn id_param() -> Value {
    json!({ "$ref": "#/components/parameters/PathId" })
}
fn locale_param() -> Value {
    json!({ "$ref": "#/components/parameters/Locale" })
}
fn country_param() -> Value {
    json!({ "$ref": "#/components/parameters/Country" })
}
fn path_str_param(name: &str) -> Value {
    json!({ "name": name, "in": "path", "required": true, "schema": { "type": "string" } })
}
fn query_str_param(name: &str) -> Value {
    json!({ "name": name, "in": "query", "schema": { "type": "string" } })
}
fn query_i64_param(name: &str) -> Value {
    json!({ "name": name, "in": "query", "schema": { "type": "integer", "format": "int64" } })
}

// ── shared schema refs ───────────────────────────────────────────────────
fn schema_ref(name: &str) -> Value {
    json!({ "$ref": format!("#/components/schemas/{name}") })
}
fn list_schema(name: &str) -> Value {
    json!({ "type": "array", "items": schema_ref(name) })
}

/// `requestBody` block referencing a component schema by name.
fn body(name: &str) -> Value {
    json!({
        "required": true,
        "content": { "application/json": { "schema": schema_ref(name) } }
    })
}

/// `200 OK` response returning a single item.
fn ok_item(name: &str) -> Value {
    json!({
        "200": {
            "description": "OK",
            "content": { "application/json": { "schema": schema_ref(name) } }
        }
    })
}

/// `200 OK` response returning an array of items.
fn ok_list(name: &str) -> Value {
    json!({
        "200": {
            "description": "OK",
            "content": { "application/json": { "schema": list_schema(name) } }
        }
    })
}

/// `200 OK` response with an inline schema.
fn ok_inline(schema: Value) -> Value {
    json!({
        "200": {
            "description": "OK",
            "content": { "application/json": { "schema": schema } }
        }
    })
}

/// `204 No Content` response, used for deletes / revokes.
fn no_content() -> Value {
    json!({ "204": { "description": "Deleted." } })
}

// ── operation helpers ────────────────────────────────────────────────────
fn list_op(tag: &str, summary: &str, item_schema: &str) -> Value {
    json!({ "tags": [tag], "summary": summary, "responses": ok_list(item_schema) })
}
fn list_op_p(tag: &str, summary: &str, item_schema: &str, params: Vec<Value>) -> Value {
    json!({
        "tags": [tag], "summary": summary,
        "parameters": params,
        "responses": ok_list(item_schema)
    })
}
fn get_op(tag: &str, summary: &str, item_schema: &str) -> Value {
    json!({ "tags": [tag], "summary": summary, "responses": ok_item(item_schema) })
}
fn post_op(tag: &str, summary: &str, body_schema: &str, response_schema: &str) -> Value {
    json!({
        "tags": [tag],
        "summary": summary,
        "requestBody": body(body_schema),
        "responses": ok_item(response_schema)
    })
}
fn patch_op(tag: &str, summary: &str, body_schema: &str, response_schema: &str) -> Value {
    json!({
        "tags": [tag],
        "summary": summary,
        "requestBody": body(body_schema),
        "responses": ok_item(response_schema)
    })
}
fn delete_op(tag: &str, summary: &str) -> Value {
    json!({ "tags": [tag], "summary": summary, "responses": no_content() })
}

fn paths() -> Value {
    let mut paths = Map::new();

    // ── meta ──────────────────────────────────────────────────────────────
    paths.insert("/healthz".into(), json!({
        "get": {
            "tags": ["meta"],
            "summary": "Liveness probe — returns plain text 'ok'.",
            "responses": {
                "200": {
                    "description": "OK",
                    "content": { "text/plain": { "schema": { "type": "string", "example": "ok" } } }
                }
            }
        }
    }));
    paths.insert("/openapi.json".into(), json!({
        "get": {
            "tags": ["meta"],
            "summary": "This OpenAPI document.",
            "responses": {
                "200": {
                    "description": "OpenAPI 3.1 spec as JSON.",
                    "content": { "application/json": { "schema": { "type": "object" } } }
                }
            }
        }
    }));
    paths.insert("/docs".into(), json!({
        "get": {
            "tags": ["meta"],
            "summary": "Browseable Scalar UI rendering this spec.",
            "responses": {
                "200": {
                    "description": "HTML page.",
                    "content": { "text/html": { "schema": { "type": "string" } } }
                }
            }
        }
    }));

    // ── auth ──────────────────────────────────────────────────────────────
    paths.insert("/auth/login".into(), json!({
        "post": {
            "tags": ["auth"],
            "summary": "Verify username + password, create session cookie.",
            "description": "Dispatches by username: if `username == PLUTUS_ADMIN_USERNAME`, the password is checked against the env-var plaintext (admin path). Otherwise the user is looked up in the `users` table and the password is verified against the stored Argon2 hash. Either way, success writes a session row and returns `Set-Cookie: plutus_session=<id>`.\n\nWhen `password_reset_required` is `true` in the response, the frontend must route the user to `/change-password` before any data-bearing call — most routes return 403 until the change is made.",
            "requestBody": {
                "required": true,
                "content": { "application/json": { "schema": {
                    "type": "object",
                    "required": ["username", "password"],
                    "properties": {
                        "username": { "type": "string" },
                        "password": { "type": "string" }
                    }
                }}}
            },
            "responses": {
                "200": {
                    "description": "Session cookie set; `Set-Cookie: plutus_session=…`.",
                    "content": { "application/json": { "schema": {
                        "type": "object",
                        "required": ["ok", "password_reset_required", "is_admin", "username"],
                        "properties": {
                            "ok": { "type": "boolean" },
                            "password_reset_required": { "type": "boolean" },
                            "is_admin": { "type": "boolean" },
                            "username": { "type": "string" }
                        }
                    }}}
                },
                "400": { "description": "Missing username or password." },
                "401": { "description": "Wrong username or password." }
            }
        }
    }));
    paths.insert("/auth/logout".into(), json!({
        "post": {
            "tags": ["auth"],
            "summary": "Delete the session row + clear cookie.",
            "responses": {
                "200": {
                    "description": "Cookie cleared.",
                    "content": { "application/json": { "schema": {
                        "type": "object",
                        "required": ["ok"],
                        "properties": { "ok": { "type": "boolean" } }
                    }}}
                }
            }
        }
    }));
    paths.insert("/auth/me".into(), json!({
        "get": {
            "tags": ["auth"],
            "summary": "Identity of the current caller.",
            "description": "Returns `allowed_countries` so the caller can scope queries to the user's permitted markets. Empty list = admin / anonymous / system (no scope; treat as 'see everything').",
            "responses": {
                "200": {
                    "description": "Actor info.",
                    "content": { "application/json": { "schema": {
                        "type": "object",
                        "required": ["kind", "label", "is_admin", "allowed_countries"],
                        "properties": {
                            "kind": { "type": "string", "enum": ["web", "api_token", "admin", "anonymous", "system"] },
                            "label": { "type": "string" },
                            "user_id": { "type": "integer", "format": "int64", "nullable": true },
                            "token_id": { "type": "integer", "format": "int64", "nullable": true },
                            "is_admin": { "type": "boolean" },
                            "allowed_countries": {
                                "type": "array",
                                "items": { "type": "string", "enum": ["US", "HK", "CN"] }
                            }
                        }
                    }}}
                }
            }
        }
    }));
    paths.insert("/auth/change-password".into(), json!({
        "post": {
            "tags": ["auth"],
            "summary": "Self-service password change.",
            "description": "The authenticated user supplies their current password (or any value when `password_reset_required` was true — the admin reset already invalidated that authority) along with the new password twice. On success the new Argon2 hash is written and the reset-required flag is cleared.",
            "requestBody": body("ChangePasswordIn"),
            "responses": {
                "200": {
                    "description": "Password updated.",
                    "content": { "application/json": { "schema": {
                        "type": "object",
                        "required": ["ok"],
                        "properties": { "ok": { "type": "boolean" } }
                    }}}
                },
                "400": { "description": "Mismatched / empty new password." },
                "401": { "description": "Current password did not verify." },
                "403": { "description": "Not signed in as a regular user (admin and bearer-token callers cannot use this route)." }
            }
        }
    }));

    // ── admin ─────────────────────────────────────────────────────────────
    paths.insert("/admin/users".into(), json!({
        "get": {
            "tags": ["admin"],
            "summary": "List all user accounts. Admin only.",
            "responses": ok_list("UserOut")
        },
        "post": {
            "tags": ["admin"],
            "summary": "Create a new user account. Admin only.",
            "requestBody": body("AdminCreateUserIn"),
            "responses": ok_item("UserOut")
        }
    }));
    paths.insert("/admin/users/{id}".into(), json!({
        "parameters": [id_param()],
        "delete": delete_op("admin", "Delete a user account. Admin only.")
    }));
    paths.insert("/admin/users/{id}/reset-password".into(), json!({
        "parameters": [id_param()],
        "post": {
            "tags": ["admin"],
            "summary": "Force-reset a user's password.",
            "description": "Admin sets a new temporary password and flips `password_reset_required=true`. The user can still log in with the temp value, but every route except `/auth/me`, `/auth/logout`, and `/auth/change-password` will return 403 until they change it.",
            "requestBody": body("AdminResetPasswordIn"),
            "responses": ok_item("UserOut")
        }
    }));
    paths.insert("/admin/users/{id}/countries".into(), json!({
        "parameters": [id_param()],
        "post": {
            "tags": ["admin"],
            "summary": "Replace a user's market-country allowlist.",
            "description": "Sets which market tabs the user can see in the web UI. Each entry must be in `{US, HK, CN}`; empty lists are rejected (a user with zero markets has no tab to land on). Takes effect on the user's next request — `/auth/me` reflects it immediately.",
            "requestBody": body("AdminUpdateCountriesIn"),
            "responses": ok_item("UserOut")
        }
    }));
    paths.insert("/admin/brokers".into(), json!({
        "get": {
            "tags": ["admin"],
            "summary": "List broker entries. Admin only.",
            "responses": ok_list("BrokerOut")
        },
        "post": {
            "tags": ["admin"],
            "summary": "Register a new broker. Admin only.",
            "requestBody": body("AdminCreateBrokerIn"),
            "responses": ok_item("BrokerOut")
        }
    }));
    paths.insert("/admin/brokers/{id}".into(), json!({
        "parameters": [id_param()],
        "patch": {
            "tags": ["admin"],
            "summary": "Rename a broker. Admin only.",
            "requestBody": body("AdminUpdateBrokerIn"),
            "responses": ok_item("BrokerOut")
        },
        "delete": delete_op("admin", "Delete a broker (refused while any account references it).")
    }));
    paths.insert("/admin/tokens".into(), json!({
        "get": {
            "tags": ["admin"],
            "summary": "List admin-grade API tokens. Admin only.",
            "responses": ok_list("TokenOut")
        },
        "post": {
            "tags": ["admin"],
            "summary": "Mint an admin-grade token; full secret shown once. Admin only.",
            "description": "Distinct from `/tokens` (per-user). These tokens authenticate as admin and can call every `/admin/*` route.",
            "requestBody": body("TokenIn"),
            "responses": ok_item("TokenCreatedOut")
        }
    }));
    paths.insert("/admin/tokens/{id}".into(), json!({
        "parameters": [id_param()],
        "delete": delete_op("admin", "Hard-delete an admin token (bearer requests start returning 401).")
    }));

    // ── tokens (web only) ─────────────────────────────────────────────────
    paths.insert("/tokens".into(), json!({
        "get": list_op("tokens", "List API tokens (cookie-only).", "TokenOut"),
        "post": post_op(
            "tokens",
            "Mint a new bearer token; full secret shown once.",
            "TokenIn",
            "TokenCreatedOut"
        )
    }));
    paths.insert("/tokens/{id}".into(), json!({
        "parameters": [id_param()],
        "delete": delete_op("tokens", "Revoke a token.")
    }));

    // ── reference ─────────────────────────────────────────────────────────
    paths.insert("/markets".into(), json!({
        "get": list_op("reference", "List markets (MIC codes, timezones, lot sizes).", "MarketOut")
    }));
    paths.insert("/brokers".into(), json!({
        "get": list_op("reference", "List broker entries.", "BrokerOut")
    }));
    paths.insert("/accounts".into(), json!({
        "get": list_op("reference", "List accounts.", "AccountOut"),
        "post": post_op("reference", "Create an account.", "AccountIn", "AccountOut")
    }));
    paths.insert("/accounts/{id}".into(), json!({
        "parameters": [id_param()],
        "get": get_op("reference", "Fetch one account.", "AccountOut"),
        "delete": delete_op("reference", "Delete an account (refused while any transaction references it).")
    }));
    paths.insert("/sectors".into(), json!({
        "get": list_op("reference", "List sector codes (ICB / GICS / TRBC; mixed scheme).", "SectorOut"),
        "post": post_op("reference", "Upsert a sector entry.", "SectorIn", "SectorOut")
    }));
    paths.insert("/fx".into(), json!({
        "get": list_op("reference", "List FX rate observations.", "FxOut"),
        "post": post_op("reference", "Insert an FX rate observation.", "FxIn", "FxOut")
    }));

    // ── stocks ────────────────────────────────────────────────────────────
    paths.insert("/stocks".into(), json!({
        "get": list_op_p(
            "stocks",
            "List stocks. Filters compose: `country` (US/HK/CN) by market, `symbol` exact match (case-insensitive), `q` substring search across ticker AND localized name. `limit` defaults to 50, max 200.",
            "StockOut",
            vec![
                country_param(),
                locale_param(),
                query_str_param("symbol"),
                query_str_param("q"),
                query_i64_param("limit"),
            ]
        ),
        "post": post_op("stocks", "Create a stock.", "StockIn", "StockOut")
    }));
    paths.insert("/stocks/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("stocks", "Fetch one stock.", "StockOut"),
        "patch": patch_op("stocks", "Update the multi-locale content blob.", "StockPatch", "StockOut"),
        "delete": delete_op("stocks", "Delete a stock.")
    }));
    paths.insert("/stocks/{id}/ohlcv".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List OHLCV rows.", "OhlcvOut"),
        "post": post_op("stocks", "Insert one OHLCV row.", "OhlcvIn", "OhlcvOut")
    }));
    paths.insert("/ohlcv/batch".into(), json!({
        "post": post_op(
            "stocks",
            "Cross-stock bulk upsert of OHLCV bars. Each item must carry stock_id. Upserts on (stock_id, trade_date); all-or-nothing transaction.",
            "OhlcvBatchIn",
            "OhlcvBatchOut"
        )
    }));
    paths.insert("/stocks/{id}/news".into(), json!({
        "parameters": [id_param()],
        "get": list_op(
            "stocks",
            "List news_stock_links for the stock (most recent first).",
            "NewsStockLinkOut"
        )
    }));
    paths.insert("/stocks/{id}/earnings".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List earnings_events for the stock.", "EarningsOut")
    }));
    paths.insert("/stocks/{id}/filings".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List filings for the stock.", "FilingOut")
    }));
    paths.insert("/stocks/{id}/fundamentals".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List fundamentals snapshots for the stock.", "FundamentalsOut")
    }));
    paths.insert("/stocks/{id}/analyst/estimates".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List analyst estimates for the stock.", "AnalystEstimateOut")
    }));
    paths.insert("/stocks/{id}/analyst/ratings".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List analyst ratings for the stock.", "AnalystRatingOut")
    }));
    paths.insert("/stocks/{id}/insider".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List insider transactions for the stock.", "InsiderTxnOut")
    }));
    paths.insert("/stocks/{id}/connect/holdings".into(), json!({
        "parameters": [id_param()],
        "get": list_op(
            "stocks",
            "List Stock Connect holdings snapshots for the stock.",
            "ConnectHoldingsOut"
        )
    }));
    paths.insert("/stocks/{id}/screener-hits".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List screener_hits referencing the stock.", "ScreenerHitOut")
    }));
    paths.insert("/stocks/{id}/recommendations".into(), json!({
        "parameters": [id_param()],
        "get": list_op(
            "stocks",
            "List recommendations targeting the stock.",
            "RecommendationOut"
        )
    }));
    paths.insert("/stocks/{id}/correlation-pairs".into(), json!({
        "parameters": [id_param()],
        "get": list_op(
            "stocks",
            "List correlation_pairs the stock participates in.",
            "CorrelationPairOut"
        )
    }));
    paths.insert("/stocks/{id}/catalysts".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List catalysts attached to the stock.", "CatalystOut")
    }));

    // ── watchlist ─────────────────────────────────────────────────────────
    paths.insert("/watchlist/items".into(), json!({
        "get": list_op_p(
            "watchlists",
            "List stocks on the watchlist.",
            "WatchlistItemOut",
            vec![country_param()]
        ),
        "post": post_op(
            "watchlists",
            "Add a stock to the watchlist (idempotent on stock_id).",
            "WatchlistItemIn",
            "WatchlistItemOut"
        )
    }));
    paths.insert("/watchlist/items/{stock_id}".into(), json!({
        "parameters": [json!({
            "name": "stock_id", "in": "path", "required": true,
            "schema": { "type": "integer", "format": "int64" }
        })],
        "delete": delete_op("watchlists", "Remove a stock from the watchlist.")
    }));
    paths.insert("/watchlist/reports".into(), json!({
        "get": list_op_p(
            "watchlists",
            "List daily / weekly watchlist reports.",
            "WatchlistReportOut",
            vec![locale_param()]
        ),
        "post": post_op(
            "watchlists",
            "Upsert a watchlist report (natural key: kind + period_start).",
            "WatchlistReportIn",
            "WatchlistReportOut"
        )
    }));
    paths.insert("/watchlist/reports/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("watchlists", "Fetch one watchlist report.", "WatchlistReportOut"),
        "delete": delete_op("watchlists", "Delete a watchlist report.")
    }));

    // ── transactions / holdings ───────────────────────────────────────────
    paths.insert("/transactions".into(), json!({
        "get": list_op("transactions", "List transactions.", "TransactionOut"),
        "post": post_op(
            "transactions",
            "Record a transaction (idempotent via `Idempotency-Key` header).",
            "TransactionIn",
            "TransactionOut"
        )
    }));
    paths.insert("/transactions/{id}".into(), json!({
        "parameters": [id_param()],
        "get": get_op("transactions", "Fetch one transaction.", "TransactionOut"),
        "delete": delete_op("transactions", "Delete a transaction.")
    }));
    paths.insert("/holdings".into(), json!({
        "get": {
            "tags": ["holdings"],
            "summary": "Compute open positions from transactions.",
            "parameters": [json!({
                "name": "method", "in": "query",
                "schema": { "type": "string", "enum": ["fifo", "lifo", "average"], "default": "fifo" }
            })],
            "responses": ok_list("HoldingOut")
        }
    }));
    paths.insert("/portfolio/value-series".into(), json!({
        "get": {
            "tags": ["holdings"],
            "summary": "Daily portfolio market value + cost basis over a window.",
            "description": "Derived from `transactions` (FIFO cost basis) and `ohlcv_daily` (latest close on or before the date, carried forward across weekends and holidays). One row per calendar day. Default window is 30 days; capped at 365.",
            "parameters": [json!({
                "name": "days", "in": "query",
                "schema": { "type": "integer", "minimum": 1, "maximum": 365, "default": 30 }
            })],
            "responses": ok_list("DailyValueOut")
        }
    }));

    // ── trade plans ───────────────────────────────────────────────────────
    // Two-tier: `/trade-plans` is the per-user, per-stock header; each plan
    // carries N price levels at `/trade-plans/{id}/levels`. Levels are then
    // accessible / mutable via `/trade-plans/levels/{id}` directly.
    paths.insert("/trade-plans".into(), json!({
        "get": list_op_p(
            "trade-plans",
            "List the caller's trade plans, with optional filters.",
            "TradePlanOut",
            vec![query_i64_param("stock_id"), query_str_param("status")]
        ),
        "post": post_op("trade-plans", "Create a new trade plan.", "TradePlanIn", "TradePlanOut")
    }));
    paths.insert("/trade-plans/{id}".into(), json!({
        "parameters": [id_param()],
        "get": get_op("trade-plans", "Fetch one trade plan.", "TradePlanOut"),
        "patch": patch_op(
            "trade-plans",
            "Update rationale / status (active|closed).",
            "TradePlanPatch",
            "TradePlanOut"
        ),
        "delete": delete_op("trade-plans", "Delete a trade plan; its levels cascade.")
    }));
    paths.insert("/trade-plans/{id}/levels".into(), json!({
        "parameters": [id_param()],
        "get": list_op("trade-plans", "List the levels under one plan.", "TradePlanLevelOut"),
        "post": post_op(
            "trade-plans",
            "Add a new level (kind: buy|stop_loss|take_profit|trim) to the plan.",
            "TradePlanLevelIn",
            "TradePlanLevelOut"
        )
    }));
    paths.insert("/trade-plans/levels/{id}".into(), json!({
        "parameters": [id_param()],
        "patch": patch_op(
            "trade-plans",
            "Update a level (price, size, status). Flipping status=triggered stamps triggered_at.",
            "TradePlanLevelPatch",
            "TradePlanLevelOut"
        ),
        "delete": delete_op("trade-plans", "Delete a single level.")
    }));

    // ── pending orders ────────────────────────────────────────────────────
    // Per-user broker order book — what the user has live (or intends to
    // place) with their broker. Agents can read this to suggest tightening
    // or cancelling existing orders alongside quotes and holdings.
    paths.insert("/pending-orders".into(), json!({
        "get": list_op_p(
            "pending-orders",
            "List the caller's pending limit orders, with optional filters.",
            "PendingOrderOut",
            vec![
                query_i64_param("account_id"),
                query_i64_param("stock_id"),
                query_str_param("status")
            ]
        ),
        "post": post_op(
            "pending-orders",
            "Record a pending order (side: buy|sell; type: limit|stop|stop_limit).",
            "PendingOrderIn",
            "PendingOrderOut"
        )
    }));
    paths.insert("/pending-orders/{id}".into(), json!({
        "parameters": [id_param()],
        "get": get_op("pending-orders", "Fetch one pending order.", "PendingOrderOut"),
        "patch": patch_op(
            "pending-orders",
            "Update the order. Flipping status=filled stamps filled_at; status=cancelled stamps cancelled_at.",
            "PendingOrderPatch",
            "PendingOrderOut"
        ),
        "delete": delete_op("pending-orders", "Hard-delete the order record.")
    }));

    // ── news ──────────────────────────────────────────────────────────────
    paths.insert("/news".into(), json!({
        "get": list_op_p(
            "news",
            "List news items (server merges translations when ?locale=).",
            "NewsOut",
            vec![locale_param()]
        ),
        "post": post_op("news", "Create a news item.", "NewsIn", "NewsOut")
    }));
    // PATCH semantics: partial update, content JSONB-merged at the top
    // level so single-locale fixes don't wipe other locales.
    paths.insert("/news/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("news", "Fetch one news item.", "NewsOut"),
        "patch": {
            "tags": ["news"],
            "summary": "Partial update of a news item.",
            "description": "Every field is optional; absent fields are left untouched. `content` is JSONB-merged at the top level so e.g. `{ \"zh-CN\": { \"title\": \"…\" } }` adds/replaces just that locale and preserves any other locales on the row. To fully replace `content`, delete + re-create.",
            "requestBody": {
                "required": true,
                "content": {
                    "application/json": {
                        "schema": schema_ref("NewsPatch")
                    }
                }
            },
            "responses": ok_inline(schema_ref("NewsOut"))
        },
        "delete": delete_op("news", "Delete a news item.")
    }));
    paths.insert("/news/{id}/stock-links".into(), json!({
        "parameters": [id_param()],
        "get": list_op("news", "List linked stocks.", "NewsStockLinkOut"),
        "post": post_op("news", "Link a stock.", "NewsStockLinkIn", "NewsStockLinkOut")
    }));
    paths.insert("/news/{id}/sector-links".into(), json!({
        "parameters": [id_param()],
        "get": list_op("news", "List linked sectors.", "NewsSectorLinkOut"),
        "post": post_op("news", "Link a sector.", "NewsSectorLinkIn", "NewsSectorLinkOut")
    }));
    paths.insert("/news/{id}/macro-links".into(), json!({
        "parameters": [id_param()],
        "get": list_op("news", "List linked macro indicators.", "NewsMacroLinkOut"),
        "post": post_op(
            "news",
            "Link a macro indicator.",
            "NewsMacroLinkIn",
            "NewsMacroLinkOut"
        )
    }));
    paths.insert("/news/{id}/country-links".into(), json!({
        "parameters": [id_param()],
        "get": list_op("news", "List linked countries.", "NewsCountryLinkOut"),
        "post": post_op("news", "Link a country.", "NewsCountryLinkIn", "NewsCountryLinkOut")
    }));

    // ── calendar (briefs / earnings / macro events / catalysts) ───────────
    paths.insert("/market-briefs".into(), json!({
        "get": list_op_p(
            "calendar",
            "List market briefs (pre/post-market / smart_money_scan).",
            "MarketBriefOut",
            vec![country_param(), locale_param()]
        ),
        "post": post_op(
            "calendar",
            "Upsert a market brief (natural key: country + kind + trade_date).",
            "MarketBriefIn",
            "MarketBriefOut"
        )
    }));
    paths.insert("/market-briefs/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("calendar", "Fetch one market brief.", "MarketBriefOut"),
        "delete": delete_op("calendar", "Delete a market brief.")
    }));
    paths.insert("/earnings".into(), json!({
        "get": list_op_p(
            "calendar",
            "List earnings events.",
            "EarningsOut",
            vec![country_param()]
        ),
        "post": post_op(
            "calendar",
            "Upsert an earnings event (natural key: stock_id + fiscal_year + fiscal_period).",
            "EarningsIn",
            "EarningsOut"
        )
    }));
    paths.insert("/earnings/batch".into(), json!({
        "post": post_op(
            "calendar",
            "Bulk upsert earnings events. All-or-nothing transaction; max 1000 items. Each item upserts on (stock_id, fiscal_year, fiscal_period).",
            "EarningsBatchIn",
            "EarningsBatchOut"
        )
    }));
    paths.insert("/earnings/{id}".into(), json!({
        "parameters": [id_param()],
        "get": get_op("calendar", "Fetch one earnings event.", "EarningsOut"),
        "delete": delete_op("calendar", "Delete an earnings event.")
    }));
    paths.insert("/catalysts".into(), json!({
        "get": list_op_p(
            "calendar",
            "List catalysts.",
            "CatalystOut",
            vec![country_param(), locale_param()]
        ),
        "post": post_op(
            "calendar",
            "Create or refresh a catalyst. Upserts on (user_id, kind, date, stock_id, sector_code, country, source).",
            "CatalystIn",
            "CatalystOut"
        )
    }));
    paths.insert("/catalysts/batch".into(), json!({
        "post": post_op(
            "calendar",
            "Bulk create or refresh catalysts. All-or-nothing transaction; max 1000 items.",
            "CatalystBatchIn",
            "CatalystBatchOut"
        )
    }));
    paths.insert("/catalysts/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("calendar", "Fetch one catalyst.", "CatalystOut"),
        "delete": delete_op("calendar", "Delete a catalyst.")
    }));

    // ── macros ────────────────────────────────────────────────────────────
    paths.insert("/macro/indicators".into(), json!({
        "get": list_op("macros", "List macro indicator codes.", "MacroIndicatorOut"),
        "post": post_op(
            "macros",
            "Upsert a macro indicator.",
            "MacroIndicatorIn",
            "MacroIndicatorOut"
        )
    }));
    paths.insert("/macro/indicators/{code}/observations".into(), json!({
        "parameters": [path_str_param("code")],
        "get": list_op(
            "macros",
            "List observations for an indicator.",
            "MacroObservationOut"
        )
    }));
    paths.insert("/macro/observations".into(), json!({
        "post": post_op(
            "macros",
            "Upsert one observation (natural key: indicator_code + obs_date). Re-POSTing the same key refreshes value/source and stamps revised_at.",
            "MacroObservationIn",
            "MacroObservationOut"
        )
    }));
    paths.insert("/macro/events".into(), json!({
        "get": list_op_p(
            "macros",
            "List discrete macro events (FOMC decision, CPI release, …).",
            "MacroEventOut",
            vec![country_param(), locale_param()]
        ),
        "post": post_op(
            "macros",
            "Upsert a macro event (natural key: indicator_code + event_date).",
            "MacroEventIn",
            "MacroEventOut"
        )
    }));
    paths.insert("/macro/events/batch".into(), json!({
        "post": post_op(
            "macros",
            "Bulk upsert macro events. All-or-nothing transaction; max 1000 items. Each item upserts on (indicator_code, event_date).",
            "MacroEventBatchIn",
            "MacroEventBatchOut"
        )
    }));
    paths.insert("/macro/events/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("macros", "Fetch one macro event.", "MacroEventOut"),
        "delete": delete_op("macros", "Delete a macro event.")
    }));

    // ── analyst / insider / filings / fundamentals / connect ──────────────
    paths.insert("/analyst/estimates".into(), json!({
        "post": post_op(
            "analyst",
            "Insert an analyst estimate.",
            "AnalystEstimateIn",
            "AnalystEstimateOut"
        )
    }));
    paths.insert("/analyst/estimates/batch".into(), json!({
        "post": post_op(
            "analyst",
            "Bulk insert analyst estimates. All-or-nothing transaction; max 1000 items.",
            "AnalystEstimateBatchIn",
            "AnalystEstimateBatchOut"
        )
    }));
    paths.insert("/analyst/estimates/{id}".into(), json!({
        "parameters": [id_param()],
        "delete": delete_op("analyst", "Delete an analyst estimate.")
    }));
    paths.insert("/analyst/ratings".into(), json!({
        "post": post_op(
            "analyst",
            "Insert an analyst rating.",
            "AnalystRatingIn",
            "AnalystRatingOut"
        )
    }));
    paths.insert("/analyst/ratings/batch".into(), json!({
        "post": post_op(
            "analyst",
            "Bulk insert analyst ratings. All-or-nothing transaction; max 1000 items.",
            "AnalystRatingBatchIn",
            "AnalystRatingBatchOut"
        )
    }));
    paths.insert("/analyst/ratings/{id}".into(), json!({
        "parameters": [id_param()],
        "delete": delete_op("analyst", "Delete an analyst rating.")
    }));
    paths.insert("/insider/transactions".into(), json!({
        "post": post_op(
            "insider",
            "Insert an insider transaction.",
            "InsiderTxnIn",
            "InsiderTxnOut"
        )
    }));
    paths.insert("/insider/transactions/batch".into(), json!({
        "post": post_op(
            "insider",
            "Bulk insert insider transactions. All-or-nothing transaction; max 1000 items.",
            "InsiderTxnBatchIn",
            "InsiderTxnBatchOut"
        )
    }));
    paths.insert("/insider/transactions/{id}".into(), json!({
        "parameters": [id_param()],
        "delete": delete_op("insider", "Delete an insider transaction.")
    }));
    paths.insert("/filings".into(), json!({
        "post": post_op("filings", "Create a filing entry.", "FilingIn", "FilingOut")
    }));
    paths.insert("/filings/{id}".into(), json!({
        "parameters": [id_param()],
        "get": get_op("filings", "Fetch one filing.", "FilingOut")
    }));
    paths.insert("/fundamentals".into(), json!({
        "post": post_op(
            "fundamentals",
            "Insert a fundamentals snapshot.",
            "FundamentalsIn",
            "FundamentalsOut"
        )
    }));
    paths.insert("/connect/flow".into(), json!({
        "get": list_op("connect", "List Stock Connect daily flow.", "ConnectFlowOut"),
        "post": post_op(
            "connect",
            "Insert a flow observation.",
            "ConnectFlowIn",
            "ConnectFlowOut"
        )
    }));
    paths.insert("/connect/holdings".into(), json!({
        "post": post_op(
            "connect",
            "Insert a Stock Connect holdings snapshot.",
            "ConnectHoldingsIn",
            "ConnectHoldingsOut"
        )
    }));

    // ── agent outputs: screeners / portfolio-reviews / recs / self-exams /
    //    correlations / universes ─────────────────────────────────────────
    paths.insert("/screener-runs".into(), json!({
        "get": list_op_p(
            "agent-outputs",
            "List screener runs.",
            "ScreenerRunOut",
            vec![locale_param()]
        ),
        "post": post_op(
            "agent-outputs",
            "Upsert a screener run (natural key: name + kind + run_date).",
            "ScreenerRunIn",
            "ScreenerRunOut"
        )
    }));
    paths.insert("/screener-runs/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("agent-outputs", "Fetch one screener run.", "ScreenerRunOut"),
        "delete": delete_op(
            "agent-outputs",
            "Delete a screener run and cascade-delete its hits in one transaction. Use to clean up superseded runs (different criteria, same name/kind/run_date)."
        )
    }));
    paths.insert("/screener-runs/{id}/hits".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": list_op("agent-outputs", "List hits for a run.", "ScreenerHitOut"),
        "post": post_op(
            "agent-outputs",
            "Insert a hit for a run.",
            "ScreenerHitIn",
            "ScreenerHitOut"
        )
    }));

    paths.insert("/portfolio-reviews".into(), json!({
        "get": list_op_p(
            "agent-outputs",
            "List portfolio reviews (weekly / monthly / quarterly).",
            "PortfolioReviewOut",
            vec![locale_param()]
        ),
        "post": post_op(
            "agent-outputs",
            "Upsert a portfolio review (natural key: kind + period_start).",
            "PortfolioReviewIn",
            "PortfolioReviewOut"
        )
    }));
    paths.insert("/portfolio-reviews/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op(
            "agent-outputs",
            "Fetch one portfolio review.",
            "PortfolioReviewOut"
        ),
        "delete": delete_op("agent-outputs", "Delete a portfolio review.")
    }));

    paths.insert("/recommendations".into(), json!({
        "get": list_op_p(
            "agent-outputs",
            "List recommendations.",
            "RecommendationOut",
            vec![query_str_param("status"), query_i64_param("stock_id"), locale_param()]
        ),
        "post": post_op(
            "agent-outputs",
            "Create a recommendation (initial status 'open').",
            "RecommendationIn",
            "RecommendationOut"
        )
    }));
    paths.insert("/recommendations/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("agent-outputs", "Fetch one recommendation.", "RecommendationOut"),
        "delete": delete_op("agent-outputs", "Delete a recommendation.")
    }));
    paths.insert("/recommendations/{id}/close".into(), json!({
        "parameters": [id_param()],
        "post": post_op(
            "agent-outputs",
            "Close-out a recommendation (status, outcome_md, pnl_pct, closed_at).",
            "RecommendationClosePatch",
            "RecommendationOut"
        )
    }));

    paths.insert("/self-exams".into(), json!({
        "get": list_op_p(
            "agent-outputs",
            "List self-exams (agent reflecting on past recommendations).",
            "SelfExamOut",
            vec![query_str_param("kind"), locale_param()]
        ),
        "post": post_op(
            "agent-outputs",
            "Upsert a self-exam (natural key: kind + period_start).",
            "SelfExamIn",
            "SelfExamOut"
        )
    }));
    paths.insert("/self-exams/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("agent-outputs", "Fetch one self-exam.", "SelfExamOut"),
        "delete": delete_op("agent-outputs", "Delete a self-exam.")
    }));

    paths.insert("/universes".into(), json!({
        "get": list_op(
            "agent-outputs",
            "List correlation universes (named stock sets).",
            "UniverseOut"
        ),
        "post": post_op(
            "agent-outputs",
            "Upsert a universe by name.",
            "UniverseIn",
            "UniverseOut"
        )
    }));
    paths.insert("/universes/{id}".into(), json!({
        "parameters": [id_param()],
        "get": get_op("agent-outputs", "Fetch one universe.", "UniverseOut"),
        "delete": delete_op(
            "agent-outputs",
            "Delete a universe definition. Returns 409 if any correlation_run still references it — delete the dependent runs first."
        )
    }));
    paths.insert("/correlation-runs".into(), json!({
        "get": list_op_p(
            "agent-outputs",
            "List correlation runs.",
            "CorrelationRunOut",
            vec![locale_param()]
        ),
        "post": post_op(
            "agent-outputs",
            "Create a correlation run.",
            "CorrelationRunIn",
            "CorrelationRunOut"
        )
    }));
    paths.insert("/correlation-runs/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("agent-outputs", "Fetch one correlation run.", "CorrelationRunOut"),
        "delete": delete_op(
            "agent-outputs",
            "Delete a correlation run and cascade-delete its pair rows in one transaction. Use to clean up superseded runs or to free a universe before deleting it."
        )
    }));
    paths.insert("/correlation-runs/{id}/pairs".into(), json!({
        "parameters": [id_param()],
        "get": list_op(
            "agent-outputs",
            "List pair correlations for a run.",
            "CorrelationPairOut"
        ),
        "post": post_op(
            "agent-outputs",
            "Insert a pair correlation.",
            "CorrelationPairIn",
            "CorrelationPairOut"
        )
    }));

    // ── audit ─────────────────────────────────────────────────────────────
    // AuditEntryOut is defined inline in handlers/audit.rs without ToSchema,
    // so we inline the response shape here.
    // ── Unread state ────────────────────────────────────────────────────
    // The 10 event-like entity GETs (news/{id}, market-briefs/{id}, …) all
    // mark the item read as a side effect; those paths are documented
    // alongside their other operations above. These two endpoints are
    // the dedicated unread surface.
    paths.insert("/unread/counts".into(), json!({
        "get": {
            "tags": ["unread"],
            "summary": "Per-entity unread counts for the calling user. Drives the sidebar badge.",
            "description": "Returns `{ entity_type: count }` over the full set of tracked entity kinds. Counts skip items created before the user's `users.created_at` so new accounts start from a clean slate. Admin / anonymous callers get zero across the board.",
            "responses": ok_inline(json!({
                "type": "object",
                "description": "Map of canonical entity_type → unread row count. Always contains every tracked kind (zero-filled).",
                "additionalProperties": { "type": "integer", "format": "int64", "minimum": 0 },
                "example": {
                    "news": 3,
                    "market_brief": 1,
                    "macro_event": 0,
                    "earnings_event": 0,
                    "catalyst": 5,
                    "screener_run": 0,
                    "recommendation": 2,
                    "portfolio_review": 0,
                    "correlation_run": 0,
                    "self_exam": 0
                }
            }))
        }
    }));
    paths.insert("/reads/mark-all/{kind}".into(), json!({
        "post": {
            "tags": ["unread"],
            "summary": "Bulk mark every visible entity of this kind as read for the caller.",
            "description": "Idempotent: already-read items are skipped. Returns the number of fresh inserts so the caller can show a confirmation (or skip the flash when nothing changed). Used by the per-page \"mark all read\" button.",
            "parameters": [
                {
                    "name": "kind",
                    "in": "path",
                    "required": true,
                    "schema": {
                        "type": "string",
                        "enum": [
                            "news", "market_brief", "macro_event", "earnings_event",
                            "catalyst", "screener_run", "recommendation",
                            "portfolio_review", "correlation_run", "self_exam"
                        ]
                    }
                }
            ],
            "responses": ok_inline(json!({
                "type": "object",
                "required": ["marked"],
                "properties": {
                    "marked": {
                        "type": "integer",
                        "format": "int64",
                        "minimum": 0,
                        "description": "Number of fresh inserts; 0 if everything was already read."
                    }
                }
            }))
        }
    }));
    paths.insert("/reads/{kind}/{id}".into(), json!({
        "delete": {
            "tags": ["unread"],
            "summary": "Flip an item back to unread for the calling user.",
            "description": "Removes the row from `user_reads`. Idempotent — deleting an already-unread item still returns 204. `kind` must be one of the canonical entity_type strings (`news`, `market_brief`, `macro_event`, `earnings_event`, `catalyst`, `screener_run`, `recommendation`, `portfolio_review`, `correlation_run`, `self_exam`); unknown values return 400.",
            "parameters": [
                {
                    "name": "kind",
                    "in": "path",
                    "required": true,
                    "schema": {
                        "type": "string",
                        "enum": [
                            "news", "market_brief", "macro_event", "earnings_event",
                            "catalyst", "screener_run", "recommendation",
                            "portfolio_review", "correlation_run", "self_exam"
                        ]
                    }
                },
                id_param()
            ],
            "responses": {
                "204": { "description": "Marked unread (or was already unread)." },
                "400": { "description": "Unknown entity kind." },
                "403": { "description": "Caller has no user_id (admin / anonymous)." }
            }
        }
    }));

    paths.insert("/audit".into(), json!({
        "get": {
            "tags": ["audit"],
            "summary": "List audit log entries (server-side write trail).",
            "responses": ok_inline(json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["id", "entity_type", "entity_id", "action", "actor_kind", "actor_label", "request_id", "created_at"],
                    "properties": {
                        "id": { "type": "integer", "format": "int64" },
                        "entity_type": { "type": "string" },
                        "entity_id": { "type": "string" },
                        "action": { "type": "string" },
                        "actor_kind": { "type": "string" },
                        "actor_id": { "type": "integer", "format": "int64", "nullable": true },
                        "actor_label": { "type": "string" },
                        "before": { "type": "string", "nullable": true },
                        "after": { "type": "string", "nullable": true },
                        "request_id": { "type": "string" },
                        "created_at": { "type": "string" }
                    }
                }
            }))
        }
    }));

    Value::Object(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every operation in the spec must declare at least one response, and
    /// `$ref` schema references must point at components that actually exist.
    #[test]
    fn spec_is_self_consistent() {
        let spec = spec();
        let schemas = spec
            .pointer("/components/schemas")
            .and_then(|v| v.as_object())
            .expect("components.schemas present");
        let paths = spec
            .pointer("/paths")
            .and_then(|v| v.as_object())
            .expect("paths present");

        assert_eq!(spec.pointer("/info/title").and_then(|v| v.as_str()), Some("Plutus API"));
        // Tighter sanity floors so the build fails loudly if the spec ever
        // shrinks unexpectedly (e.g. a refactor drops a chunk of routes).
        // Current snapshot after trade-plans + pending-orders + admin
        // brokers/tokens/countries sync: ~93 paths, ~95 schemas.
        assert!(paths.len() >= 80, "too few paths: {}", paths.len());
        assert!(schemas.len() >= 80, "too few schemas: {}", schemas.len());

        // Walk every $ref in the doc and assert each schema target exists.
        fn walk(v: &Value, schemas: &Map<String, Value>) {
            match v {
                Value::Object(map) => {
                    for (k, child) in map {
                        if k == "$ref" {
                            if let Some(s) = child.as_str() {
                                if let Some(name) = s.strip_prefix("#/components/schemas/") {
                                    assert!(
                                        schemas.contains_key(name),
                                        "dangling schema $ref: {name}"
                                    );
                                }
                            }
                        } else {
                            walk(child, schemas);
                        }
                    }
                }
                Value::Array(items) => items.iter().for_each(|i| walk(i, schemas)),
                _ => {}
            }
        }
        walk(&spec, schemas);
    }

    /// Sanity-check that we register every In/Out DTO we expect.
    #[test]
    fn key_schemas_present() {
        let spec = spec();
        let schemas = spec
            .pointer("/components/schemas")
            .and_then(|v| v.as_object())
            .expect("components.schemas present");
        for name in [
            "StockIn", "StockOut", "StockPatch",
            "MarketBriefIn", "MarketBriefOut",
            "RecommendationIn", "RecommendationOut", "RecommendationClosePatch",
            "TokenIn", "TokenOut", "TokenCreatedOut",
            "AuditEntry", // intentionally missing — inlined in /audit
        ] {
            if name == "AuditEntry" {
                assert!(!schemas.contains_key(name), "AuditEntry should not be a component");
            } else {
                assert!(schemas.contains_key(name), "missing schema: {name}");
            }
        }
    }

    /// Drift detector: every `.route(...)` in lib.rs must have a matching
    /// entry in `paths()`, and every entry in `paths()` must correspond
    /// to a real route. The check is path + HTTP-method tuple.
    ///
    /// Catches the common drift class: "I added a handler but forgot
    /// to update the OpenAPI spec." Run by `cargo test -p plutus-api`.
    ///
    /// Parser: we read `lib.rs` as a literal string at compile time and
    /// scan for `.route("<path>", <methods-blob>)` blocks. The path is
    /// the first string literal; the methods are any of
    /// {get, post, patch, put, delete} called as a function inside the
    /// methods blob. Axum's `:id` style is normalized to OpenAPI's
    /// `{id}` style for comparison.
    ///
    /// Exempt paths: the outer router's `/` (mounted outside `/api/v1`
    /// and intentionally undocumented).
    #[test]
    fn routes_match_spec() {
        use std::collections::{BTreeMap, BTreeSet};

        const LIB_SRC: &str = include_str!("lib.rs");
        const EXEMPT: &[&str] = &[
            // Lives on the outer router, not under /api/v1 — not part of
            // the documented API surface.
            "/",
        ];

        /// Parse lib.rs for `.route("path", ...method(...).method(...)...)`.
        /// Returns path → set of upper-case method names. Multiple
        /// `.route("/x", ...)` calls for the same path merge their methods.
        fn parse_axum_routes(src: &str) -> BTreeMap<String, BTreeSet<String>> {
            let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            let bytes = src.as_bytes();
            let needle = b".route(";
            let mut i = 0;
            while i + needle.len() <= bytes.len() {
                let Some(rel) = bytes[i..].windows(needle.len()).position(|w| w == needle) else {
                    break;
                };
                let after_open = i + rel + needle.len();
                let mut p = after_open;
                // Skip whitespace, then expect the opening quote of the path literal.
                while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                    p += 1;
                }
                if p >= bytes.len() || bytes[p] != b'"' {
                    i = after_open;
                    continue;
                }
                p += 1;
                let path_start = p;
                while p < bytes.len() && bytes[p] != b'"' {
                    p += 1;
                }
                if p >= bytes.len() {
                    break;
                }
                let raw_path = std::str::from_utf8(&bytes[path_start..p]).unwrap();
                // Normalize axum's `:id` to OpenAPI's `{id}`.
                let path: String = raw_path
                    .split('/')
                    .map(|seg| {
                        if let Some(name) = seg.strip_prefix(':') {
                            format!("{{{name}}}")
                        } else {
                            seg.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("/");
                p += 1; // past the closing quote of the path
                // Scan to the matching ')' for the .route( call, tracking
                // paren depth so nested ( ) inside method calls are skipped.
                let methods_start = p;
                let mut depth: i32 = 1;
                while p < bytes.len() && depth > 0 {
                    match bytes[p] {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        p += 1;
                    }
                }
                let methods_blob = std::str::from_utf8(&bytes[methods_start..p]).unwrap();

                // Pull verbs that are called as functions: `verb(`. Guard
                // against matching `foo_get(` or `route_get(` by checking
                // the preceding byte isn't alphanumeric or underscore.
                let mut methods: BTreeSet<String> = BTreeSet::new();
                for verb in ["get", "post", "patch", "put", "delete"] {
                    let pat = format!("{verb}(");
                    let mut start = 0;
                    while let Some(found) = methods_blob[start..].find(&pat) {
                        let abs = start + found;
                        let preceding = if abs == 0 {
                            b' '
                        } else {
                            methods_blob.as_bytes()[abs - 1]
                        };
                        if !preceding.is_ascii_alphanumeric() && preceding != b'_' {
                            methods.insert(verb.to_uppercase());
                        }
                        start = abs + pat.len();
                    }
                }
                out.entry(path).or_default().extend(methods);
                i = p;
            }
            out
        }

        let mut axum_routes = parse_axum_routes(LIB_SRC);
        for ex in EXEMPT {
            axum_routes.remove(*ex);
        }
        assert!(
            !axum_routes.is_empty(),
            "axum route parser found nothing — did lib.rs's `.route(...)` formatting change?"
        );

        let spec = spec();
        let spec_paths = spec
            .pointer("/paths")
            .and_then(|v| v.as_object())
            .expect("paths present");

        // path → set of methods (uppercase) for the spec side.
        let mut spec_routes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (path, ops_value) in spec_paths {
            let Some(ops) = ops_value.as_object() else { continue };
            let methods: BTreeSet<String> = ops
                .keys()
                .filter(|k| matches!(k.as_str(), "get" | "post" | "patch" | "put" | "delete"))
                .map(|k| k.to_uppercase())
                .collect();
            if !methods.is_empty() {
                spec_routes.insert(path.clone(), methods);
            }
        }

        let mut errors: Vec<String> = Vec::new();

        // Routes present in axum but not in the spec (or with extra methods
        // the spec doesn't document).
        for (path, methods) in &axum_routes {
            match spec_routes.get(path) {
                None => errors.push(format!(
                    "path {path:?} is wired in lib.rs but missing from openapi.rs::paths() entirely (methods {methods:?})"
                )),
                Some(spec_methods) => {
                    let missing: Vec<_> = methods.difference(spec_methods).collect();
                    if !missing.is_empty() {
                        errors.push(format!(
                            "path {path:?}: lib.rs has methods {missing:?} but the spec doesn't"
                        ));
                    }
                }
            }
        }

        // Routes present in the spec but with no axum route to back them.
        for (path, methods) in &spec_routes {
            match axum_routes.get(path) {
                None => errors.push(format!(
                    "path {path:?} is documented in openapi.rs::paths() but no axum route handles it (methods {methods:?})"
                )),
                Some(actual_methods) => {
                    let extra: Vec<_> = methods.difference(actual_methods).collect();
                    if !extra.is_empty() {
                        errors.push(format!(
                            "path {path:?}: spec documents methods {extra:?} that lib.rs doesn't expose"
                        ));
                    }
                }
            }
        }

        assert!(
            errors.is_empty(),
            "OpenAPI spec drift from actual routes:\n  - {}",
            errors.join("\n  - ")
        );
    }
}
