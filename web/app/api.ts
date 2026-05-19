// Tiny typed wrapper around the plutus REST API. The base URL comes from
// PLUTUS_API_URL (set by docker compose or .env). Defaults match the
// dev-mode setup where the Rust server binds 127.0.0.1:8080.

const BASE = process.env.PLUTUS_API_URL ?? 'http://127.0.0.1:8080'

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
  let headers: Record<string, string> = { accept: 'application/json' }
  if (cookie) headers.cookie = cookie
  let res = await fetch(`${BASE}/api/v1${path}`, { headers })
  if (!res.ok) {
    throw new Error(`plutus api ${path} failed: ${res.status}`)
  }
  return (await res.json()) as T
}

async function post<T>(path: string, body: unknown, cookie?: string | null): Promise<T> {
  let headers: Record<string, string> = {
    accept: 'application/json',
    'content-type': 'application/json',
  }
  if (cookie) headers.cookie = cookie
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

export interface StockTranslation {
  stock_id: number
  locale: string
  name: string
  description_md: string | null
  updated_at: string
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
  title: string
  summary: string | null
  content_md: string | null
  agent_summary_md: string | null
  language: string
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

export interface NewsTranslation {
  id: number
  news_id: number
  locale: string
  title: string
  summary: string | null
  content_md: string | null
  agent_summary_md: string | null
  translator: string
  updated_at: string
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
  headline: string
  summary_md: string | null
  content_md: string | null
  sentiment: string | null
  sentiment_score: string | null
  metrics: string | null
  notes: string | null
  language: string
  source: string
  created_at: string
  updated_at: string
}

export interface MacroEvent {
  id: number
  indicator_code: string
  event_date: string
  event_kind: string
  title: string
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
  translations: string | null
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
  headline: string
  content_md: string | null
  sentiment: string | null
  sentiment_score: string | null
  source: string
  language: string
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
  description_md: string | null
  summary_md: string | null
  sentiment: string | null
  language: string
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
  rationale_md: string | null
  metrics: string | null
  created_at: string
}

export interface PortfolioReview {
  id: number
  kind: string
  period_start: string
  period_end: string
  headline: string
  summary_md: string | null
  content_md: string | null
  decisions_md: string | null
  sentiment: string | null
  sentiment_score: string | null
  metrics: string | null
  language: string
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
  rationale_md: string
  target_price: string | null
  target_currency: string | null
  target_horizon: string
  issued_at: string
  status: string
  outcome_md: string | null
  pnl_pct: string | null
  closed_at: string | null
  language: string
  source: string
  created_at: string
  updated_at: string
}

export interface SelfExam {
  id: number
  kind: string
  period_start: string
  period_end: string
  headline: string
  content_md: string | null
  metrics: string | null
  recommendation_ids: string | null
  notes: string | null
  language: string
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

export interface Catalyst {
  id: number
  stock_id: number | null
  sector_code: string | null
  country: string | null
  catalyst_kind: string
  title: string
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
  stocks: () => get<Stock[]>('/stocks'),
  stock: (id: number) => get<Stock>(`/stocks/${id}`),
  stockTranslations: (id: number) =>
    get<StockTranslation[]>(`/stocks/${id}/translations`),
  createStock: (input: {
    market_code: string
    symbol: string
    currency: string
    asset_class: string
    isin?: string
    figi?: string
    lot_size?: number
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
  newsTranslations: (newsId: number) =>
    get<NewsTranslation[]>(`/news/${newsId}/translations`),
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
  /// `Set-Cookie` header and forward it to the browser. The JSON body shape
  /// is `{ ok: true }` on success or an `ApiError` body on 401.
  loginRaw: (password: string) =>
    fetch(`${BASE}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify({ password }),
    }),
  logoutRaw: (cookie?: string | null) => {
    let headers: Record<string, string> = { accept: 'application/json' }
    if (cookie) headers.cookie = cookie
    return fetch(`${BASE}/api/v1/auth/logout`, { method: 'POST', headers })
  },

  me: (cookie?: string | null) =>
    get<{ kind: string; label: string; token_id: number | null }>('/auth/me', cookie),
}
