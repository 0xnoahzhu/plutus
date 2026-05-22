# plutus

Personal investment data store. The hermes AI agent writes data here through a
REST API; the web UI is the human-side viewer.

Currently in active use as a single-user app, with multi-user plumbing in
place: per-user data isolation, admin-managed accounts, API tokens, and
session cookies. Translatable content (en + zh-CN) on every agent-writable
row. The web UI is fully bilingual.

## Stack

- **Backend**: Rust 1.95, axum 0.7, toasty 0.6.1 (tokio-rs ORM) with raw
  `tokio-postgres` for queries toasty doesn't model (JSONB, multi-column
  unique constraints, dynamic SET clauses).
- **Database**: PostgreSQL 18 + pgvector 0.8 + Apache AGE 1.7 (extensions
  installed; pgvector and AGE not currently in active use).
- **Frontend**: Remix 3 beta (server-rendered, not React) + `@remix-run/ui`
  headless primitives + `lucide-static` icons.
- **Deployment**: Podman Quadlet on a remote host, built via `scripts/deploy.sh`.

Workspace layout:

```
plutus/
├── Cargo.toml                # workspace root
├── crates/
│   ├── plutus-core/          # types, errors, cost-basis algorithms
│   ├── plutus-storage/       # toasty models + queries + post-migrate SQL
│   ├── plutus-api/           # axum router, DTOs, middleware (audit, auth)
│   └── plutus-server/        # binary: clap subcommands (serve / migrate)
├── web/                      # Remix 3 frontend
│   ├── app/controllers/      # one .tsx per page
│   ├── app/ui/               # shared components (pagination, markdown, …)
│   └── app/i18n/messages.ts  # en + zh-CN string table
├── deploy/
│   ├── postgres/Dockerfile   # pgvector:pg18 + AGE 1.7
│   ├── compose.yml           # full stack
│   └── compose.dev.yml       # postgres only
├── scripts/
│   ├── deploy.sh             # rsync + rebuild + restart on remote host
│   └── smoke.sh              # end-to-end smoke test
└── .env.example
```

## Quick start

### 1. Bring up Postgres

```bash
docker compose -f deploy/compose.dev.yml up -d
# or: podman compose -f deploy/compose.dev.yml up -d
```

The custom image is built on first run (PG 18 + pgvector + AGE) and takes
about a minute. Subsequent starts are instant.

### 2. Configure env

```bash
cp .env.example .env
# Set PLUTUS_ADMIN_USERNAME and PLUTUS_ADMIN_PASSWORD if you want the
# admin login path; leave blank to disable admin entirely.
# Regular user accounts are created via the admin UI and live in the
# database with Argon2-hashed passwords.
```

### 3. Migrate

```bash
cargo run -p plutus-server -- migrate
```

This brings up the schema (~44 tables on a fresh DB), runs the post-migrate
block in `crates/plutus-storage/src/db.rs` (ALTER TABLEs, UNIQUE indexes,
FK constraints, seed data), and seeds reference data: currencies, markets
(XNAS / XNYS / XHKG / XSHG / XSHE), brokers, the GICS sector taxonomy.

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

Mounted under `/api/v1`. Full OpenAPI spec at `/api/v1/openapi.json`
(97+ paths); Swagger UI at `/api/v1/docs`.

High-level grouping:

| Group | Endpoints |
|---|---|
| Meta | `GET /healthz`, `GET /openapi.json`, `GET /docs` |
| Auth | `POST /auth/login`, `POST /auth/logout`, `GET /auth/me`, `POST /auth/change-password` |
| Admin (env-auth) | `GET/POST /admin/users`, `POST /admin/users/:id/reset-password`, `GET/POST /admin/brokers`, `GET /admin/tokens` |
| API tokens | `GET/POST /tokens`, `DELETE /tokens/:id` |
| Reference data | `GET /markets`, `GET /brokers`, `GET /sectors`, `GET /fx`, `POST /fx` |
| Stocks | `GET/POST /stocks`, `GET/PATCH/DELETE /stocks/:id`, `GET/POST /ohlcv` |
| Watchlist | `GET/POST /watchlist/items`, `DELETE /watchlist/items/:id`, `GET/POST /watchlist/reports` |
| Transactions / accounts | `GET/POST /transactions`, `GET/DELETE /transactions/:id`, `GET/POST /accounts`, `GET /accounts/:id` |
| Holdings | `GET /holdings?method=fifo|lifo|average&country=&q=&page=…` |
| Trade plans / orders | `GET/POST /trade-plans`, `POST /trade-plans/:id/{trigger,reset,cancel,close,delete}`, `GET/POST /pending-orders` |
| Recommendations | `GET/POST /recommendations`, `PATCH /recommendations/:id/close` |
| Catalysts / earnings / macro | `GET/POST /catalysts`, `GET/POST /earnings`, `GET/POST /macro/{events,indicators,observations}` |
| Screeners / correlations | `GET/POST /screener-runs`, `POST /screener-runs/:id/hits`, `GET/POST /correlation-runs`, `GET/POST /universes` |
| News | `GET/POST /news`, `GET /news/:id`, `POST /news/:id/{stock,sector,macro,country}-links` |
| Briefs / reviews | `GET/POST /market-briefs`, `GET/POST /portfolio-reviews`, `GET/POST /self-exams` |
| Portfolio rollups | `GET /portfolio/value-series?days=…` |
| Audit | `GET /audit` |

### Auth model

Three actor kinds, controlled by `PLUTUS_API_REQUIRE_AUTH`:

- **`false` (default)** — endpoints accept anonymous requests. Use with
  `PLUTUS_BIND_ADDR=127.0.0.1:8080` (default) so exposure is limited to
  localhost. Convenient for the agent on the same host.
- **`true`** — requests must carry either a `plutus_session` cookie
  (issued by `POST /auth/login`) or `Authorization: Bearer <token>`.

User taxonomy:

- **Admin** — env-only (`PLUTUS_ADMIN_USERNAME` / `PLUTUS_ADMIN_PASSWORD`).
  Not a row in `users`. Manages regular users via `/admin/*`. A sentinel
  `__admin` row at `id=0` exists in `users` solely so audit FKs resolve;
  it can never validate via login.
- **Regular user** — row in `users`, Argon2-hashed password. Each has an
  `allowed_countries` allowlist (subset of `{US, HK, CN}`); country-scoped
  list endpoints honor it.
- **API token** — long-lived bearer token minted via `POST /tokens`,
  acts as the user it was minted under. Plaintext is shown once at
  creation alongside the hash.

### Per-user isolation

Every user-data table carries `user_id BIGINT NOT NULL` with a
`FOREIGN KEY ... REFERENCES users(id) ON DELETE CASCADE`. The list of
isolated entities (catalysts, screener_runs, portfolio_reviews,
recommendations, self_exams, correlation_runs, universe_definitions,
trade_plans, pending_orders, transactions, accounts, watchlist_items,
watchlist_reports, market_briefs, web_sessions, api_tokens, audit_log)
filters reads by the authenticated caller's id.

`user_id = 0` is a sentinel meaning "admin or system actor" — admin
logins create web_sessions at id=0, and the audit middleware defaults
to 0 when the actor has no user (admin, anonymous, system). Reference
data (`stocks`, `markets`, `brokers`, `sectors`, `news_items`, `macro_*`,
`ohlcv_daily`, `filings`, etc.) is shared.

### Translatable content

Every entity with human-readable text uses a JSONB `content` column
shaped as:

```json
{ "<locale>": { "title": "...", "summary_md": "..." } }
```

POST/PATCH bodies always include the full multi-locale blob; the server
stores it verbatim. List/get endpoints accept `?locale=en` (or `zh-CN`)
and return the localized fields flattened to top-level (`title`,
`summary_md`, `headline`, `rationale_md`, …). Falls back to `en` when
the requested locale's keys are missing.

### Idempotent writes

Several entities have a natural unique key and accept upserts:

| Entity | Conflict key | Endpoint |
|---|---|---|
| `catalysts` | `(user_id, catalyst_kind, catalyst_date, stock_id, sector_code, country, source)` | `POST /catalysts`, `POST /catalysts/batch` |
| `earnings_events` | `(stock_id, fiscal_year, fiscal_period)` | `POST /earnings`, `POST /earnings/batch` |
| `macro_events` | `(indicator_code, event_date)` | `POST /macro/events`, `POST /macro/events/batch` |
| `macro_observations` | `(indicator_code, obs_date)` | `POST /macro/observations` |
| `ohlcv_daily` | `(stock_id, trade_date)` | `POST /ohlcv/batch` |
| `screener_runs` | `(user_id, name, kind, run_date)` | `POST /screener-runs` |
| `portfolio_reviews` | `(user_id, kind, period_start)` | `POST /portfolio-reviews` |
| `self_exams` | `(user_id, kind, period_start)` | `POST /self-exams` |
| `watchlist_reports` | `(user_id, kind, period_start)` | `POST /watchlist/reports` |
| `market_briefs` | `(user_id, country, kind, trade_date)` | `POST /market-briefs` |
| `stocks` | `(market_code, symbol)` | `POST /stocks` returns 409 on collision |

Re-POSTing the same natural key refreshes the mutable fields and bumps
`updated_at`. Batch writes (`/<entity>/batch`) cap at 1000 items and run
the whole batch in one transaction.

### Audit

Every write through the API is recorded in `audit_log` by an axum
middleware. Entries carry `(user_id, entity_type, entity_id, action,
actor_kind, actor_id, actor_label, before, after, request_id)`. Read
via `GET /audit` — admin sees everything; users see their own writes.

## Multi-market support

A single `markets` reference table holds per-market metadata (timezone,
currency, default lot size, settlement days). Stocks reference it via
`market_code`. No per-market table splitting. Country filters in URLs use
a single `?country=US` (one country at a time) that the web UI persists
across pages.

Seeded markets: `XNAS`, `XNYS`, `XHKG`, `XSHG`, `XSHE`. Adding more is one
row in `markets`.

## Cost basis

Holdings are derived from transactions on every read. Three methods,
picked per query:

- `?method=fifo` (default): closes oldest open lots first.
- `?method=lifo`: closes newest open lots first.
- `?method=average`: weighted average across remaining lots.

Commission and FX rate are captured at execution time and baked into the
base-currency cost. Test coverage lives in
`crates/plutus-core/src/cost_basis.rs::tests`.

The holdings handler JOINs `stocks` so each row carries inlined
`symbol` / `market_code` / `currency`, and looks up the latest OHLCV
close to populate `market_value_base` and `unrealized_pnl_base`. Sorted
server-side by symbol so refresh order is stable.

## Pagination and search

Index pages (`/stocks`, `/holdings`, `/watchlists`, `/transactions`)
share a server-rendered pagination + search bar:

- `?page=N&per_page=15&q=<symbol>` on the backend.
- `X-Total-Count` header returned only when `?page` is set (no extra
  COUNT for agent bulk-fetch calls).
- 15 rows per page by default. Page-jump input (`[7] / 12 [Go]`)
  in the pagination bar; Enter or click jumps to the page.

Web rows are whole-row clickable via a document-level `data-row-href`
delegation: cmd/ctrl/shift + click opens in a new tab, clicks on
nested buttons / forms / inputs skip the row handler.

## Deployment

`scripts/deploy.sh` rsyncs to a Podman host (default `noah@10.1.2.51`,
override with `DEPLOY_HOST`), rebuilds the API or web image, restarts
the matching `systemctl --user` Quadlet service, and runs a smoke
probe on the public endpoint.

```bash
./scripts/deploy.sh                  # api + web
./scripts/deploy.sh --only api
./scripts/deploy.sh --only web
./scripts/deploy.sh --all            # also rebuild postgres
```

## Notes on the toolchain

- **Toasty 0.6.x** is alpha. We use what works cleanly (derive macros,
  `filter_by_<unique>`, `toasty::create!`, `.order_by`) and drop down
  to raw `tokio_postgres` for everything else (JSONB read/write,
  multi-column unique constraints, dynamic SET clauses, FK enforcement,
  ON CONFLICT upserts). The toasty handle is wrapped in
  `Arc<tokio::sync::Mutex<…>>` so axum can share it; can drop the mutex
  when toasty's pool semantics mature.
- **Remix 3 beta** is brand new (April 2026) and intentionally does not
  use React. Components are `function Foo() { return ({ props }) => <jsx /> }`
  closures, styling is via the `mix={css({…})}` prop. A small bridge
  (`web/app/ui/remix-theme.ts`) maps our token palette onto the
  `@remix-run/ui` CSS-variable contract so headless primitives style
  themselves without manual overrides.

## Repository conventions

- All written content (docs, comments, commits, identifiers) is English.
  User-facing UI strings are translated via `web/app/i18n/messages.ts`.
- Each crate compiles independently — no implicit reach-throughs across
  the layering line. `plutus-storage` doesn't leak toasty types past
  `Db`; consumers use the `models` and `queries` modules.
- All sorting / pagination / filtering happens server-side. The web
  layer doesn't re-sort or re-filter what the API returned.
- Backend handlers validate at the boundary (FK existence on
  `sector_code`, `indicator_code`; canonical-case normalization on
  `region`; etc.) and return clean 400 / 409 with actionable
  messages — the agent reads those and adjusts.
