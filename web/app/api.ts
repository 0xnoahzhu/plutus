// Tiny typed wrapper around the plutus REST API. The base URL comes from
// PLUTUS_API_URL (set by docker compose or .env). Defaults match the
// dev-mode setup where the Rust server binds 127.0.0.1:8080.

import { AsyncLocalStorage } from 'node:async_hooks'

const BASE = process.env.PLUTUS_API_URL ?? 'http://127.0.0.1:8080'

/// Per-request cookie context. Populated by `withAuth` (and any other
/// per-request wrapper) so every `api.*()` call inside the request
/// inherits the caller's session without the controller having to
/// thread the cookie through every call site explicitly.
///
/// Explicit `cookie` arguments on `api.X(...)` still win — this is just
/// the fallback when none is supplied.
const requestCookieStore = new AsyncLocalStorage<string | null>()

/// Run `fn` with `cookie` available to every `api.*()` call inside it,
/// including across `await` boundaries and `Promise.all` parallel calls.
export function runWithCookie<T>(cookie: string | null, fn: () => T): T {
  return requestCookieStore.run(cookie, fn)
}

/// Pull the cookie for the current request out of the ALS, falling back
/// to `null` when no `runWithCookie` is active (e.g. server boot).
function ambientCookie(): string | null {
  return requestCookieStore.getStore() ?? null
}

export interface Market {
  code: string
  name: string
  country: string
  timezone: string
  currency_code: string
  default_lot_size: number
  settlement_days: number
}

export interface Broker {
  id: number
  code: string
  name: string
}

export interface Account {
  id: number
  broker_id: number
  name: string
  account_number: string | null
  base_currency: string
  created_at: string
}

export interface Stock {
  id: number
  market_code: string
  symbol: string
  isin: string | null
  figi: string | null
  currency: string
  lot_size: number | null
  asset_class: string
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`). Null when neither the requested locale nor `en`
  /// has the field populated.
  name: string | null
  description_md: string | null
  created_at: string
  updated_at: string
}

export interface Transaction {
  id: number
  account_id: number
  stock_id: number | null
  kind: string
  executed_at: string
  quantity: string
  price: string
  trade_currency: string
  commission: string
  commission_currency: string
  tax: string
  tax_currency: string
  fx_rate_to_base: string
  external_ref: string | null
  notes: string | null
  source: string
  source_metadata: string | null
  created_at: string
  updated_at: string
}

export interface Holding {
  stock_id: number
  account_id: number | null
  quantity: string
  avg_cost_trade: string
  cost_base: string
  realized_pnl_base: string
}

/// Append `?locale=` to a path when the caller passed a non-default locale.
/// Default ('en' / undefined) returns the path unchanged.
function withLocale(path: string, locale?: string): string {
  if (!locale || locale === 'en') return path
  let sep = path.includes('?') ? '&' : '?'
  return `${path}${sep}locale=${encodeURIComponent(locale)}`
}

async function get<T>(path: string, cookie?: string | null): Promise<T> {
  let effective = cookie ?? ambientCookie()
  let headers: Record<string, string> = { accept: 'application/json' }
  if (effective) headers.cookie = effective
  let res = await fetch(`${BASE}/api/v1${path}`, { headers })
  if (!res.ok) {
    throw new Error(`plutus api ${path} failed: ${res.status}`)
  }
  return (await res.json()) as T
}

async function post<T>(path: string, body: unknown, cookie?: string | null): Promise<T> {
  let effective = cookie ?? ambientCookie()
  let headers: Record<string, string> = {
    accept: 'application/json',
    'content-type': 'application/json',
  }
  if (effective) headers.cookie = effective
  let res = await fetch(`${BASE}/api/v1${path}`, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  })
  if (!res.ok) {
    let text = await res.text()
    throw new Error(`plutus api ${path} failed: ${res.status} ${text}`)
  }
  return (await res.json()) as T
}

export interface WatchlistItem {
  id: number
  stock_id: number
  added_at: string
  notes: string | null
}

export interface NewsItem {
  id: number
  external_id: string | null
  url: string
  canonical_url: string | null
  archive_url: string | null
  url_status: number | null
  last_verified_at: string | null
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`). Null when neither the requested locale nor `en`
  /// has the field populated.
  title: string | null
  summary: string | null
  content_md: string | null
  agent_summary_md: string | null
  source: string
  source_kind: string
  category: string
  region: string
  published_at: string
  fetched_at: string
  sentiment: string | null
  sentiment_score: string | null
  importance: string
  created_at: string
  updated_at: string
}

export interface NewsStockLink {
  id: number
  news_id: number
  stock_id: number
  relation: string
  relevance: string | null
}

export interface NewsSectorLink {
  id: number
  news_id: number
  sector_code: string
}

export interface NewsMacroLink {
  id: number
  news_id: number
  indicator_code: string
}

export interface NewsCountryLink {
  id: number
  news_id: number
  country: string
}

export interface Sector {
  code: string
  name: string
  parent_code: string | null
  scheme: string
}

export interface WatchlistReport {
  id: number
  kind: string
  period_start: string
  period_end: string
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`). Null when neither the requested locale nor `en`
  /// has the field populated.
  headline: string | null
  summary_md: string | null
  content_md: string | null
  notes: string | null
  sentiment: string | null
  sentiment_score: string | null
  metrics: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface MacroEvent {
  id: number
  indicator_code: string
  event_date: string
  event_kind: string
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`). Null when neither the requested locale nor `en`
  /// has the field populated.
  title: string | null
  summary_md: string | null
  decision: string | null
  decision_bps: number | null
  new_value: string | null
  consensus_estimate: string | null
  surprise: string | null
  previous_value: string | null
  vote: string | null
  dot_plot: string | null
  url: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface MacroIndicator {
  code: string
  name: string
  country: string
  unit: string
  frequency: string
  source: string
  description: string | null
}

export interface EarningsEvent {
  id: number
  stock_id: number
  fiscal_year: number
  fiscal_period: string
  announce_at: string | null
  announce_date: string
  announce_timing: string
  status: string
  eps_estimate: string | null
  eps_actual: string | null
  revenue_estimate: string | null
  revenue_actual: string | null
  currency: string | null
  guidance_md: string | null
  notes: string | null
  url: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface MarketBrief {
  id: number
  country: string
  kind: string
  trade_date: string
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`). Null when neither the requested locale nor `en`
  /// has the field populated.
  headline: string | null
  content_md: string | null
  sentiment: string | null
  sentiment_score: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface ScreenerRun {
  id: number
  name: string
  kind: string
  run_date: string
  universe: string
  universe_size: number | null
  criteria: string | null
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`).
  description_md: string | null
  summary_md: string | null
  sentiment: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface ScreenerHit {
  id: number
  run_id: number
  stock_id: number
  rank: number | null
  score: string | null
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`).
  rationale_md: string | null
  metrics: string | null
  created_at: string
}

export interface PortfolioReview {
  id: number
  kind: string
  period_start: string
  period_end: string
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`).
  headline: string | null
  summary_md: string | null
  content_md: string | null
  decisions_md: string | null
  sentiment: string | null
  sentiment_score: string | null
  metrics: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface Recommendation {
  id: number
  stock_id: number | null
  sector_code: string | null
  action: string
  confidence: string | null
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`).
  rationale_md: string | null
  target_price: string | null
  target_currency: string | null
  target_horizon: string
  issued_at: string
  status: string
  outcome_md: string | null
  pnl_pct: string | null
  closed_at: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface SelfExam {
  id: number
  kind: string
  period_start: string
  period_end: string
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`).
  headline: string | null
  content_md: string | null
  metrics: string | null
  recommendation_ids: string | null
  notes: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface UniverseDefinition {
  id: number
  name: string
  description_md: string | null
  /// JSON-encoded array of stock ids, e.g. "[1,2,3]".
  stock_ids: string
  created_at: string
  updated_at: string
}

export interface CorrelationRun {
  id: number
  kind: string
  run_date: string
  universe_id: number
  lookback_days: number
  method: string
  summary_md: string | null
  metrics: string | null
  source: string
  created_at: string
  updated_at: string
}

export interface CorrelationPair {
  id: number
  run_id: number
  stock_a_id: number
  stock_b_id: number
  correlation: string
}

export interface TradePlan {
  id: number
  stock_id: number
  rationale: string | null
  status: string
  created_at: string
  updated_at: string
}

export interface TradePlanLevel {
  id: number
  plan_id: number
  stock_id: number
  kind: string
  price: string
  quantity: string | null
  fraction_pct: string | null
  status: string
  triggered_at: string | null
  notes: string | null
  sort_order: number | null
  created_at: string
  updated_at: string
}

/// One pending limit order placed with the broker. Standalone — no
/// parent header, optional back-pointer to a trade-plan level.
export interface PendingOrder {
  id: number
  account_id: number
  stock_id: number
  trade_plan_level_id: number | null
  /// 'buy' | 'sell'
  side: string
  /// 'limit' | 'stop' | 'stop_limit'
  order_type: string
  limit_price: string | null
  stop_price: string | null
  quantity: string
  /// 'gtc' | 'day' | 'gtd'
  time_in_force: string
  expires_at: string | null
  /// 'open' | 'filled' | 'cancelled' | 'expired'
  status: string
  placed_at: string
  filled_at: string | null
  cancelled_at: string | null
  broker_order_ref: string | null
  notes: string | null
  created_at: string
  updated_at: string
}

export interface Catalyst {
  id: number
  stock_id: number | null
  sector_code: string | null
  country: string | null
  catalyst_kind: string
  /// Already projected for the request locale by the storage layer (with
  /// fallback to `en`).
  title: string | null
  summary_md: string | null
  catalyst_date: string
  date_confidence: string
  impact_level: string
  bull_case_md: string | null
  bear_case_md: string | null
  status: string
  notes: string | null
  url: string | null
  source: string
  created_at: string
  updated_at: string
}

export const api = {
  base: BASE,
  health: () => get<string>('/healthz').catch(() => 'down'),
  markets: () => get<Market[]>('/markets'),
  brokers: () => get<Broker[]>('/brokers'),
  accounts: () => get<Account[]>('/accounts'),
  stocks: (locale?: string) => get<Stock[]>(withLocale('/stocks', locale)),
  stock: (id: number, locale?: string) =>
    get<Stock>(withLocale(`/stocks/${id}`, locale)),
  createStock: (input: {
    market_code: string
    symbol: string
    currency: string
    asset_class: string
    isin?: string
    figi?: string
    lot_size?: number
    content: unknown
  }) => post<Stock>('/stocks', input),
  holdings: (params: { market?: string[]; method?: string } = {}) => {
    let q = new URLSearchParams()
    if (params.method) q.set('method', params.method)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<Holding[]>(`/holdings${suffix}`)
  },
  transactions: () => get<Transaction[]>('/transactions'),
  watchlistItems: (country?: string) => {
    let suffix = country ? `?country=${country}` : ''
    return get<WatchlistItem[]>(`/watchlist/items${suffix}`)
  },
  news: (locale?: string) => get<NewsItem[]>(withLocale('/news', locale)),
  newsItem: (id: number, locale?: string) =>
    get<NewsItem>(withLocale(`/news/${id}`, locale)),
  newsStockLinks: (newsId: number) =>
    get<NewsStockLink[]>(`/news/${newsId}/stock-links`),
  newsSectorLinks: (newsId: number) =>
    get<NewsSectorLink[]>(`/news/${newsId}/sector-links`),
  newsMacroLinks: (newsId: number) =>
    get<NewsMacroLink[]>(`/news/${newsId}/macro-links`),
  newsCountryLinks: (newsId: number) =>
    get<NewsCountryLink[]>(`/news/${newsId}/country-links`),
  newsForStock: (stockId: number) =>
    get<NewsStockLink[]>(`/stocks/${stockId}/news`),
  sectors: () => get<Sector[]>('/sectors'),
  marketBriefs: (country?: string, locale?: string) => {
    let q = new URLSearchParams()
    if (country) q.set('country', country)
    if (locale) q.set('locale', locale)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<MarketBrief[]>(`/market-briefs${suffix}`)
  },
  earnings: (country?: string) => {
    let suffix = country ? `?country=${country}` : ''
    return get<EarningsEvent[]>(`/earnings${suffix}`)
  },
  macroEvents: (country?: string, locale?: string) => {
    let q = new URLSearchParams()
    if (country) q.set('country', country)
    if (locale) q.set('locale', locale)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<MacroEvent[]>(`/macro/events${suffix}`)
  },
  macroIndicators: () => get<MacroIndicator[]>('/macro/indicators'),
  watchlistReports: (params: { kind?: string; locale?: string } = {}) => {
    let q = new URLSearchParams()
    if (params.kind) q.set('kind', params.kind)
    if (params.locale) q.set('locale', params.locale)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<WatchlistReport[]>(`/watchlist/reports${suffix}`)
  },
  earningsForStock: (stockId: number) =>
    get<EarningsEvent[]>(`/stocks/${stockId}/earnings`),

  screenerRuns: (locale?: string) =>
    get<ScreenerRun[]>(withLocale('/screener-runs', locale)),
  screenerRun: (id: number, locale?: string) =>
    get<ScreenerRun>(withLocale(`/screener-runs/${id}`, locale)),
  screenerHits: (runId: number, locale?: string) =>
    get<ScreenerHit[]>(withLocale(`/screener-runs/${runId}/hits`, locale)),
  screenerHitsForStock: (stockId: number, locale?: string) =>
    get<ScreenerHit[]>(withLocale(`/stocks/${stockId}/screener-hits`, locale)),

  portfolioReviews: (locale?: string) =>
    get<PortfolioReview[]>(withLocale('/portfolio-reviews', locale)),
  portfolioReview: (id: number, locale?: string) =>
    get<PortfolioReview>(withLocale(`/portfolio-reviews/${id}`, locale)),

  recommendations: (params: { status?: string; stock_id?: number; locale?: string } = {}) => {
    let q = new URLSearchParams()
    if (params.status) q.set('status', params.status)
    if (params.stock_id !== undefined) q.set('stock_id', String(params.stock_id))
    if (params.locale) q.set('locale', params.locale)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<Recommendation[]>(`/recommendations${suffix}`)
  },
  recommendation: (id: number, locale?: string) =>
    get<Recommendation>(withLocale(`/recommendations/${id}`, locale)),
  recommendationsForStock: (stockId: number, locale?: string) =>
    get<Recommendation[]>(withLocale(`/stocks/${stockId}/recommendations`, locale)),

  selfExams: (params: { kind?: string; locale?: string } = {}) => {
    let q = new URLSearchParams()
    if (params.kind) q.set('kind', params.kind)
    if (params.locale) q.set('locale', params.locale)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<SelfExam[]>(`/self-exams${suffix}`)
  },
  selfExam: (id: number, locale?: string) =>
    get<SelfExam>(withLocale(`/self-exams/${id}`, locale)),

  universes: () => get<UniverseDefinition[]>('/universes'),
  universe: (id: number) => get<UniverseDefinition>(`/universes/${id}`),

  correlationRuns: (locale?: string) =>
    get<CorrelationRun[]>(withLocale('/correlation-runs', locale)),
  correlationRun: (id: number, locale?: string) =>
    get<CorrelationRun>(withLocale(`/correlation-runs/${id}`, locale)),
  correlationPairs: (runId: number) =>
    get<CorrelationPair[]>(`/correlation-runs/${runId}/pairs`),
  correlationPairsForStock: (stockId: number) =>
    get<CorrelationPair[]>(`/stocks/${stockId}/correlation-pairs`),

  catalysts: (params: { country?: string; status?: string; locale?: string } = {}) => {
    let q = new URLSearchParams()
    if (params.country) q.set('country', params.country)
    if (params.status) q.set('status', params.status)
    if (params.locale) q.set('locale', params.locale)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<Catalyst[]>(`/catalysts${suffix}`)
  },
  catalyst: (id: number, locale?: string) =>
    get<Catalyst>(withLocale(`/catalysts/${id}`, locale)),
  catalystsForStock: (stockId: number, locale?: string) =>
    get<Catalyst[]>(withLocale(`/stocks/${stockId}/catalysts`, locale)),

  audit: () => get<unknown[]>('/audit'),

  /// Returns the raw upstream Response so the caller can read the
  /// `Set-Cookie` header AND the JSON body (to check `password_reset_required`),
  /// then forward both back to the browser.
  loginRaw: (username: string, password: string) =>
    fetch(`${BASE}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify({ username, password }),
    }),
  logoutRaw: (cookie?: string | null) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/auth/logout`, { method: 'POST', headers })
  },

  /// Self-service password change. Used by the forced-reset flow as well —
  /// when `password_reset_required` is set, `current_password` is ignored by
  /// the server (the admin-issued temp hash is already invalidated).
  changePasswordRaw: (
    cookie: string | null | undefined,
    body: { current_password: string; new_password: string; new_password_confirm: string },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/auth/change-password`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })
  },

  me: (cookie?: string | null) =>
    get<{
      kind: string
      label: string
      user_id: number | null
      token_id: number | null
      is_admin: boolean
    }>('/auth/me', cookie),

  // ── Admin (admin-only — all return 403 to regular users) ───────────────
  adminListUsers: (cookie?: string | null) =>
    get<
      Array<{
        id: number
        username: string
        password_reset_required: boolean
        created_at: string
        updated_at: string
      }>
    >('/admin/users', cookie),

  adminCreateUserRaw: (
    cookie: string | null | undefined,
    body: { username: string; password: string },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/admin/users`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })
  },

  adminResetUserPasswordRaw: (
    cookie: string | null | undefined,
    id: number,
    password: string,
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/admin/users/${id}/reset-password`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ password }),
    })
  },

  adminDeleteUserRaw: (cookie: string | null | undefined, id: number) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/admin/users/${id}`, { method: 'DELETE', headers })
  },

  // ── API tokens (regular user — manage own keys) ────────────────────────
  tokens: () =>
    get<
      Array<{
        id: number
        label: string
        created_at: string
        last_used_at: string | null
      }>
    >('/tokens'),

  /// Returns the freshly minted plain token in the body — visible only this
  /// once (the server stores a SHA-256 hash). Caller is responsible for
  /// surfacing it to the user with a copy hint.
  createTokenRaw: (cookie: string | null | undefined, label: string) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/tokens`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ label }),
    })
  },

  /// Hard delete — the row goes away, the hash no longer resolves to a
  /// user, any bearer request still carrying the plaintext starts getting
  /// 401.
  deleteTokenRaw: (cookie: string | null | undefined, id: number) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/tokens/${id}`, { method: 'DELETE', headers })
  },

  // ── Accounts (regular user — manage own broker accounts) ───────────────
  createAccountRaw: (
    cookie: string | null | undefined,
    input: {
      broker_id: number
      name: string
      account_number: string | null
      base_currency: string
    },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/accounts`, {
      method: 'POST',
      headers,
      body: JSON.stringify(input),
    })
  },

  deleteAccountRaw: (cookie: string | null | undefined, id: number) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/accounts/${id}`, { method: 'DELETE', headers })
  },

  // ── Admin brokers (admin only — manage the shared brokers table) ──────
  adminListBrokers: (cookie?: string | null) => get<Broker[]>('/admin/brokers', cookie),

  adminCreateBrokerRaw: (
    cookie: string | null | undefined,
    body: { code: string; name: string },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/admin/brokers`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })
  },

  adminUpdateBrokerRaw: (
    cookie: string | null | undefined,
    id: number,
    name: string,
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/admin/brokers/${id}`, {
      method: 'PATCH',
      headers,
      body: JSON.stringify({ name }),
    })
  },

  adminDeleteBrokerRaw: (cookie: string | null | undefined, id: number) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/admin/brokers/${id}`, { method: 'DELETE', headers })
  },

  // ── Admin tokens (admin only — admin-grade bearer tokens) ─────────────
  adminListTokens: (cookie?: string | null) =>
    get<
      Array<{
        id: number
        label: string
        created_at: string
        last_used_at: string | null
      }>
    >('/admin/tokens', cookie),

  adminCreateTokenRaw: (cookie: string | null | undefined, label: string) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/admin/tokens`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ label }),
    })
  },

  adminDeleteTokenRaw: (cookie: string | null | undefined, id: number) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/admin/tokens/${id}`, { method: 'DELETE', headers })
  },

  // ── Trade plans (regular user — manage own trade plans + levels) ──────
  tradePlans: (params: { stock_id?: number; status?: string } = {}) => {
    let q = new URLSearchParams()
    if (params.stock_id !== undefined) q.set('stock_id', String(params.stock_id))
    if (params.status) q.set('status', params.status)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<TradePlan[]>(`/trade-plans${suffix}`)
  },

  tradePlanLevels: (planId: number) =>
    get<TradePlanLevel[]>(`/trade-plans/${planId}/levels`),

  createTradePlanRaw: (
    cookie: string | null | undefined,
    body: { stock_id: number; rationale: string | null },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/trade-plans`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })
  },

  updateTradePlanRaw: (
    cookie: string | null | undefined,
    id: number,
    body: { status?: string; rationale?: string | null },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/trade-plans/${id}`, {
      method: 'PATCH',
      headers,
      body: JSON.stringify(body),
    })
  },

  deleteTradePlanRaw: (cookie: string | null | undefined, id: number) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/trade-plans/${id}`, { method: 'DELETE', headers })
  },

  addTradePlanLevelRaw: (
    cookie: string | null | undefined,
    planId: number,
    body: {
      kind: string
      price: string
      quantity?: string | null
      fraction_pct?: string | null
      notes?: string | null
      sort_order?: number | null
    },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/trade-plans/${planId}/levels`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })
  },

  updateTradePlanLevelRaw: (
    cookie: string | null | undefined,
    levelId: number,
    body: {
      kind?: string
      price?: string
      quantity?: string | null
      fraction_pct?: string | null
      notes?: string | null
      sort_order?: number | null
      status?: string
    },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/trade-plans/levels/${levelId}`, {
      method: 'PATCH',
      headers,
      body: JSON.stringify(body),
    })
  },

  deleteTradePlanLevelRaw: (cookie: string | null | undefined, levelId: number) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/trade-plans/levels/${levelId}`, {
      method: 'DELETE',
      headers,
    })
  },

  // ── Pending orders (limit orders live at the broker) ─────────────────
  pendingOrders: (
    params: { account_id?: number; stock_id?: number; status?: string } = {},
  ) => {
    let q = new URLSearchParams()
    if (params.account_id !== undefined) q.set('account_id', String(params.account_id))
    if (params.stock_id !== undefined) q.set('stock_id', String(params.stock_id))
    if (params.status) q.set('status', params.status)
    let suffix = q.toString() ? `?${q.toString()}` : ''
    return get<PendingOrder[]>(`/pending-orders${suffix}`)
  },

  createPendingOrderRaw: (
    cookie: string | null | undefined,
    body: {
      account_id: number
      stock_id: number
      trade_plan_level_id?: number | null
      side: string
      order_type: string
      limit_price?: string | null
      stop_price?: string | null
      quantity: string
      time_in_force?: string
      expires_at?: string | null
      broker_order_ref?: string | null
      notes?: string | null
      placed_at?: string | null
    },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/pending-orders`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })
  },

  updatePendingOrderRaw: (
    cookie: string | null | undefined,
    id: number,
    body: {
      account_id?: number
      side?: string
      order_type?: string
      limit_price?: string | null
      stop_price?: string | null
      quantity?: string
      time_in_force?: string
      expires_at?: string | null
      broker_order_ref?: string | null
      notes?: string | null
      status?: string
    },
  ) => {
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      accept: 'application/json',
    }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/pending-orders/${id}`, {
      method: 'PATCH',
      headers,
      body: JSON.stringify(body),
    })
  },

  deletePendingOrderRaw: (cookie: string | null | undefined, id: number) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/pending-orders/${id}`, { method: 'DELETE', headers })
  },
}
