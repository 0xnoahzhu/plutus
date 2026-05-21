//! Database handle wrapping toasty's `Db`.
//!
//! Toasty's runtime API takes `&mut Db` for every query. To share the handle
//! across axum's request-handling tasks we wrap it in an async `Mutex`. With a
//! single user and a single agent, contention is negligible.

use std::sync::Arc;
use tokio::sync::Mutex;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("toasty error: {0}")]
    Toasty(#[from] toasty::Error),
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Clone)]
pub struct Db {
    inner: Arc<Mutex<toasty::Db>>,
    /// Original connection URL, stashed so [`Db::raw_client`] can open a
    /// fresh `tokio_postgres` connection for queries that need JSON
    /// operators or other features toasty 0.6 doesn't express.
    url: Arc<String>,
}

/// Owned wrapper around a fresh `tokio_postgres::Client`. The background
/// connection task is held alongside the client so it stays alive for
/// the lifetime of the handle; dropping this drops both.
pub struct RawClient {
    client: tokio_postgres::Client,
    _handle: tokio::task::JoinHandle<()>,
}

impl std::ops::Deref for RawClient {
    type Target = tokio_postgres::Client;
    fn deref(&self) -> &tokio_postgres::Client { &self.client }
}

impl std::ops::DerefMut for RawClient {
    fn deref_mut(&mut self) -> &mut tokio_postgres::Client { &mut self.client }
}

impl Db {
    /// Open a postgres connection and register all models.
    pub async fn connect(url: &str) -> Result<Self> {
        use crate::models::*;
        let db = toasty::Db::builder()
            .models(toasty::models!(
                User,
                ApiToken,
                WebSession,
                CurrencyRow,
                Market,
                Broker,
                Account,
                Stock,
                WatchlistItem,
                Transaction,
                OhlcvDaily,
                FxRateDaily,
                AuditLog,
                Sector,
                MacroIndicator,
                MacroObservation,
                NewsItem,
                NewsStockLink,
                NewsSectorLink,
                NewsMacroLink,
                NewsCountryLink,
                MarketBrief,
                EarningsEvent,
                MacroEvent,
                WatchlistReport,
                ScreenerRun,
                ScreenerHit,
                PortfolioReview,
                Recommendation,
                SelfExam,
                UniverseDefinition,
                CorrelationRun,
                CorrelationPair,
                Catalyst,
                TradePlan,
                TradePlanLevel,
                PendingOrder,
                Filing,
                FundamentalsQuarterly,
                AnalystEstimate,
                AnalystRating,
                InsiderTransaction,
                ConnectFlowDaily,
                ConnectHoldingsDaily,
            ))
            .connect(url)
            .await?;
        Ok(Self {
            inner: Arc::new(Mutex::new(db)),
            url: Arc::new(url.to_string()),
        })
    }

    /// Open a fresh `tokio_postgres::Client` for raw SQL access. Used by
    /// query modules that need features toasty 0.6 doesn't speak (JSON
    /// operators, full-text search). Caller owns the client and the
    /// background connection task; both are dropped when the returned
    /// guard goes out of scope.
    pub async fn raw_client(&self) -> Result<RawClient> {
        let (client, connection) = tokio_postgres::connect(&self.url, tokio_postgres::NoTls)
            .await
            .map_err(DbError::from)?;
        let handle = tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::warn!("raw_client connection ended: {e}");
            }
        });
        Ok(RawClient { client, _handle: handle })
    }

    /// Apply schema. For Phase 0 this delegates to toasty's `push_schema`,
    /// which creates missing tables. Things toasty can't express (ALTER on
    /// existing tables, pgvector columns) are layered on top by
    /// [`Db::post_migrate_sql`].
    pub async fn migrate(&self) -> Result<()> {
        let guard = self.inner.lock().await;
        guard.push_schema().await?;
        Ok(())
    }

}

/// Post-migrate hook. Runs raw SQL via a side `tokio-postgres` connection for
/// things toasty 0.6 can't express: `ALTER TABLE` on existing tables, JSONB
/// columns + JSON-op queries, and `DROP TABLE` for tables that were retired
/// after an audit (see the "Retired tables" block at the bottom).
///
/// All statements are idempotent (`CREATE ... IF NOT EXISTS`, `DROP ... IF EXISTS`,
/// `ADD COLUMN IF NOT EXISTS`), so this can be safely called on every boot.
pub async fn post_migrate(url: &str) -> Result<()> {
    let (client, connection) = tokio_postgres::connect(url, tokio_postgres::NoTls)
        .await
        .map_err(DbError::from)?;

    // The connection future must be polled for the client to make progress;
    // it ends on disconnect.
    let conn_task = tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::warn!("post_migrate side connection ended: {e}");
        }
    });

    client
        .batch_execute(POST_MIGRATE_SQL)
        .await
        .map_err(DbError::from)?;

    drop(client);
    let _ = conn_task.await;
    Ok(())
}

const POST_MIGRATE_SQL: &str = r#"
-- ── Users + per-row ownership ──────────────────────────────────────────────
--
-- Multi-user mode adds a `users` table and a `user_id` column on every
-- per-user data table. Each ALTER is idempotent (`ADD COLUMN IF NOT EXISTS`)
-- so re-running migrate on an old database backfills the column with `0`
-- (the sentinel for "orphaned / pre-multi-user"). The admin account is NOT a
-- row here — its credentials come from PLUTUS_ADMIN_USERNAME /
-- PLUTUS_ADMIN_PASSWORD env vars.
CREATE TABLE IF NOT EXISTS users (
    id                       BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    username                 TEXT NOT NULL,
    password_hash            TEXT NOT NULL,
    password_reset_required  BOOLEAN NOT NULL DEFAULT FALSE,
    -- CSV of two-letter country codes (e.g. 'US,HK,CN') scoping which
    -- market tabs the user sees in the web UI. Defaults to all three so
    -- existing users carry on with current behavior; admin can narrow it
    -- per-user from /admin. See `User::country_codes()` for parsing.
    allowed_countries        TEXT NOT NULL DEFAULT 'US,HK,CN',
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS users_username_uniq ON users (username);
-- Backfill for databases created before allowed_countries existed.
ALTER TABLE users ADD COLUMN IF NOT EXISTS allowed_countries TEXT NOT NULL DEFAULT 'US,HK,CN';

-- Toasty-managed tables: ensure user_id exists for older deployments.
ALTER TABLE accounts          ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE transactions      ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
-- Canonicalize `transactions.kind` to upper SCREAMING_SNAKE_CASE. Agents
-- following an earlier version of the OpenAPI docs wrote lowercase values
-- (`buy`, `sell`, ...) which the holdings rollup silently filtered out
-- (the `TransactionKind` enum only matched upper-case). The handler now
-- canonicalizes on write; this one-time UPDATE fixes already-written
-- rows. Idempotent — repeated runs find no lower-case values left.
UPDATE transactions
   SET kind = UPPER(kind)
 WHERE kind <> UPPER(kind);

-- Account dedup + uniqueness. There was no constraint preventing two
-- account rows with the same (user_id, broker_id, account_number), and a
-- user actually hit this — two "IBKR / U18630011" rows under one user,
-- one with all the transactions and one empty. This block deletes the
-- redundant rows (ONLY when they're truly unreferenced — zero
-- transactions, zero pending orders) and then installs the unique index.
-- NULLS NOT DISTINCT so two accounts both lacking account_number still
-- collide; without it Postgres treats every NULL pair as distinct.
DELETE FROM accounts a
 WHERE EXISTS (
     SELECT 1
       FROM accounts other
      WHERE other.user_id = a.user_id
        AND other.broker_id = a.broker_id
        AND other.account_number IS NOT DISTINCT FROM a.account_number
        AND other.id <> a.id
 )
   AND NOT EXISTS (SELECT 1 FROM transactions  t WHERE t.account_id  = a.id)
   AND NOT EXISTS (SELECT 1 FROM pending_orders p WHERE p.account_id = a.id);
CREATE UNIQUE INDEX IF NOT EXISTS accounts_user_broker_number_uniq
    ON accounts (user_id, broker_id, account_number)
    NULLS NOT DISTINCT;
ALTER TABLE api_tokens        ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE api_tokens        ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT FALSE;
-- Plaintext token, kept alongside the hash so the list view can show +
-- copy without regenerating. Nullable: pre-existing rows from before this
-- column landed have NULL and render as "—" (no copy button). New tokens
-- always populate it. See models/api_token.rs for the trade-off rationale.
ALTER TABLE api_tokens        ADD COLUMN IF NOT EXISTS token_plain TEXT;
-- Tokens are hard-deleted on revoke now; purge any soft-revoked rows
-- from the previous design and drop the column. Wrapped in a DO block so
-- referencing `revoked_at` doesn't fail on already-migrated databases
-- where the column is gone.
DO $migrate_api_tokens_hard_delete$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'api_tokens' AND column_name = 'revoked_at'
    ) THEN
        EXECUTE 'DELETE FROM api_tokens WHERE revoked_at IS NOT NULL';
    END IF;
END
$migrate_api_tokens_hard_delete$;
ALTER TABLE api_tokens        DROP COLUMN IF EXISTS revoked_at;
ALTER TABLE audit_log         ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE web_sessions      ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE web_sessions      ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE web_sessions      ADD COLUMN IF NOT EXISTS username TEXT NOT NULL DEFAULT '';
ALTER TABLE watchlist_items   ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
-- Replace the single-user UNIQUE(stock_id) with UNIQUE(user_id, stock_id) so
-- multiple users can independently watch the same ticker. Older deployments
-- where the legacy `watchlist_items_stock_uniq` index was never installed may
-- have duplicate rows; dedup by keeping the lowest id per (user_id, stock_id)
-- before promoting to UNIQUE so this migration is replayable.
DROP INDEX IF EXISTS watchlist_items_stock_uniq;
DELETE FROM watchlist_items a
    USING watchlist_items b
    WHERE a.id > b.id
      AND a.user_id = b.user_id
      AND a.stock_id = b.stock_id;
CREATE UNIQUE INDEX IF NOT EXISTS watchlist_items_user_stock_uniq
    ON watchlist_items (user_id, stock_id);
CREATE INDEX IF NOT EXISTS accounts_user_idx        ON accounts (user_id);
CREATE INDEX IF NOT EXISTS transactions_user_idx    ON transactions (user_id);
CREATE INDEX IF NOT EXISTS api_tokens_user_idx      ON api_tokens (user_id);
CREATE INDEX IF NOT EXISTS audit_log_user_idx       ON audit_log (user_id);
CREATE INDEX IF NOT EXISTS web_sessions_user_idx    ON web_sessions (user_id);

-- stocks: translatable content (name, description_md) folded from the
-- legacy `stocks.name` / `stocks.description_md` base columns plus every
-- row in `stock_translations`. Side table is dropped after backfill.
-- Source-language convention is `en` (stock base columns were always
-- English; there was no `language` column on the row).
ALTER TABLE stocks ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_stocks_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'stocks' AND column_name = 'name'
    ) THEN
        EXECUTE $sql$
            UPDATE stocks s
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    'en',
                    jsonb_build_object(
                        'name', s.name,
                        'description_md', s.description_md
                    )
                ) || COALESCE(
                    (
                        SELECT jsonb_object_agg(
                            t.locale,
                            jsonb_strip_nulls(jsonb_build_object(
                                'name', t.name,
                                'description_md', t.description_md
                            ))
                        )
                        FROM stock_translations t
                        WHERE t.stock_id = s.id
                    ),
                    '{}'::jsonb
                )
            )
            WHERE s.content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_stocks_content$;
ALTER TABLE stocks DROP COLUMN IF EXISTS name;
ALTER TABLE stocks DROP COLUMN IF EXISTS description_md;
DROP TABLE IF EXISTS stock_translations;

-- news_items: translatable content (title, summary, content_md,
-- agent_summary_md) folded from the legacy base columns plus every row in
-- news_translations. The source-language key comes from the `language`
-- column on news_items. `translator` per-locale metadata is dropped.
ALTER TABLE news_items ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_news_items_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'news_items' AND column_name = 'title'
    ) THEN
        EXECUTE $sql$
            UPDATE news_items n
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    COALESCE(NULLIF(n.language, ''), 'en'),
                    jsonb_build_object(
                        'title',             n.title,
                        'summary',           n.summary,
                        'content_md',        n.content_md,
                        'agent_summary_md',  n.agent_summary_md
                    )
                ) || COALESCE(
                    (
                        SELECT jsonb_object_agg(
                            t.locale,
                            jsonb_strip_nulls(jsonb_build_object(
                                'title',            t.title,
                                'summary',          t.summary,
                                'content_md',       t.content_md,
                                'agent_summary_md', t.agent_summary_md
                            ))
                        )
                        FROM news_translations t
                        WHERE t.news_id = n.id
                    ),
                    '{}'::jsonb
                )
            )
            WHERE n.content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_news_items_content$;
ALTER TABLE news_items DROP COLUMN IF EXISTS title;
ALTER TABLE news_items DROP COLUMN IF EXISTS summary;
ALTER TABLE news_items DROP COLUMN IF EXISTS content_md;
ALTER TABLE news_items DROP COLUMN IF EXISTS agent_summary_md;
ALTER TABLE news_items DROP COLUMN IF EXISTS language;
DROP TABLE IF EXISTS news_translations;

-- market_briefs: daily pre-market and post-market notes. Same idempotency
-- trick (IDENTITY) used here because push_schema can't reach this model on
-- the second migrate. Translatable content (headline, content_md) lives in
-- the `content` JSONB column — see the watchlist_reports block below for
-- the design rationale.
CREATE TABLE IF NOT EXISTS market_briefs (
    id               BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id          BIGINT NOT NULL DEFAULT 0,
    country          TEXT,
    kind             TEXT,
    trade_date       TEXT,
    sentiment        TEXT,
    sentiment_score  NUMERIC,
    source           TEXT,
    content          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at       TIMESTAMPTZ,
    updated_at       TIMESTAMPTZ
);
ALTER TABLE market_briefs ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE market_briefs ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_market_briefs_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'market_briefs' AND column_name = 'headline'
    ) THEN
        EXECUTE $sql$
            UPDATE market_briefs
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    COALESCE(NULLIF(language, ''), 'en'),
                    jsonb_build_object(
                        'headline',   headline,
                        'content_md', content_md
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_market_briefs_content$;
ALTER TABLE market_briefs DROP COLUMN IF EXISTS headline;
ALTER TABLE market_briefs DROP COLUMN IF EXISTS content_md;
ALTER TABLE market_briefs DROP COLUMN IF EXISTS language;
ALTER TABLE market_briefs DROP COLUMN IF EXISTS translations;
DROP INDEX IF EXISTS market_briefs_natural_key;
CREATE INDEX IF NOT EXISTS market_briefs_user_idx ON market_briefs (user_id);
CREATE INDEX IF NOT EXISTS market_briefs_country_idx ON market_briefs (country);
CREATE INDEX IF NOT EXISTS market_briefs_kind_idx ON market_briefs (kind);
CREATE INDEX IF NOT EXISTS market_briefs_date_idx ON market_briefs (trade_date);
CREATE UNIQUE INDEX IF NOT EXISTS market_briefs_natural_key
    ON market_briefs (user_id, country, kind, trade_date);

-- earnings_events: per-stock-per-fiscal-period earnings calendar entries.
CREATE TABLE IF NOT EXISTS earnings_events (
    id                 BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    stock_id           BIGINT,
    fiscal_year        INTEGER,
    fiscal_period      TEXT,
    announce_at        TIMESTAMPTZ,
    announce_date      TEXT,
    announce_timing    TEXT,
    status             TEXT,
    eps_estimate       NUMERIC,
    eps_actual         NUMERIC,
    revenue_estimate   NUMERIC,
    revenue_actual     NUMERIC,
    currency           TEXT,
    guidance_md        TEXT,
    notes              TEXT,
    url                TEXT,
    source             TEXT,
    created_at         TIMESTAMPTZ,
    updated_at         TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS earnings_events_stock_idx ON earnings_events (stock_id);
CREATE INDEX IF NOT EXISTS earnings_events_date_idx ON earnings_events (announce_date);
CREATE INDEX IF NOT EXISTS earnings_events_status_idx ON earnings_events (status);
CREATE UNIQUE INDEX IF NOT EXISTS earnings_events_natural_key
    ON earnings_events (stock_id, fiscal_year, fiscal_period);

-- OHLCV: dedup so bulk loaders can re-run without duplicating bars.
-- Table itself is toasty-managed; this index lives in post_migrate SQL.
CREATE UNIQUE INDEX IF NOT EXISTS ohlcv_daily_natural_key
    ON ohlcv_daily (stock_id, trade_date);

-- macro_events: discrete policy / data-release events (FOMC, CPI, LPR, …).
-- Shared table (no user_id). Translatable content (title, summary_md) lives
-- in a single `content` JSONB column.
CREATE TABLE IF NOT EXISTS macro_events (
    id                  BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    indicator_code      TEXT,
    event_date          TEXT,
    event_kind          TEXT,
    decision            TEXT,
    decision_bps        INTEGER,
    new_value           NUMERIC,
    consensus_estimate  NUMERIC,
    surprise            NUMERIC,
    previous_value      NUMERIC,
    vote                TEXT,
    dot_plot            TEXT,
    url                 TEXT,
    source              TEXT,
    content             JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ,
    updated_at          TIMESTAMPTZ
);
ALTER TABLE macro_events ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_macro_events_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'macro_events' AND column_name = 'title'
    ) THEN
        EXECUTE $sql$
            UPDATE macro_events
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    'en',
                    jsonb_build_object(
                        'title',      title,
                        'summary_md', summary_md
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_macro_events_content$;
ALTER TABLE macro_events DROP COLUMN IF EXISTS title;
ALTER TABLE macro_events DROP COLUMN IF EXISTS summary_md;
ALTER TABLE macro_events DROP COLUMN IF EXISTS translations;
CREATE INDEX IF NOT EXISTS macro_events_indicator_idx ON macro_events (indicator_code);
CREATE INDEX IF NOT EXISTS macro_events_date_idx ON macro_events (event_date);
CREATE INDEX IF NOT EXISTS macro_events_kind_idx ON macro_events (event_kind);
CREATE UNIQUE INDEX IF NOT EXISTS macro_events_natural_key
    ON macro_events (indicator_code, event_date);

-- macro_observations: time series. The model docstring claims a natural
-- key of (indicator_code, obs_date) "enforced at the app layer", but
-- the old insert path was a plain INSERT and the index didn't exist,
-- so a re-POST of the same day's value silently dup'd. Dedupe first
-- (keep max id per group — that's the latest revision the agent sent),
-- then add the unique index so future POSTs go through ON CONFLICT.
DELETE FROM macro_observations
WHERE id NOT IN (
    SELECT MAX(id) FROM macro_observations
    GROUP BY indicator_code, obs_date
);
CREATE UNIQUE INDEX IF NOT EXISTS macro_observations_natural_key
    ON macro_observations (indicator_code, obs_date);

-- The user's watchlist is a single flat list — the historical `watchlists`
-- group table is gone and `watchlist_items` no longer carries `watchlist_id`.
-- The DROP/ALTER below clean up older deployments that still have the
-- per-group columns. The unique index is now per-user (see `watchlist_items_user_stock_uniq` above).
DROP TABLE IF EXISTS watchlists CASCADE;
ALTER TABLE watchlist_items DROP COLUMN IF EXISTS watchlist_id;
DROP INDEX IF EXISTS watchlist_items_watchlist_idx;

-- watchlist_reports: daily / weekly briefs for the watchlist. Per-user.
-- Translatable text (headline, summary_md, content_md, notes) lives in
-- the `content` JSONB column shaped as
--   { "<locale>": { "headline": "...", "summary_md": "...", "content_md": "...", "notes": "..." } }
-- Queries pick the right locale at SELECT time with JSON operators —
-- there are no per-locale base columns to keep in sync anymore.
CREATE TABLE IF NOT EXISTS watchlist_reports (
    id               BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id          BIGINT NOT NULL DEFAULT 0,
    kind             TEXT,
    period_start     TEXT,
    period_end       TEXT,
    sentiment        TEXT,
    sentiment_score  NUMERIC,
    metrics          TEXT,
    source           TEXT,
    content          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at       TIMESTAMPTZ,
    updated_at       TIMESTAMPTZ
);
-- Multi-language refactor migration: add the JSONB column to older
-- deployments, backfill it from the legacy base + `translations` TEXT
-- columns, then drop the legacy columns. Each ALTER is independently
-- idempotent so re-running migrate is safe.
ALTER TABLE watchlist_reports ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE watchlist_reports ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_watchlist_reports_content$
BEGIN
    -- Only backfill rows that still have the legacy columns present and
    -- haven't been migrated yet (content still empty). Information_schema
    -- check keeps this no-op on already-migrated deployments.
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'watchlist_reports' AND column_name = 'headline'
    ) THEN
        EXECUTE $sql$
            UPDATE watchlist_reports
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    COALESCE(NULLIF(language, ''), 'en'),
                    jsonb_build_object(
                        'headline',   headline,
                        'summary_md', summary_md,
                        'content_md', content_md,
                        'notes',      notes
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_watchlist_reports_content$;
ALTER TABLE watchlist_reports DROP COLUMN IF EXISTS headline;
ALTER TABLE watchlist_reports DROP COLUMN IF EXISTS summary_md;
ALTER TABLE watchlist_reports DROP COLUMN IF EXISTS content_md;
ALTER TABLE watchlist_reports DROP COLUMN IF EXISTS notes;
ALTER TABLE watchlist_reports DROP COLUMN IF EXISTS language;
ALTER TABLE watchlist_reports DROP COLUMN IF EXISTS translations;
ALTER TABLE watchlist_reports DROP COLUMN IF EXISTS watchlist_id;
DROP INDEX IF EXISTS watchlist_reports_watchlist_idx;
DROP INDEX IF EXISTS watchlist_reports_natural_key;
CREATE INDEX IF NOT EXISTS watchlist_reports_user_idx
    ON watchlist_reports (user_id);
CREATE INDEX IF NOT EXISTS watchlist_reports_kind_idx
    ON watchlist_reports (kind);
CREATE INDEX IF NOT EXISTS watchlist_reports_period_idx
    ON watchlist_reports (period_start);
CREATE UNIQUE INDEX IF NOT EXISTS watchlist_reports_natural_key
    ON watchlist_reports (user_id, kind, period_start);

-- ── Phase 2: agent jobs ─────────────────────────────────────────────────

-- screener_runs: translatable content (description_md, summary_md) lives
-- in a single `content` JSONB column. `name` stays a native column because
-- it's an identifier, not translatable text.
CREATE TABLE IF NOT EXISTS screener_runs (
    id              BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL DEFAULT 0,
    name            TEXT,
    kind            TEXT,
    run_date        TEXT,
    universe        TEXT,
    universe_size   INTEGER,
    criteria        TEXT,
    sentiment       TEXT,
    source          TEXT,
    content         JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ
);
ALTER TABLE screener_runs ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE screener_runs ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_screener_runs_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'screener_runs' AND column_name = 'description_md'
    ) THEN
        EXECUTE $sql$
            UPDATE screener_runs
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    COALESCE(NULLIF(language, ''), 'en'),
                    jsonb_build_object(
                        'description_md', description_md,
                        'summary_md',     summary_md
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_screener_runs_content$;
ALTER TABLE screener_runs DROP COLUMN IF EXISTS description_md;
ALTER TABLE screener_runs DROP COLUMN IF EXISTS summary_md;
ALTER TABLE screener_runs DROP COLUMN IF EXISTS language;
ALTER TABLE screener_runs DROP COLUMN IF EXISTS translations;
DROP INDEX IF EXISTS screener_runs_natural_key;
CREATE INDEX IF NOT EXISTS screener_runs_user_idx ON screener_runs (user_id);
CREATE INDEX IF NOT EXISTS screener_runs_name_idx ON screener_runs (name);
CREATE INDEX IF NOT EXISTS screener_runs_date_idx ON screener_runs (run_date);
CREATE UNIQUE INDEX IF NOT EXISTS screener_runs_natural_key
    ON screener_runs (user_id, name, kind, run_date);

-- screener_hits: user_id is denormalized from the parent run so per-user
-- filtering doesn't require a join. Translatable rationale_md lives in a
-- single `content` JSONB column.
CREATE TABLE IF NOT EXISTS screener_hits (
    id              BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL DEFAULT 0,
    run_id          BIGINT,
    stock_id        BIGINT,
    rank            INTEGER,
    score           NUMERIC,
    metrics         TEXT,
    content         JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ
);
ALTER TABLE screener_hits ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE screener_hits ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_screener_hits_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'screener_hits' AND column_name = 'rationale_md'
    ) THEN
        EXECUTE $sql$
            UPDATE screener_hits
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    'en',
                    jsonb_build_object(
                        'rationale_md', rationale_md
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_screener_hits_content$;
ALTER TABLE screener_hits DROP COLUMN IF EXISTS rationale_md;
ALTER TABLE screener_hits DROP COLUMN IF EXISTS translations;
CREATE INDEX IF NOT EXISTS screener_hits_user_idx ON screener_hits (user_id);
CREATE INDEX IF NOT EXISTS screener_hits_run_idx ON screener_hits (run_id);
CREATE INDEX IF NOT EXISTS screener_hits_stock_idx ON screener_hits (stock_id);

-- portfolio_reviews: translatable content (headline, summary_md, content_md,
-- decisions_md) moved into a single `content` JSONB column.
CREATE TABLE IF NOT EXISTS portfolio_reviews (
    id              BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL DEFAULT 0,
    kind            TEXT,
    period_start    TEXT,
    period_end      TEXT,
    sentiment       TEXT,
    sentiment_score NUMERIC,
    metrics         TEXT,
    source          TEXT,
    content         JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ
);
ALTER TABLE portfolio_reviews ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE portfolio_reviews ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_portfolio_reviews_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'portfolio_reviews' AND column_name = 'headline'
    ) THEN
        EXECUTE $sql$
            UPDATE portfolio_reviews
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    COALESCE(NULLIF(language, ''), 'en'),
                    jsonb_build_object(
                        'headline',     headline,
                        'summary_md',   summary_md,
                        'content_md',   content_md,
                        'decisions_md', decisions_md
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_portfolio_reviews_content$;
ALTER TABLE portfolio_reviews DROP COLUMN IF EXISTS headline;
ALTER TABLE portfolio_reviews DROP COLUMN IF EXISTS summary_md;
ALTER TABLE portfolio_reviews DROP COLUMN IF EXISTS content_md;
ALTER TABLE portfolio_reviews DROP COLUMN IF EXISTS decisions_md;
ALTER TABLE portfolio_reviews DROP COLUMN IF EXISTS language;
ALTER TABLE portfolio_reviews DROP COLUMN IF EXISTS translations;
DROP INDEX IF EXISTS portfolio_reviews_natural_key;
CREATE INDEX IF NOT EXISTS portfolio_reviews_user_idx ON portfolio_reviews (user_id);
CREATE INDEX IF NOT EXISTS portfolio_reviews_kind_idx ON portfolio_reviews (kind);
CREATE INDEX IF NOT EXISTS portfolio_reviews_period_idx ON portfolio_reviews (period_start);
CREATE UNIQUE INDEX IF NOT EXISTS portfolio_reviews_natural_key
    ON portfolio_reviews (user_id, kind, period_start);

-- recommendations: translatable content (rationale_md, outcome_md) lives
-- in a single `content` JSONB column.
CREATE TABLE IF NOT EXISTS recommendations (
    id              BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL DEFAULT 0,
    stock_id        BIGINT,
    sector_code     TEXT,
    action          TEXT,
    confidence      NUMERIC,
    target_price    NUMERIC,
    target_currency TEXT,
    target_horizon  TEXT,
    issued_at       TIMESTAMPTZ,
    status          TEXT,
    pnl_pct         NUMERIC,
    closed_at       TIMESTAMPTZ,
    source          TEXT,
    content         JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ
);
ALTER TABLE recommendations ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE recommendations ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_recommendations_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'recommendations' AND column_name = 'rationale_md'
    ) THEN
        EXECUTE $sql$
            UPDATE recommendations
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    COALESCE(NULLIF(language, ''), 'en'),
                    jsonb_build_object(
                        'rationale_md', rationale_md,
                        'outcome_md',   outcome_md
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_recommendations_content$;
ALTER TABLE recommendations DROP COLUMN IF EXISTS rationale_md;
ALTER TABLE recommendations DROP COLUMN IF EXISTS outcome_md;
ALTER TABLE recommendations DROP COLUMN IF EXISTS language;
ALTER TABLE recommendations DROP COLUMN IF EXISTS translations;
CREATE INDEX IF NOT EXISTS recommendations_user_idx ON recommendations (user_id);
CREATE INDEX IF NOT EXISTS recommendations_stock_idx ON recommendations (stock_id);
CREATE INDEX IF NOT EXISTS recommendations_status_idx ON recommendations (status);
CREATE INDEX IF NOT EXISTS recommendations_issued_idx ON recommendations (issued_at);

-- self_exams: translatable content (headline, content_md, notes) moved
-- into a single `content` JSONB column.
CREATE TABLE IF NOT EXISTS self_exams (
    id                 BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id            BIGINT NOT NULL DEFAULT 0,
    kind               TEXT,
    period_start       TEXT,
    period_end         TEXT,
    metrics            TEXT,
    recommendation_ids TEXT,
    source             TEXT,
    content            JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at         TIMESTAMPTZ,
    updated_at         TIMESTAMPTZ
);
ALTER TABLE self_exams ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE self_exams ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_self_exams_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'self_exams' AND column_name = 'headline'
    ) THEN
        EXECUTE $sql$
            UPDATE self_exams
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    COALESCE(NULLIF(language, ''), 'en'),
                    jsonb_build_object(
                        'headline',   headline,
                        'content_md', content_md,
                        'notes',      notes
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_self_exams_content$;
ALTER TABLE self_exams DROP COLUMN IF EXISTS headline;
ALTER TABLE self_exams DROP COLUMN IF EXISTS content_md;
ALTER TABLE self_exams DROP COLUMN IF EXISTS notes;
ALTER TABLE self_exams DROP COLUMN IF EXISTS language;
ALTER TABLE self_exams DROP COLUMN IF EXISTS translations;
DROP INDEX IF EXISTS self_exams_natural_key;
CREATE INDEX IF NOT EXISTS self_exams_user_idx ON self_exams (user_id);
CREATE INDEX IF NOT EXISTS self_exams_kind_idx ON self_exams (kind);
CREATE UNIQUE INDEX IF NOT EXISTS self_exams_natural_key
    ON self_exams (user_id, kind, period_start);

CREATE TABLE IF NOT EXISTS universe_definitions (
    id              BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL DEFAULT 0,
    name            TEXT,
    description_md  TEXT,
    stock_ids       TEXT,
    created_at      TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ
);
ALTER TABLE universe_definitions ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
-- Older deployments may have a UNIQUE constraint on `name` alone (created via
-- the inline `TEXT UNIQUE` shorthand). Drop it so we can replace with a per-user
-- unique index. Postgres names the auto-generated constraint
-- `universe_definitions_name_key`.
ALTER TABLE universe_definitions DROP CONSTRAINT IF EXISTS universe_definitions_name_key;
CREATE INDEX IF NOT EXISTS universe_definitions_user_idx
    ON universe_definitions (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS universe_definitions_user_name_uniq
    ON universe_definitions (user_id, name);

-- correlation_runs: translatable content (summary_md) lives in a single
-- `content` JSONB column.
CREATE TABLE IF NOT EXISTS correlation_runs (
    id              BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL DEFAULT 0,
    kind            TEXT,
    run_date        TEXT,
    universe_id     BIGINT,
    lookback_days   INTEGER,
    method          TEXT,
    metrics         TEXT,
    source          TEXT,
    content         JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ
);
ALTER TABLE correlation_runs ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE correlation_runs ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_correlation_runs_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'correlation_runs' AND column_name = 'summary_md'
    ) THEN
        EXECUTE $sql$
            UPDATE correlation_runs
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    'en',
                    jsonb_build_object(
                        'summary_md', summary_md
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_correlation_runs_content$;
ALTER TABLE correlation_runs DROP COLUMN IF EXISTS summary_md;
ALTER TABLE correlation_runs DROP COLUMN IF EXISTS translations;
CREATE INDEX IF NOT EXISTS correlation_runs_user_idx ON correlation_runs (user_id);
CREATE INDEX IF NOT EXISTS correlation_runs_universe_idx ON correlation_runs (universe_id);
CREATE INDEX IF NOT EXISTS correlation_runs_date_idx ON correlation_runs (run_date);

-- correlation_pairs: user_id denormalized from the parent run.
CREATE TABLE IF NOT EXISTS correlation_pairs (
    id              BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL DEFAULT 0,
    run_id          BIGINT,
    stock_a_id      BIGINT,
    stock_b_id      BIGINT,
    correlation     NUMERIC
);
ALTER TABLE correlation_pairs ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS correlation_pairs_user_idx ON correlation_pairs (user_id);
CREATE INDEX IF NOT EXISTS correlation_pairs_run_idx ON correlation_pairs (run_id);
CREATE INDEX IF NOT EXISTS correlation_pairs_a_idx ON correlation_pairs (stock_a_id);
CREATE INDEX IF NOT EXISTS correlation_pairs_b_idx ON correlation_pairs (stock_b_id);
CREATE UNIQUE INDEX IF NOT EXISTS correlation_pairs_natural_key
    ON correlation_pairs (run_id, stock_a_id, stock_b_id);

-- catalysts: translatable content (title, summary_md, bull_case_md,
-- bear_case_md, notes) moved into a single JSONB column. See the
-- watchlist_reports block above for the design rationale.
CREATE TABLE IF NOT EXISTS catalysts (
    id              BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL DEFAULT 0,
    stock_id        BIGINT,
    sector_code     TEXT,
    country         TEXT,
    catalyst_kind   TEXT,
    catalyst_date   TEXT,
    date_confidence TEXT,
    impact_level    TEXT,
    status          TEXT,
    url             TEXT,
    source          TEXT,
    content         JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ
);
ALTER TABLE catalysts ADD COLUMN IF NOT EXISTS user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE catalysts ADD COLUMN IF NOT EXISTS content JSONB NOT NULL DEFAULT '{}'::jsonb;
DO $migrate_catalysts_content$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'catalysts' AND column_name = 'title'
    ) THEN
        EXECUTE $sql$
            UPDATE catalysts
            SET content = jsonb_strip_nulls(
                jsonb_build_object(
                    'en',
                    jsonb_build_object(
                        'title',        title,
                        'summary_md',   summary_md,
                        'bull_case_md', bull_case_md,
                        'bear_case_md', bear_case_md,
                        'notes',        notes
                    )
                )
            ) || COALESCE(NULLIF(translations, '')::jsonb, '{}'::jsonb)
            WHERE content = '{}'::jsonb
        $sql$;
    END IF;
END
$migrate_catalysts_content$;
ALTER TABLE catalysts DROP COLUMN IF EXISTS title;
ALTER TABLE catalysts DROP COLUMN IF EXISTS summary_md;
ALTER TABLE catalysts DROP COLUMN IF EXISTS bull_case_md;
ALTER TABLE catalysts DROP COLUMN IF EXISTS bear_case_md;
ALTER TABLE catalysts DROP COLUMN IF EXISTS notes;
ALTER TABLE catalysts DROP COLUMN IF EXISTS translations;
CREATE INDEX IF NOT EXISTS catalysts_user_idx ON catalysts (user_id);
CREATE INDEX IF NOT EXISTS catalysts_stock_idx ON catalysts (stock_id);
CREATE INDEX IF NOT EXISTS catalysts_sector_idx ON catalysts (sector_code);
CREATE INDEX IF NOT EXISTS catalysts_date_idx ON catalysts (catalyst_date);
CREATE INDEX IF NOT EXISTS catalysts_status_idx ON catalysts (status);
CREATE INDEX IF NOT EXISTS catalysts_kind_idx ON catalysts (catalyst_kind);
-- Dedup key: re-running a calendar-update agent (same source, same
-- event) must upsert instead of creating duplicates. NULLS NOT DISTINCT
-- (PG 15+) makes two NULL stock_ids count as a collision, so a country
-- catalyst with stock_id IS NULL still has uniqueness enforced on the
-- rest of the key. `source` is in the key so two different upstream
-- feeds reporting the "same" event don't silently collapse.
CREATE UNIQUE INDEX IF NOT EXISTS catalysts_natural_key
    ON catalysts (user_id, catalyst_kind, catalyst_date, stock_id, sector_code, country, source)
    NULLS NOT DISTINCT;

-- ── Trade plans ────────────────────────────────────────────────────────────
-- Per-user, per-stock plans recording intended buy / sell price points.
-- Two-tier: `trade_plans` is the header (rationale + status), each plan
-- carries N price points in `trade_plan_levels`. Levels denormalize
-- user_id + stock_id from the parent for query speed (same pattern as
-- screener_hits). Status is flipped manually — no auto-trigger on OHLCV
-- crosses; the user is the decision-maker.
CREATE TABLE IF NOT EXISTS trade_plans (
    id          BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id     BIGINT NOT NULL,
    stock_id    BIGINT NOT NULL,
    rationale   TEXT,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS trade_plans_user_idx ON trade_plans (user_id);
CREATE INDEX IF NOT EXISTS trade_plans_stock_idx ON trade_plans (stock_id);
CREATE INDEX IF NOT EXISTS trade_plans_status_idx ON trade_plans (status);

CREATE TABLE IF NOT EXISTS trade_plan_levels (
    id            BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id       BIGINT NOT NULL,
    stock_id      BIGINT NOT NULL,
    plan_id       BIGINT NOT NULL REFERENCES trade_plans(id) ON DELETE CASCADE,
    kind          TEXT NOT NULL,
    price         NUMERIC NOT NULL,
    quantity      NUMERIC,
    fraction_pct  NUMERIC,
    status        TEXT NOT NULL DEFAULT 'active',
    triggered_at  TIMESTAMPTZ,
    notes         TEXT,
    sort_order    INTEGER,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS trade_plan_levels_user_idx ON trade_plan_levels (user_id);
CREATE INDEX IF NOT EXISTS trade_plan_levels_stock_idx ON trade_plan_levels (stock_id);
CREATE INDEX IF NOT EXISTS trade_plan_levels_plan_idx ON trade_plan_levels (plan_id);
CREATE INDEX IF NOT EXISTS trade_plan_levels_status_idx ON trade_plan_levels (status);

-- ── Pending orders ─────────────────────────────────────────────────────────
-- Limit-style orders the user has placed with their broker (or intends to
-- place). Flat single-tier table — each row is a standalone broker order,
-- independent of any trade plan. Optional `trade_plan_level_id` records
-- provenance back to the planned level the order was placed for; that FK
-- uses ON DELETE SET NULL so the order survives a plan deletion (the
-- historical fact that you submitted it doesn't disappear because the
-- intent record went away). `account_id` / `stock_id` are plain BIGINTs
-- with no DB FK, matching the rest of the per-user tables; ownership is
-- enforced in the Rust query layer.
CREATE TABLE IF NOT EXISTS pending_orders (
    id                   BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    user_id              BIGINT NOT NULL,
    account_id           BIGINT NOT NULL,
    stock_id             BIGINT NOT NULL,
    trade_plan_level_id  BIGINT REFERENCES trade_plan_levels(id) ON DELETE SET NULL,
    side                 TEXT NOT NULL,
    order_type           TEXT NOT NULL,
    limit_price          NUMERIC,
    stop_price           NUMERIC,
    quantity             NUMERIC NOT NULL,
    time_in_force        TEXT NOT NULL DEFAULT 'gtc',
    expires_at           TIMESTAMPTZ,
    status               TEXT NOT NULL DEFAULT 'open',
    placed_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    filled_at            TIMESTAMPTZ,
    cancelled_at         TIMESTAMPTZ,
    broker_order_ref     TEXT,
    notes                TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS pending_orders_user_idx       ON pending_orders (user_id);
CREATE INDEX IF NOT EXISTS pending_orders_account_idx    ON pending_orders (account_id);
CREATE INDEX IF NOT EXISTS pending_orders_stock_idx      ON pending_orders (stock_id);
CREATE INDEX IF NOT EXISTS pending_orders_status_idx     ON pending_orders (status);
CREATE INDEX IF NOT EXISTS pending_orders_plan_level_idx ON pending_orders (trade_plan_level_id);

-- ── Foreign keys ───────────────────────────────────────────────────────────
-- Every per-user table carried a `user_id BIGINT NOT NULL DEFAULT 0` from
-- the multi-user migration, but no FK constraint actually enforced
-- referential integrity against `users(id)`. That let early test data
-- accumulate with user_id ∈ {0, 1} (neither row ever existed in users),
-- so the audit log has ~800 orphan rows, web_sessions has ~35, etc.
--
-- Two-step migration:
--   1. UPDATE every per-user table to repoint orphan user_ids at
--      user_id=4 (`0xnoahzhu`), the only real account. We pick "adopt
--      to id=4" instead of DELETE so the historical web_sessions and
--      audit_log entries stay attached to the user who created them.
--   2. Add ON DELETE CASCADE FK constraints. Wrapped in a DO block
--      that checks pg_constraint first so this is idempotent across
--      re-runs and tolerates tables that may have been retired.
--
-- The internal FKs on trade_plan_levels(plan_id) and
-- pending_orders(trade_plan_level_id) are declared in the inline
-- CREATE TABLE DDL above but never actually materialized on the live
-- DB (those tables were created before the FK clauses were added).
-- Recreate them here so a delete of a trade plan cleans up its levels,
-- and a delete of a level nulls out the order's back-reference.
DO $migrate_user_fks$
DECLARE
    t TEXT;
    per_user_tables TEXT[] := ARRAY[
        'accounts', 'transactions', 'watchlist_items', 'watchlist_reports',
        'catalysts', 'screener_runs', 'screener_hits',
        'portfolio_reviews', 'recommendations', 'self_exams',
        'correlation_runs', 'correlation_pairs', 'universe_definitions',
        'trade_plans', 'trade_plan_levels', 'pending_orders',
        'market_briefs', 'web_sessions', 'api_tokens', 'audit_log'
    ];
BEGIN
    -- 1. Adopt orphans to user_id=4 wherever the column exists and the
    --    row's user_id isn't in users.
    FOREACH t IN ARRAY per_user_tables LOOP
        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_name = t AND column_name = 'user_id'
        ) THEN
            EXECUTE format(
                'UPDATE %I SET user_id = 4 WHERE user_id NOT IN (SELECT id FROM users)',
                t
            );
        END IF;
    END LOOP;

    -- 2. Add the FK constraints, ON DELETE CASCADE. Skip the table if
    --    the user_id column is gone, or if the constraint already
    --    exists from a previous run.
    FOREACH t IN ARRAY per_user_tables LOOP
        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_name = t AND column_name = 'user_id'
        ) AND NOT EXISTS (
            SELECT 1 FROM pg_constraint
            WHERE conname = t || '_user_id_fk'
        ) THEN
            EXECUTE format(
                'ALTER TABLE %I ADD CONSTRAINT %I FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE',
                t, t || '_user_id_fk'
            );
        END IF;
    END LOOP;
END
$migrate_user_fks$;

-- Recreate the two missing internal FKs.
DO $migrate_internal_fks$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'trade_plan_levels_plan_id_fk'
    ) THEN
        ALTER TABLE trade_plan_levels
            ADD CONSTRAINT trade_plan_levels_plan_id_fk
            FOREIGN KEY (plan_id) REFERENCES trade_plans(id) ON DELETE CASCADE;
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'pending_orders_trade_plan_level_id_fk'
    ) THEN
        ALTER TABLE pending_orders
            ADD CONSTRAINT pending_orders_trade_plan_level_id_fk
            FOREIGN KEY (trade_plan_level_id) REFERENCES trade_plan_levels(id) ON DELETE SET NULL;
    END IF;
END
$migrate_internal_fks$;

-- ── Retired tables ─────────────────────────────────────────────────────────
-- Cleanup pass after auditing usage. Each DROP is idempotent (IF EXISTS)
-- so re-running migrate on a fresh database is a no-op; on a previously
-- migrated database the orphaned tables go away.
--   - news_embeddings: pgvector table for semantic news search. Schema
--     was set up but nothing ever populated or queried it. Easy to
--     re-add later if the feature lands (CREATE TABLE + HNSW index).
--   - idempotency_keys: framework prep for replaying POST responses
--     across retries. Never wired up; the `IdempotencyKey` newtype was
--     similarly unused.
--   - settings: a generic key/value app-settings store. Replaced by env
--     vars (admin creds, bind addr) and per-browser cookies (locale,
--     theme).
--   - broker_symbols: was meant to map broker-side symbols to the
--     canonical `stocks` table; nothing imported or consulted it.
DROP TABLE IF EXISTS news_embeddings;
DROP TABLE IF EXISTS idempotency_keys;
DROP TABLE IF EXISTS settings;
DROP TABLE IF EXISTS broker_symbols;
"#;

// Re-open the impl so the helpers above are inside the type.
impl Db {
    /// Placeholder to keep the second `impl Db { ... }` block valid in case
    /// future helpers land below the post-migrate definition. No-op.
    #[doc(hidden)]
    pub fn _placeholder(&self) {}

    /// Acquire the underlying toasty handle for a single operation.
    /// Helper modules call this and run their query inside the guard scope.
    pub async fn with<F, R>(&self, f: F) -> R
    where
        F: for<'a> AsyncFnOnce(&'a mut toasty::Db) -> R,
    {
        let mut guard = self.inner.lock().await;
        f(&mut *guard).await
    }
}
