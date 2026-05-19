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
    analyst::{AnalystEstimateIn, AnalystEstimateOut, AnalystRatingIn, AnalystRatingOut},
    broker::BrokerOut,
    catalyst::{CatalystIn, CatalystOut},
    connect::{ConnectFlowIn, ConnectFlowOut, ConnectHoldingsIn, ConnectHoldingsOut},
    correlation::{
        CorrelationPairIn, CorrelationPairOut, CorrelationRunIn, CorrelationRunOut, UniverseIn,
        UniverseOut,
    },
    earnings::{EarningsIn, EarningsOut},
    filing::{FilingIn, FilingOut},
    fundamentals::{FundamentalsIn, FundamentalsOut},
    fx::{FxIn, FxOut},
    holding::HoldingOut,
    insider::{InsiderTxnIn, InsiderTxnOut},
    macro_event::{MacroEventIn, MacroEventOut},
    macros::{MacroIndicatorIn, MacroIndicatorOut, MacroObservationIn, MacroObservationOut},
    market::MarketOut,
    market_brief::{MarketBriefIn, MarketBriefOut},
    news::{
        NewsCountryLinkIn, NewsCountryLinkOut, NewsIn, NewsMacroLinkIn, NewsMacroLinkOut, NewsOut,
        NewsSectorLinkIn, NewsSectorLinkOut, NewsStockLinkIn, NewsStockLinkOut, NewsTranslationIn,
        NewsTranslationOut,
    },
    ohlcv::{OhlcvIn, OhlcvOut},
    portfolio_review::{PortfolioReviewIn, PortfolioReviewOut},
    recommendation::{RecommendationClosePatch, RecommendationIn, RecommendationOut},
    screener::{ScreenerHitIn, ScreenerHitOut, ScreenerRunIn, ScreenerRunOut},
    sector::{SectorIn, SectorOut},
    self_exam::{SelfExamIn, SelfExamOut},
    stock::{StockIn, StockOut, StockPatch, StockTranslationIn, StockTranslationOut},
    token::{TokenCreatedOut, TokenIn, TokenOut},
    transaction::{TransactionIn, TransactionOut},
    watchlist::{WatchlistItemIn, WatchlistItemOut},
    watchlist_report::{WatchlistReportIn, WatchlistReportOut},
};

/// Marker type for `utoipa` derive. The struct itself is never instantiated;
/// only its `OpenApi` impl is used, which exposes the registered component
/// schemas for every DTO listed below.
#[derive(OpenApi)]
#[openapi(components(schemas(
    AccountIn, AccountOut,
    AnalystEstimateIn, AnalystEstimateOut,
    AnalystRatingIn, AnalystRatingOut,
    BrokerOut,
    CatalystIn, CatalystOut,
    ConnectFlowIn, ConnectFlowOut,
    ConnectHoldingsIn, ConnectHoldingsOut,
    CorrelationPairIn, CorrelationPairOut,
    CorrelationRunIn, CorrelationRunOut,
    UniverseIn, UniverseOut,
    EarningsIn, EarningsOut,
    FilingIn, FilingOut,
    FundamentalsIn, FundamentalsOut,
    FxIn, FxOut,
    HoldingOut,
    InsiderTxnIn, InsiderTxnOut,
    MacroEventIn, MacroEventOut,
    MacroIndicatorIn, MacroIndicatorOut,
    MacroObservationIn, MacroObservationOut,
    MarketOut,
    MarketBriefIn, MarketBriefOut,
    NewsIn, NewsOut,
    NewsStockLinkIn, NewsStockLinkOut,
    NewsSectorLinkIn, NewsSectorLinkOut,
    NewsMacroLinkIn, NewsMacroLinkOut,
    NewsCountryLinkIn, NewsCountryLinkOut,
    NewsTranslationIn, NewsTranslationOut,
    OhlcvIn, OhlcvOut,
    PortfolioReviewIn, PortfolioReviewOut,
    RecommendationIn, RecommendationOut, RecommendationClosePatch,
    ScreenerHitIn, ScreenerHitOut,
    ScreenerRunIn, ScreenerRunOut,
    SectorIn, SectorOut,
    SelfExamIn, SelfExamOut,
    StockIn, StockOut, StockPatch,
    StockTranslationIn, StockTranslationOut,
    TokenIn, TokenOut, TokenCreatedOut,
    TransactionIn, TransactionOut,
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
            "description": "Personal investment data store. The hermes AI agent writes data via this API; the web UI is the human-side viewer.\n\nAll routes are mounted under `/api/v1`. Auth is optional by default (`PLUTUS_API_REQUIRE_AUTH=false`) — flip the env to require either a session cookie (`plutus_session`) or a bearer token on every call."
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
        { "name": "auth", "description": "Master-password login + session cookie." },
        { "name": "tokens", "description": "Long-lived bearer tokens for the agent." },
        { "name": "stocks", "description": "Tradable instruments + metadata + translations." },
        { "name": "watchlists", "description": "The user's watchlist — a flat list of stocks plus daily / weekly reports." },
        { "name": "transactions", "description": "Trade ledger; holdings are derived from this." },
        { "name": "holdings", "description": "Derived open positions per cost basis." },
        { "name": "news", "description": "Articles + per-entity link tables + translations." },
        { "name": "calendar", "description": "Briefs, earnings, macro events, catalysts." },
        { "name": "macros", "description": "Macro indicators + observations + events." },
        { "name": "analyst", "description": "External estimates + ratings." },
        { "name": "insider", "description": "Insider transactions." },
        { "name": "filings", "description": "SEC / HKEX / CSRC filings." },
        { "name": "fundamentals", "description": "Per-period fundamentals snapshots." },
        { "name": "connect", "description": "HK Stock Connect flow + holdings." },
        { "name": "agent-outputs", "description": "Screener runs, recommendations, reviews, self-exams, correlations." },
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
fn put_op(tag: &str, summary: &str, body_schema: &str, response_schema: &str) -> Value {
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
            "summary": "Verify master password, set session cookie.",
            "requestBody": {
                "required": true,
                "content": { "application/json": { "schema": {
                    "type": "object",
                    "required": ["password"],
                    "properties": { "password": { "type": "string" } }
                }}}
            },
            "responses": {
                "200": {
                    "description": "Session cookie set; `Set-Cookie: plutus_session=…`.",
                    "content": { "application/json": { "schema": {
                        "type": "object",
                        "required": ["ok"],
                        "properties": { "ok": { "type": "boolean" } }
                    }}}
                },
                "401": { "description": "Wrong password." }
            }
        }
    }));
    paths.insert("/auth/logout".into(), json!({
        "post": {
            "tags": ["auth"],
            "summary": "Clear session cookie.",
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
            "responses": {
                "200": {
                    "description": "Actor info.",
                    "content": { "application/json": { "schema": {
                        "type": "object",
                        "required": ["kind", "label"],
                        "properties": {
                            "kind": { "type": "string", "enum": ["web", "api_token", "anonymous", "system"] },
                            "label": { "type": "string" },
                            "token_id": { "type": "integer", "format": "int64", "nullable": true }
                        }
                    }}}
                }
            }
        }
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
        "get": get_op("reference", "Fetch one account.", "AccountOut")
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
            "List stocks (filter by country via stock.market_code).",
            "StockOut",
            vec![country_param()]
        ),
        "post": post_op("stocks", "Create a stock.", "StockIn", "StockOut")
    }));
    paths.insert("/stocks/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("stocks", "Fetch one stock.", "StockOut"),
        "patch": patch_op("stocks", "Update mutable stock fields.", "StockPatch", "StockOut"),
        "delete": delete_op("stocks", "Delete a stock.")
    }));
    paths.insert("/stocks/{id}/translations".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List all locale translations for a stock.", "StockTranslationOut")
    }));
    paths.insert("/stocks/{id}/translations/{locale}".into(), json!({
        "parameters": [id_param(), path_str_param("locale")],
        "put": put_op(
            "stocks",
            "Upsert a single-locale translation (name + description_md).",
            "StockTranslationIn",
            "StockTranslationOut"
        )
    }));
    paths.insert("/stocks/{id}/ohlcv".into(), json!({
        "parameters": [id_param()],
        "get": list_op("stocks", "List OHLCV rows.", "OhlcvOut"),
        "post": post_op("stocks", "Insert one OHLCV row.", "OhlcvIn", "OhlcvOut")
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
    paths.insert("/news/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": get_op("news", "Fetch one news item.", "NewsOut"),
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
    paths.insert("/news/{id}/translations".into(), json!({
        "parameters": [id_param()],
        "get": list_op(
            "news",
            "List all locale translations for a news item.",
            "NewsTranslationOut"
        )
    }));
    paths.insert("/news/{id}/translations/{locale}".into(), json!({
        "parameters": [id_param(), path_str_param("locale")],
        "put": put_op(
            "news",
            "Upsert a single-locale translation (title / summary / content_md / agent_summary_md).",
            "NewsTranslationIn",
            "NewsTranslationOut"
        )
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
        "post": post_op("calendar", "Create a catalyst.", "CatalystIn", "CatalystOut")
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
            "Insert one observation.",
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
    paths.insert("/analyst/ratings".into(), json!({
        "post": post_op(
            "analyst",
            "Insert an analyst rating.",
            "AnalystRatingIn",
            "AnalystRatingOut"
        )
    }));
    paths.insert("/insider/transactions".into(), json!({
        "post": post_op(
            "insider",
            "Insert an insider transaction.",
            "InsiderTxnIn",
            "InsiderTxnOut"
        )
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
        "get": get_op("agent-outputs", "Fetch one screener run.", "ScreenerRunOut")
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
        "get": get_op("agent-outputs", "Fetch one universe.", "UniverseOut")
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
        "get": get_op("agent-outputs", "Fetch one correlation run.", "CorrelationRunOut")
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
        // Current snapshot: ~81 paths, ~80 schemas.
        assert!(paths.len() >= 60, "too few paths: {}", paths.len());
        assert!(schemas.len() >= 60, "too few schemas: {}", schemas.len());

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
}
