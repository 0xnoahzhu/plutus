//! OpenAPI spec assembly. Phase 0 publishes a minimal spec describing endpoints
//! and DTOs so the hermes agent can discover the API. Full operation
//! annotations can land later as the API stabilizes.

use serde_json::json;

pub fn spec() -> serde_json::Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "plutus API",
            "version": "0.1.0",
            "description": "Personal investment data store. Phase 0 — REST surface for stocks, watchlists, transactions, holdings, OHLCV, and FX rates."
        },
        "servers": [
            { "url": "/api/v1" }
        ],
        "paths": {
            "/healthz": { "get": { "summary": "Liveness probe" } },
            "/auth/login": { "post": { "summary": "Master-password login → session cookie" } },
            "/auth/logout": { "post": { "summary": "Clear session cookie" } },
            "/auth/me": { "get": { "summary": "Identity of the current caller" } },
            "/tokens": {
                "get": { "summary": "List API tokens (web only)" },
                "post": { "summary": "Create an API token (web only)" }
            },
            "/tokens/{id}": { "delete": { "summary": "Revoke a token (web only)" } },
            "/markets": { "get": { "summary": "List markets" } },
            "/brokers": { "get": { "summary": "List brokers" } },
            "/accounts": {
                "get": { "summary": "List accounts" },
                "post": { "summary": "Create an account" }
            },
            "/accounts/{id}": { "get": { "summary": "Fetch one account" } },
            "/stocks": {
                "get": { "summary": "List stocks" },
                "post": { "summary": "Create a stock" }
            },
            "/stocks/{id}": {
                "get": { "summary": "Fetch one stock" },
                "delete": { "summary": "Delete a stock" }
            },
            "/stocks/{id}/translations": { "get": { "summary": "List i18n translations" } },
            "/stocks/{id}/translations/{locale}": { "put": { "summary": "Upsert a translation" } },
            "/stocks/{id}/ohlcv": {
                "get": { "summary": "List OHLCV rows for a stock" },
                "post": { "summary": "Insert one OHLCV row" }
            },
            "/watchlists": {
                "get": { "summary": "List watchlists" },
                "post": { "summary": "Create a watchlist" }
            },
            "/watchlists/{id}": {
                "get": { "summary": "Fetch one watchlist" },
                "delete": { "summary": "Delete a watchlist" }
            },
            "/watchlists/{id}/items": {
                "get": { "summary": "List items" },
                "post": { "summary": "Add a stock to a watchlist" }
            },
            "/watchlists/{id}/items/{stock_id}": { "delete": { "summary": "Remove a stock from a watchlist" } },
            "/transactions": {
                "get": { "summary": "List transactions" },
                "post": { "summary": "Record a transaction" }
            },
            "/transactions/{id}": {
                "get": { "summary": "Fetch one transaction" },
                "delete": { "summary": "Delete a transaction" }
            },
            "/holdings": { "get": { "summary": "Compute holdings from transactions" } },
            "/fx": {
                "get": { "summary": "List FX rates" },
                "post": { "summary": "Insert an FX rate" }
            },
            "/audit": { "get": { "summary": "List audit log entries" } }
        },
        "components": {
            "securitySchemes": {
                "bearer": { "type": "http", "scheme": "bearer" },
                "session": { "type": "apiKey", "in": "cookie", "name": "plutus_session" }
            }
        }
    })
}
