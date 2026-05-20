//! HTTP layer for plutus.
//!
//! Exposes `build_router` for the server binary to mount.

#![allow(clippy::module_name_repetitions)]

pub mod auth;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod i18n;
pub mod openapi;
pub mod state;

pub use error::ApiError;
pub use state::AppState;

use axum::http::{header, HeaderValue, Method};
use axum::routing::{delete, get, patch, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[must_use]
pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        // Meta
        .route("/healthz", get(handlers::meta::healthz))
        .route("/openapi.json", get(handlers::meta::openapi_json))
        .route("/docs", get(handlers::meta::docs))
        // Auth
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/me", get(handlers::auth::me))
        .route("/auth/change-password", post(handlers::auth::change_password))
        // Admin (env-configured admin only; regular users get 403)
        .route(
            "/admin/users",
            get(handlers::admin::users::list).post(handlers::admin::users::create),
        )
        .route(
            "/admin/users/:id",
            delete(handlers::admin::users::delete),
        )
        .route(
            "/admin/users/:id/reset-password",
            post(handlers::admin::users::reset_password),
        )
        .route(
            "/admin/users/:id/countries",
            post(handlers::admin::users::update_countries),
        )
        .route(
            "/admin/brokers",
            get(handlers::admin::brokers::list).post(handlers::admin::brokers::create),
        )
        .route(
            "/admin/brokers/:id",
            patch(handlers::admin::brokers::update).delete(handlers::admin::brokers::delete),
        )
        .route(
            "/admin/tokens",
            get(handlers::admin::tokens::list).post(handlers::admin::tokens::create),
        )
        .route(
            "/admin/tokens/:id",
            delete(handlers::admin::tokens::delete),
        )
        // Tokens (web-only)
        .route("/tokens", get(handlers::tokens::list).post(handlers::tokens::create))
        .route("/tokens/:id", delete(handlers::tokens::delete))
        // Stocks
        .route("/stocks", get(handlers::stocks::list).post(handlers::stocks::create))
        .route(
            "/stocks/:id",
            get(handlers::stocks::get)
                .patch(handlers::stocks::update)
                .delete(handlers::stocks::delete),
        )
        .route(
            "/stocks/:id/ohlcv",
            get(handlers::ohlcv::list_for_stock).post(handlers::ohlcv::insert_one),
        )
        // Watchlist — a single flat list of stocks (no group concept).
        .route(
            "/watchlist/items",
            get(handlers::watchlists::list_items).post(handlers::watchlists::add_item),
        )
        .route(
            "/watchlist/items/:stock_id",
            delete(handlers::watchlists::remove_item),
        )
        // Watchlist reports (daily / weekly)
        .route(
            "/watchlist/reports",
            get(handlers::watchlist_reports::list).post(handlers::watchlist_reports::upsert),
        )
        .route(
            "/watchlist/reports/:id",
            get(handlers::watchlist_reports::get).delete(handlers::watchlist_reports::delete),
        )
        // ── Phase 2 agent jobs ────────────────────────────────────────
        // Screeners
        .route(
            "/screener-runs",
            get(handlers::screeners::list_runs).post(handlers::screeners::upsert_run),
        )
        .route("/screener-runs/:id", get(handlers::screeners::get_run))
        .route(
            "/screener-runs/:id/hits",
            get(handlers::screeners::list_hits).post(handlers::screeners::insert_hit),
        )
        .route(
            "/stocks/:id/screener-hits",
            get(handlers::screeners::list_hits_for_stock),
        )
        // Portfolio reviews
        .route(
            "/portfolio-reviews",
            get(handlers::portfolio_reviews::list).post(handlers::portfolio_reviews::upsert),
        )
        .route(
            "/portfolio-reviews/:id",
            get(handlers::portfolio_reviews::get).delete(handlers::portfolio_reviews::delete),
        )
        // Recommendations
        .route(
            "/recommendations",
            get(handlers::recommendations::list).post(handlers::recommendations::create),
        )
        .route(
            "/recommendations/:id",
            get(handlers::recommendations::get).delete(handlers::recommendations::delete),
        )
        .route(
            "/recommendations/:id/close",
            post(handlers::recommendations::close),
        )
        .route(
            "/stocks/:id/recommendations",
            get(handlers::recommendations::list_for_stock),
        )
        // Self-exams
        .route(
            "/self-exams",
            get(handlers::self_exams::list).post(handlers::self_exams::upsert),
        )
        .route(
            "/self-exams/:id",
            get(handlers::self_exams::get).delete(handlers::self_exams::delete),
        )
        // Correlations
        .route(
            "/universes",
            get(handlers::correlations::list_universes).post(handlers::correlations::upsert_universe),
        )
        .route("/universes/:id", get(handlers::correlations::get_universe))
        .route(
            "/correlation-runs",
            get(handlers::correlations::list_runs).post(handlers::correlations::create_run),
        )
        .route(
            "/correlation-runs/:id",
            get(handlers::correlations::get_run),
        )
        .route(
            "/correlation-runs/:id/pairs",
            get(handlers::correlations::list_pairs).post(handlers::correlations::insert_pair),
        )
        .route(
            "/stocks/:id/correlation-pairs",
            get(handlers::correlations::list_pairs_for_stock),
        )
        // Trade plans (per-user buy/sell price points)
        .route(
            "/trade-plans",
            get(handlers::trade_plans::list).post(handlers::trade_plans::create),
        )
        .route(
            "/trade-plans/:id",
            get(handlers::trade_plans::get)
                .patch(handlers::trade_plans::update)
                .delete(handlers::trade_plans::delete),
        )
        .route(
            "/trade-plans/:id/levels",
            get(handlers::trade_plans::list_levels).post(handlers::trade_plans::add_level),
        )
        .route(
            "/trade-plans/levels/:id",
            patch(handlers::trade_plans::update_level)
                .delete(handlers::trade_plans::delete_level),
        )
        // Pending limit orders (live at the broker)
        .route(
            "/pending-orders",
            get(handlers::pending_orders::list).post(handlers::pending_orders::create),
        )
        .route(
            "/pending-orders/:id",
            get(handlers::pending_orders::get)
                .patch(handlers::pending_orders::update)
                .delete(handlers::pending_orders::delete),
        )
        // Catalysts
        .route(
            "/catalysts",
            get(handlers::catalysts::list).post(handlers::catalysts::create),
        )
        .route(
            "/catalysts/:id",
            get(handlers::catalysts::get).delete(handlers::catalysts::delete),
        )
        .route(
            "/stocks/:id/catalysts",
            get(handlers::catalysts::list_for_stock),
        )
        // Markets / brokers / accounts
        .route("/markets", get(handlers::markets::list))
        .route("/brokers", get(handlers::brokers::list))
        .route(
            "/accounts",
            get(handlers::accounts::list).post(handlers::accounts::create),
        )
        .route(
            "/accounts/:id",
            get(handlers::accounts::get).delete(handlers::accounts::delete),
        )
        // Transactions
        .route(
            "/transactions",
            get(handlers::transactions::list).post(handlers::transactions::create),
        )
        .route(
            "/transactions/:id",
            get(handlers::transactions::get).delete(handlers::transactions::delete),
        )
        // Holdings (derived)
        .route("/holdings", get(handlers::holdings::list))
        // FX
        .route("/fx", get(handlers::fx::list).post(handlers::fx::insert))
        // Audit
        .route("/audit", get(handlers::audit::list))
        // ── Phase 1+ extensions ─────────────────────────────────────────
        // Sectors
        .route(
            "/sectors",
            get(handlers::sectors::list).post(handlers::sectors::upsert),
        )
        // Market briefs (pre/post-market notes)
        .route(
            "/market-briefs",
            get(handlers::market_briefs::list).post(handlers::market_briefs::upsert),
        )
        .route(
            "/market-briefs/:id",
            get(handlers::market_briefs::get).delete(handlers::market_briefs::delete),
        )
        // Earnings calendar
        .route(
            "/earnings",
            get(handlers::earnings::list).post(handlers::earnings::upsert),
        )
        .route(
            "/earnings/:id",
            get(handlers::earnings::get).delete(handlers::earnings::delete),
        )
        .route(
            "/stocks/:id/earnings",
            get(handlers::earnings::list_for_stock),
        )
        // Macro
        .route(
            "/macro/indicators",
            get(handlers::macros::list_indicators).post(handlers::macros::upsert_indicator),
        )
        .route(
            "/macro/indicators/:code/observations",
            get(handlers::macros::list_observations),
        )
        .route(
            "/macro/observations",
            post(handlers::macros::insert_observation),
        )
        // Macro events (FOMC decisions, CPI releases, …)
        .route(
            "/macro/events",
            get(handlers::macro_events::list).post(handlers::macro_events::upsert),
        )
        .route(
            "/macro/events/:id",
            get(handlers::macro_events::get).delete(handlers::macro_events::delete),
        )
        // News + links
        .route("/news", get(handlers::news::list).post(handlers::news::create))
        .route(
            "/news/:id",
            get(handlers::news::get).delete(handlers::news::delete),
        )
        .route(
            "/news/:id/stock-links",
            get(handlers::news::list_stock_links).post(handlers::news::add_stock_link),
        )
        .route(
            "/news/:id/sector-links",
            get(handlers::news::list_sector_links).post(handlers::news::add_sector_link),
        )
        .route(
            "/news/:id/macro-links",
            get(handlers::news::list_macro_links).post(handlers::news::add_macro_link),
        )
        .route(
            "/news/:id/country-links",
            get(handlers::news::list_country_links).post(handlers::news::add_country_link),
        )
        .route("/stocks/:id/news", get(handlers::news::list_news_for_stock))
        // Filings
        .route(
            "/filings",
            post(handlers::filings::create),
        )
        .route("/filings/:id", get(handlers::filings::get))
        .route(
            "/stocks/:id/filings",
            get(handlers::filings::list_for_stock),
        )
        // Fundamentals
        .route(
            "/fundamentals",
            post(handlers::fundamentals::insert),
        )
        .route(
            "/stocks/:id/fundamentals",
            get(handlers::fundamentals::list_for_stock),
        )
        // Analyst
        .route(
            "/analyst/estimates",
            post(handlers::analyst::insert_estimate),
        )
        .route(
            "/analyst/ratings",
            post(handlers::analyst::insert_rating),
        )
        .route(
            "/stocks/:id/analyst/estimates",
            get(handlers::analyst::list_estimates),
        )
        .route(
            "/stocks/:id/analyst/ratings",
            get(handlers::analyst::list_ratings),
        )
        // Insider
        .route(
            "/insider/transactions",
            post(handlers::insider::insert),
        )
        .route(
            "/stocks/:id/insider",
            get(handlers::insider::list_for_stock),
        )
        // Stock Connect
        .route(
            "/connect/flow",
            get(handlers::connect::list_flow).post(handlers::connect::insert_flow),
        )
        .route(
            "/connect/holdings",
            post(handlers::connect::insert_holdings),
        )
        .route(
            "/stocks/:id/connect/holdings",
            get(handlers::connect::list_holdings_for_stock),
        )
        .with_state(state.clone());

    let auth_state = state.clone();
    let api = api.layer(axum::middleware::from_fn(move |req, next| {
        let s = auth_state.clone();
        async move { auth::middleware::extract_actor_inner(s, req, next).await }
    }));

    Router::new()
        .nest("/api/v1", api)
        .route("/", get(handlers::meta::root))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::HeaderName::from_static("idempotency-key")])
                .allow_origin(HeaderValue::from_static("http://127.0.0.1:3000")),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
