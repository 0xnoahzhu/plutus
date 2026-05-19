//! OpenAPI spec assembly. Enumerates every route in `lib.rs` so the agent
//! has a complete contract to work against. Schemas are intentionally loose
//! (`object` with free-form additionalProperties) — the canonical shapes
//! live in `dto/*.rs` and we don't want this file to grow into a duplicate.

use serde_json::{json, Value};

pub fn spec() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "plutus API",
            "version": "0.1.0",
            "description": "Personal investment data store. The hermes AI agent writes data via this API; the web UI is the human-side viewer.\n\nAll routes are mounted under `/api/v1`. Auth is optional by default (`PLUTUS_API_REQUIRE_AUTH=false`) — flip the env to require either a session cookie (`plutus_session`) or a bearer token on every call."
        },
        "servers": [{ "url": "/api/v1" }],
        "tags": tags(),
        "paths": paths(),
        "components": {
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
        { "name": "watchlists", "description": "User-curated cross-market themed groups." },
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

// Helpers to keep the path table compact.
fn op(tag: &str, summary: &str) -> Value {
    json!({ "tags": [tag], "summary": summary, "responses": { "200": { "description": "OK" } } })
}
fn op_with_body(tag: &str, summary: &str, body_ref: &str) -> Value {
    json!({
        "tags": [tag],
        "summary": summary,
        "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": body_ref } } } },
        "responses": { "200": { "description": "OK" } }
    })
}
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

fn paths() -> Value {
    let mut paths = serde_json::Map::new();

    // ── meta ──────────────────────────────────────────────────────────────
    paths.insert("/healthz".into(), json!({
        "get": op("meta", "Liveness probe — returns plain text 'ok'.")
    }));
    paths.insert("/openapi.json".into(), json!({
        "get": op("meta", "This OpenAPI document.")
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
                        "type": "object", "properties": { "ok": { "type": "boolean" } }
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
            "responses": { "200": { "description": "Cookie cleared." } }
        }
    }));
    paths.insert("/auth/me".into(), json!({
        "get": {
            "tags": ["auth"],
            "summary": "Identity of the current caller.",
            "responses": { "200": {
                "description": "Actor info.",
                "content": { "application/json": { "schema": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "enum": ["web", "api_token", "anonymous", "system"] },
                        "label": { "type": "string" },
                        "token_id": { "type": "integer", "nullable": true }
                    }
                }}}
            }}
        }
    }));

    // ── tokens (web only) ─────────────────────────────────────────────────
    paths.insert("/tokens".into(), json!({
        "get": op("tokens", "List API tokens (cookie-only)."),
        "post": op("tokens", "Mint a new bearer token; full secret shown once.")
    }));
    paths.insert("/tokens/{id}".into(), json!({
        "parameters": [id_param()],
        "delete": op("tokens", "Revoke a token.")
    }));

    // ── reference ─────────────────────────────────────────────────────────
    paths.insert("/markets".into(), json!({ "get": op("reference", "List markets (MIC codes, timezones, lot sizes).") }));
    paths.insert("/brokers".into(), json!({ "get": op("reference", "List broker entries.") }));
    paths.insert("/accounts".into(), json!({
        "get": op("reference", "List accounts."),
        "post": op("reference", "Create an account.")
    }));
    paths.insert("/accounts/{id}".into(), json!({
        "parameters": [id_param()],
        "get": op("reference", "Fetch one account.")
    }));
    paths.insert("/sectors".into(), json!({
        "get": op("reference", "List sector codes (ICB / GICS / TRBC; mixed scheme)."),
        "post": op("reference", "Upsert a sector entry.")
    }));
    paths.insert("/fx".into(), json!({
        "get": op("reference", "List FX rate observations."),
        "post": op("reference", "Insert an FX rate observation.")
    }));

    // ── stocks ────────────────────────────────────────────────────────────
    paths.insert("/stocks".into(), json!({
        "get": op("stocks", "List stocks (filter by country via stock.market_code)."),
        "post": op("stocks", "Create a stock.")
    }));
    paths.insert("/stocks/{id}".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "Fetch one stock."),
        "patch": op("stocks", "Update mutable stock fields."),
        "delete": op("stocks", "Delete a stock.")
    }));
    paths.insert("/stocks/{id}/translations".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List all locale translations for a stock.")
    }));
    paths.insert("/stocks/{id}/translations/{locale}".into(), json!({
        "parameters": [id_param(), path_str_param("locale")],
        "put": op("stocks", "Upsert a single-locale translation (name + description_md).")
    }));
    paths.insert("/stocks/{id}/ohlcv".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List OHLCV rows."),
        "post": op("stocks", "Insert one OHLCV row.")
    }));
    paths.insert("/stocks/{id}/news".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List news_stock_links for the stock (most recent first).")
    }));
    paths.insert("/stocks/{id}/earnings".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List earnings_events for the stock.")
    }));
    paths.insert("/stocks/{id}/filings".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List filings for the stock.")
    }));
    paths.insert("/stocks/{id}/fundamentals".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List fundamentals snapshots for the stock.")
    }));
    paths.insert("/stocks/{id}/analyst/estimates".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List analyst estimates for the stock.")
    }));
    paths.insert("/stocks/{id}/analyst/ratings".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List analyst ratings for the stock.")
    }));
    paths.insert("/stocks/{id}/insider".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List insider transactions for the stock.")
    }));
    paths.insert("/stocks/{id}/connect/holdings".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List Stock Connect holdings snapshots for the stock.")
    }));
    paths.insert("/stocks/{id}/screener-hits".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List screener_hits referencing the stock.")
    }));
    paths.insert("/stocks/{id}/recommendations".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List recommendations targeting the stock.")
    }));
    paths.insert("/stocks/{id}/correlation-pairs".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List correlation_pairs the stock participates in.")
    }));
    paths.insert("/stocks/{id}/catalysts".into(), json!({
        "parameters": [id_param()],
        "get": op("stocks", "List catalysts attached to the stock.")
    }));

    // ── watchlists ────────────────────────────────────────────────────────
    paths.insert("/watchlists".into(), json!({
        "get": op("watchlists", "List watchlists."),
        "post": op("watchlists", "Create a watchlist.")
    }));
    paths.insert("/watchlists/stocks".into(), json!({
        "get": op("watchlists", "Cross-watchlist stock view (de-duplicated, with `watchlist_ids`).")
    }));
    paths.insert("/watchlists/{id}".into(), json!({
        "parameters": [id_param()],
        "get": op("watchlists", "Fetch one watchlist."),
        "patch": op("watchlists", "Update watchlist metadata."),
        "delete": op("watchlists", "Delete a watchlist.")
    }));
    paths.insert("/watchlists/{id}/items".into(), json!({
        "parameters": [id_param()],
        "get": op("watchlists", "List items."),
        "post": op("watchlists", "Add a stock to a watchlist.")
    }));
    paths.insert("/watchlists/{id}/items/{stock_id}".into(), json!({
        "parameters": [
            id_param(),
            json!({ "name": "stock_id", "in": "path", "required": true, "schema": { "type": "integer", "format": "int64" } })
        ],
        "delete": op("watchlists", "Remove a stock from a watchlist.")
    }));
    paths.insert("/watchlists/{id}/reports".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("watchlists", "List daily / weekly reports for a watchlist.")
    }));
    paths.insert("/watchlist-reports".into(), json!({
        "get": op("watchlists", "List watchlist reports across all groups."),
        "post": op("watchlists", "Upsert a watchlist report (natural key: watchlist_id + kind + period_start).")
    }));
    paths.insert("/watchlist-reports/{id}".into(), json!({
        "parameters": [id_param()],
        "get": op("watchlists", "Fetch one watchlist report."),
        "delete": op("watchlists", "Delete a watchlist report.")
    }));

    // ── transactions / holdings ───────────────────────────────────────────
    paths.insert("/transactions".into(), json!({
        "get": op("transactions", "List transactions."),
        "post": op("transactions", "Record a transaction (idempotent via `Idempotency-Key` header).")
    }));
    paths.insert("/transactions/{id}".into(), json!({
        "parameters": [id_param()],
        "get": op("transactions", "Fetch one transaction."),
        "delete": op("transactions", "Delete a transaction.")
    }));
    paths.insert("/holdings".into(), json!({
        "get": {
            "tags": ["holdings"],
            "summary": "Compute open positions from transactions.",
            "parameters": [json!({
                "name": "method", "in": "query",
                "schema": { "type": "string", "enum": ["fifo", "lifo", "average"], "default": "fifo" }
            })],
            "responses": { "200": { "description": "OK" } }
        }
    }));

    // ── news ──────────────────────────────────────────────────────────────
    paths.insert("/news".into(), json!({
        "get": {
            "tags": ["news"],
            "summary": "List news items (server merges translations when ?locale=).",
            "parameters": [locale_param()],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("news", "Create a news item.")
    }));
    paths.insert("/news/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("news", "Fetch one news item."),
        "delete": op("news", "Delete a news item.")
    }));
    paths.insert("/news/{id}/stock-links".into(), json!({
        "parameters": [id_param()],
        "get": op("news", "List linked stocks."),
        "post": op("news", "Link a stock.")
    }));
    paths.insert("/news/{id}/sector-links".into(), json!({
        "parameters": [id_param()],
        "get": op("news", "List linked sectors."),
        "post": op("news", "Link a sector.")
    }));
    paths.insert("/news/{id}/macro-links".into(), json!({
        "parameters": [id_param()],
        "get": op("news", "List linked macro indicators."),
        "post": op("news", "Link a macro indicator.")
    }));
    paths.insert("/news/{id}/country-links".into(), json!({
        "parameters": [id_param()],
        "get": op("news", "List linked countries."),
        "post": op("news", "Link a country.")
    }));
    paths.insert("/news/{id}/translations".into(), json!({
        "parameters": [id_param()],
        "get": op("news", "List all locale translations for a news item.")
    }));
    paths.insert("/news/{id}/translations/{locale}".into(), json!({
        "parameters": [id_param(), path_str_param("locale")],
        "put": op("news", "Upsert a single-locale translation (title / summary / content_md / agent_summary_md).")
    }));

    // ── calendar (briefs / earnings / macro events / catalysts) ───────────
    paths.insert("/market-briefs".into(), json!({
        "get": {
            "tags": ["calendar"],
            "summary": "List market briefs (pre/post-market / smart_money_scan).",
            "parameters": [country_param(), locale_param()],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("calendar", "Upsert a market brief (natural key: country + kind + trade_date).")
    }));
    paths.insert("/market-briefs/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("calendar", "Fetch one market brief."),
        "delete": op("calendar", "Delete a market brief.")
    }));
    paths.insert("/earnings".into(), json!({
        "get": {
            "tags": ["calendar"],
            "summary": "List earnings events.",
            "parameters": [country_param()],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("calendar", "Upsert an earnings event (natural key: stock_id + fiscal_year + fiscal_period).")
    }));
    paths.insert("/earnings/{id}".into(), json!({
        "parameters": [id_param()],
        "get": op("calendar", "Fetch one earnings event."),
        "delete": op("calendar", "Delete an earnings event.")
    }));
    paths.insert("/catalysts".into(), json!({
        "get": {
            "tags": ["calendar"],
            "summary": "List catalysts.",
            "parameters": [country_param(), locale_param()],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("calendar", "Create a catalyst.")
    }));
    paths.insert("/catalysts/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("calendar", "Fetch one catalyst."),
        "delete": op("calendar", "Delete a catalyst.")
    }));

    // ── macros ────────────────────────────────────────────────────────────
    paths.insert("/macro/indicators".into(), json!({
        "get": op("macros", "List macro indicator codes."),
        "post": op("macros", "Upsert a macro indicator.")
    }));
    paths.insert("/macro/indicators/{code}/observations".into(), json!({
        "parameters": [path_str_param("code")],
        "get": op("macros", "List observations for an indicator.")
    }));
    paths.insert("/macro/observations".into(), json!({
        "post": op("macros", "Insert one observation.")
    }));
    paths.insert("/macro/events".into(), json!({
        "get": {
            "tags": ["macros"],
            "summary": "List discrete macro events (FOMC decision, CPI release, …).",
            "parameters": [country_param(), locale_param()],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("macros", "Upsert a macro event (natural key: indicator_code + event_date).")
    }));
    paths.insert("/macro/events/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("macros", "Fetch one macro event."),
        "delete": op("macros", "Delete a macro event.")
    }));

    // ── analyst / insider / filings / fundamentals / connect ──────────────
    paths.insert("/analyst/estimates".into(), json!({ "post": op("analyst", "Insert an analyst estimate.") }));
    paths.insert("/analyst/ratings".into(), json!({ "post": op("analyst", "Insert an analyst rating.") }));
    paths.insert("/insider/transactions".into(), json!({ "post": op("insider", "Insert an insider transaction.") }));
    paths.insert("/filings".into(), json!({ "post": op("filings", "Create a filing entry.") }));
    paths.insert("/filings/{id}".into(), json!({
        "parameters": [id_param()],
        "get": op("filings", "Fetch one filing.")
    }));
    paths.insert("/fundamentals".into(), json!({ "post": op("fundamentals", "Insert a fundamentals snapshot.") }));
    paths.insert("/connect/flow".into(), json!({
        "get": op("connect", "List Stock Connect daily flow."),
        "post": op("connect", "Insert a flow observation.")
    }));
    paths.insert("/connect/holdings".into(), json!({
        "post": op("connect", "Insert a Stock Connect holdings snapshot.")
    }));

    // ── agent outputs: screeners / portfolio-reviews / recs / self-exams /
    //    correlations / universes ─────────────────────────────────────────
    paths.insert("/screener-runs".into(), json!({
        "get": {
            "tags": ["agent-outputs"],
            "summary": "List screener runs.",
            "parameters": [locale_param()],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("agent-outputs", "Upsert a screener run (natural key: name + kind + run_date).")
    }));
    paths.insert("/screener-runs/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("agent-outputs", "Fetch one screener run.")
    }));
    paths.insert("/screener-runs/{id}/hits".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("agent-outputs", "List hits for a run."),
        "post": op("agent-outputs", "Insert a hit for a run.")
    }));

    paths.insert("/portfolio-reviews".into(), json!({
        "get": {
            "tags": ["agent-outputs"],
            "summary": "List portfolio reviews (weekly / monthly / quarterly).",
            "parameters": [locale_param()],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("agent-outputs", "Upsert a portfolio review (natural key: kind + period_start).")
    }));
    paths.insert("/portfolio-reviews/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("agent-outputs", "Fetch one portfolio review."),
        "delete": op("agent-outputs", "Delete a portfolio review.")
    }));

    paths.insert("/recommendations".into(), json!({
        "get": {
            "tags": ["agent-outputs"],
            "summary": "List recommendations.",
            "parameters": [
                json!({ "name": "status", "in": "query", "schema": { "type": "string" } }),
                json!({ "name": "stock_id", "in": "query", "schema": { "type": "integer", "format": "int64" } }),
                locale_param()
            ],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("agent-outputs", "Create a recommendation (initial status 'open').")
    }));
    paths.insert("/recommendations/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("agent-outputs", "Fetch one recommendation."),
        "delete": op("agent-outputs", "Delete a recommendation.")
    }));
    paths.insert("/recommendations/{id}/close".into(), json!({
        "parameters": [id_param()],
        "post": op("agent-outputs", "Close-out a recommendation (status, outcome_md, pnl_pct, closed_at).")
    }));

    paths.insert("/self-exams".into(), json!({
        "get": {
            "tags": ["agent-outputs"],
            "summary": "List self-exams (agent reflecting on past recommendations).",
            "parameters": [
                json!({ "name": "kind", "in": "query", "schema": { "type": "string" } }),
                locale_param()
            ],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("agent-outputs", "Upsert a self-exam (natural key: kind + period_start).")
    }));
    paths.insert("/self-exams/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("agent-outputs", "Fetch one self-exam."),
        "delete": op("agent-outputs", "Delete a self-exam.")
    }));

    paths.insert("/universes".into(), json!({
        "get": op("agent-outputs", "List correlation universes (named stock sets)."),
        "post": op("agent-outputs", "Upsert a universe by name.")
    }));
    paths.insert("/universes/{id}".into(), json!({
        "parameters": [id_param()],
        "get": op("agent-outputs", "Fetch one universe.")
    }));
    paths.insert("/correlation-runs".into(), json!({
        "get": {
            "tags": ["agent-outputs"],
            "summary": "List correlation runs.",
            "parameters": [locale_param()],
            "responses": { "200": { "description": "OK" } }
        },
        "post": op("agent-outputs", "Create a correlation run.")
    }));
    paths.insert("/correlation-runs/{id}".into(), json!({
        "parameters": [id_param(), locale_param()],
        "get": op("agent-outputs", "Fetch one correlation run.")
    }));
    paths.insert("/correlation-runs/{id}/pairs".into(), json!({
        "parameters": [id_param()],
        "get": op("agent-outputs", "List pair correlations for a run."),
        "post": op("agent-outputs", "Insert a pair correlation.")
    }));

    // ── audit ─────────────────────────────────────────────────────────────
    paths.insert("/audit".into(), json!({
        "get": op("audit", "List audit log entries (server-side write trail).")
    }));

    let _ = op_with_body; // silence unused-helper warning until we wire schemas

    Value::Object(paths)
}
