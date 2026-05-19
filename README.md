# plutus

Personal investment data store. The hermes AI agent writes data here through a
REST API; the web UI is the human-side viewer.

Phase 0 status: **foundation complete**. Schema, multi-currency transactions,
FIFO/LIFO/Average cost-basis-derived holdings, OHLCV, FX rates, audit log
table, idempotency table, OpenAPI spec, and a Remix 3 web UI with persistent
market-chip filtering.

## Stack

- **Backend**: Rust 1.95, axum 0.7, toasty 0.6.1 (tokio-rs ORM).
- **Database**: PostgreSQL 18.4 + pgvector 0.8.2 + Apache AGE 1.7.0
  (extensions installed but unused in Phase 0).
- **Frontend**: Remix 3 beta (server-rendered, no React).
- **Deployment**: docker compose and podman compose (Compose Spec compliant).

Workspace layout:

```
plutus/
├── Cargo.toml                # workspace root
├── crates/
│   ├── plutus-core/          # types, errors, cost-basis algorithms
│   ├── plutus-storage/       # toasty models + queries
│   ├── plutus-api/           # axum router, DTOs, middleware
│   └── plutus-server/        # binary: clap subcommands
├── web/                      # Remix 3 frontend
├── deploy/
│   ├── postgres/Dockerfile   # pgvector:pg18 + AGE 1.7.0
│   ├── compose.yml           # full stack
│   └── compose.dev.yml       # postgres only
├── scripts/smoke.sh          # end-to-end smoke test
└── .env.example
```

## Quick start

### 1. Bring up Postgres

```bash
docker compose -f deploy/compose.dev.yml up -d
# or: podman compose -f deploy/compose.dev.yml up -d
```

The custom image is built on first run (PG 18 + pgvector + AGE) and takes about
a minute. Subsequent starts are instant.

### 2. Configure env

```bash
cp .env.example .env
# Generate a master password hash for the web UI:
cargo run -p plutus-server -- hash-password
# Paste the printed hash into PLUTUS_MASTER_PASSWORD_HASH=
# Generate a cookie secret:
openssl rand -hex 32   # paste into PLUTUS_COOKIE_SECRET=
```

### 3. Migrate + seed

```bash
cargo run -p plutus-server -- migrate
```

This creates 17 tables, seeds currencies (USD/HKD/CNY/SGD/EUR/GBP/JPY),
markets (XNAS/XNYS/XHKG/XSHG/XSHE), and brokers (IBKR/MOOMOO_US/FSMONE).

### 4. Run the API

```bash
cargo run -p plutus-server -- serve
# → listening on http://127.0.0.1:8080
```

### 5. Run the web UI (separate terminal)

```bash
cd web
pnpm install   # first time only
PORT=4100 PLUTUS_API_URL=http://127.0.0.1:8080 pnpm dev
# → http://localhost:4100
```

### 6. Smoke test

```bash
bash scripts/smoke.sh
```

## API surface

Mounted under `/api/v1`. OpenAPI spec at `/api/v1/openapi.json`.

| Group | Endpoints |
|---|---|
| Meta | `GET /healthz`, `GET /openapi.json` |
| Auth | `POST /auth/login`, `POST /auth/logout`, `GET /auth/me` |
| Tokens (web only) | `GET /tokens`, `POST /tokens`, `DELETE /tokens/:id` |
| Markets / brokers / accounts | `GET /markets`, `GET /brokers`, `GET /accounts`, `POST /accounts`, `GET /accounts/:id` |
| Stocks | `GET/POST /stocks`, `GET/DELETE /stocks/:id`, `GET /stocks/:id/translations`, `PUT /stocks/:id/translations/:locale`, `GET/POST /stocks/:id/ohlcv` |
| Watchlists | `GET/POST /watchlists`, `GET/DELETE /watchlists/:id`, `GET/POST /watchlists/:id/items`, `DELETE /watchlists/:id/items/:stock_id` |
| Transactions | `GET/POST /transactions`, `GET/DELETE /transactions/:id` |
| Holdings | `GET /holdings?method=fifo|lifo|average&account_id=…` |
| FX | `GET/POST /fx` |
| Audit | `GET /audit` |

### Auth

Two modes, set via `PLUTUS_API_REQUIRE_AUTH`:

- `false` (default): every endpoint accepts anonymous requests.
  Use this combined with `PLUTUS_BIND_ADDR=127.0.0.1:8080` (the default) so
  exposure is limited to localhost. Convenient for the agent on the same host.
- `true`: requests must carry either a session cookie (issued by
  `POST /auth/login` with the master password) or a `Authorization: Bearer …`
  header pointing at an unrevoked API token.

API tokens are created from the web UI (or by hitting `POST /tokens` while
authenticated as `web`). The plaintext token is returned exactly once.

### Idempotency

The `idempotency_keys` table is in place. Wiring of the middleware that reads
`Idempotency-Key` headers is a small follow-up; the data model already
supports the Stripe-style replay semantics.

### Audit

The `audit_log` table is in place. Handlers can already call
`plutus_storage::queries::audit::record(...)` to write entries; only a few
write paths currently emit audit rows. Wiring this through every endpoint is a
small follow-up that does not affect the schema.

## Multi-market support

A single `markets` reference table holds per-market metadata (timezone,
currency, default lot size, settlement days). Stocks reference it via
`market_code`. No per-market table splitting. Filters in URLs use a single
`market=XNAS,XHKG` query parameter that the web UI persists across pages.

Phase 0 seeds: `XNAS`, `XNYS`, `XHKG`, `XSHG`, `XSHE`. Adding more is one
seed-table row.

## Cost basis

Holdings are derived from transactions on every read. Three methods, picked
per query:

- `?method=fifo` (default): closes oldest open lots first.
- `?method=lifo`: closes newest open lots first.
- `?method=average`: weighted average across remaining lots.

Commission and FX rate are captured at execution time and baked into the
base-currency cost. Test coverage lives in
`crates/plutus-core/src/cost_basis.rs::tests`.

## Deferred for later phases

| Item | Phase |
|---|---|
| IPO / earnings / dividend calendars | 1 |
| Macro indicators (CPI, PPI, rates) | 1 |
| News / filings ingestion | 2 |
| Research notes, theses | 2 |
| Portfolio analytics (Sharpe, drawdown, benchmark) | 3 |
| Alerts and webhooks | 3 |
| pgvector and AGE usage (already installed) | 4 |

## Notes on the toolchain

- **Toasty 0.6.x** is alpha. Phase 0 only uses the features that exercise
  cleanly: derive macros, generated `get_by_<unique>`, `filter_by_<unique>`,
  the `toasty::create!` and `update()` macros, and `push_schema()`. We wrap
  the handle in `Arc<tokio::sync::Mutex<…>>` so axum can share it. When
  toasty's connection-pool semantics mature we can drop the mutex.
- **Remix 3 beta** is brand new (April 2026) and intentionally does not use
  React. Components are `function Foo() { return ({ props }) => <jsx /> }`
  closures, styling is via the `mix={css({…})}` prop. If the API stays in
  motion we may switch to React Router v7 framework mode; the API contract
  doesn't care.

## Repository conventions

- All written content (docs, comments, commits) is English.
- Each crate compiles independently — no implicit reach-throughs across the
  layering line.
- `plutus-storage` doesn't leak toasty types past `Db`; consumers use the
  `models` and `queries` modules.
