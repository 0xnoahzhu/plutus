import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import {
  ambientPortfolioSummary,
  api,
  type AuditEntry,
  type DailyValue,
  type Holding,
  type Ohlcv,
  type Stock,
} from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  type BadgeTone,
  Card,
  color,
  EmptyState,
  font,
  Layout,
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  shadow,
  space,
  Stat,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { fmtMoney } from '../ui/format.ts'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

/// One row in the "Top Movers" section: a stock the user holds, its
/// last close, and the day-over-day %-change vs. the prior trading
/// day's close. `change_pct` is signed (positive = up).
interface Mover {
  stock_id: number
  symbol: string
  close: number
  prev_close: number
  change_pct: number
}

interface PortfolioSnapshot {
  /// Sum of `cost_base` across all holdings, in the user's base
  /// currency.
  cost_basis: number
  /// Sum of `quantity * latest_close * fx_rate` per holding. `fx_rate`
  /// stays 1 for now (everything's USD in the current data); the
  /// number will need refinement once cross-currency positions land.
  market_value: number
  /// `market_value - cost_basis`. Signed.
  unrealized: number
  /// Whether we managed to look up a close for every holding. When
  /// false the market value is partial; we surface that to the UI so
  /// users don't think their P&L cratered.
  fully_priced: boolean
}

/// Chart window presets, in the order they're offered.
///
/// `mtd` / `ytd` / `all` are why the API grew `from`/`to`: none of them
/// is a fixed number of trailing days, so `?days=N` can't express them.
const RANGE_PRESETS = ['7d', '30d', 'mtd', 'ytd', '1y', 'all'] as const
type RangePreset = (typeof RANGE_PRESETS)[number]

const DEFAULT_RANGE: RangePreset = '30d'

/// The resolved chart window plus what produced it, so the chip row can
/// show which option is live and the custom inputs can prefill.
interface ChartRange {
  from: string
  to: string
  /// `null` when the window came from explicit from/to rather than a
  /// preset — no chip is active in that case.
  preset: RangePreset | null
}

/// Shift an ISO `YYYY-MM-DD` by whole days, in UTC.
///
/// UTC throughout: the series dates the API returns are UTC calendar
/// days, so resolving the window in local time would slide the window
/// off the data by one day for anyone east or west of Greenwich.
function shiftDays(iso: string, delta: number): string {
  let d = new Date(`${iso}T00:00:00Z`)
  d.setUTCDate(d.getUTCDate() + delta)
  return d.toISOString().slice(0, 10)
}

/// Turn `?range=` / `?from=&to=` into a concrete window.
///
/// `earliest` is the first transaction date, used for `all`. It comes
/// from the transactions the dashboard already fetches for its counters,
/// so resolving "all" costs nothing extra.
function resolveRange(
  params: URLSearchParams,
  today: string,
  earliest: string | null,
): ChartRange {
  let from = (params.get('from') ?? '').trim()
  let to = (params.get('to') ?? '').trim()
  // An explicit range wins over a preset — it's the more specific ask.
  // Both ends required: a half-open custom range is almost always a
  // half-filled form, and guessing the other end would silently answer
  // a question the user didn't finish asking.
  if (isIsoDate(from) && isIsoDate(to) && from <= to) {
    return { from, to, preset: null }
  }

  let raw = (params.get('range') ?? '').trim().toLowerCase()
  let preset: RangePreset = (RANGE_PRESETS as readonly string[]).includes(raw)
    ? (raw as RangePreset)
    : DEFAULT_RANGE

  let start: string
  switch (preset) {
    case '7d':
      start = shiftDays(today, -6)
      break
    case '30d':
      start = shiftDays(today, -29)
      break
    case 'mtd':
      start = `${today.slice(0, 7)}-01`
      break
    case 'ytd':
      start = `${today.slice(0, 4)}-01-01`
      break
    case '1y':
      start = shiftDays(today, -364)
      break
    case 'all':
      // No transactions yet → fall back to the default window rather
      // than an empty range, so the card still renders its hint.
      start = earliest ?? shiftDays(today, -29)
      break
  }
  return { from: start, to: today, preset }
}

function isIsoDate(s: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(s) && !Number.isNaN(Date.parse(`${s}T00:00:00Z`))
}

export const home: BuildAction<'GET', typeof routes.home> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)

    // First wave — counts and the inputs we need for the second wave.
    let [
      markets,
      brokers,
      accounts,
      stocks,
      watchlistItems,
      transactions,
      holdings,
      plans,
      openOrders,
      auditEntries,
    ] = await Promise.all([
      api.markets().catch(() => []),
      api.brokers().catch(() => []),
      api.accounts().catch(() => []),
      api.stocks().catch(() => [] as Stock[]),
      api.watchlistItems().catch(() => []),
      api.transactions().catch(() => []),
      api.holdings().catch(() => [] as Holding[]),
      api.tradePlans({ status: 'active' }).catch(() => []),
      api.pendingOrders({ status: 'open' }).catch(() => []),
      api.audit().catch(() => [] as AuditEntry[]),
    ])

    // The chart window can't be resolved until the transactions are in:
    // `all` starts at the earliest one. Everything the second wave needs
    // is now known, so the series joins the OHLCV fetch below rather
    // than costing its own round trip.
    let today = new Date().toISOString().slice(0, 10)
    let earliest =
      transactions.length > 0
        ? transactions
            .map((t) => t.executed_at.slice(0, 10))
            .reduce((a, b) => (a < b ? a : b))
        : null
    let range = resolveRange(url.searchParams, today, earliest)

    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    // Top up the map with any held stocks not present in the default
    // /stocks listing (which is capped at 200). Without this, Top
    // Movers shows the stock_id placeholder for any holding whose
    // stock sorts past the catalog's first 200 tickers.
    let missingStockIds = holdings
      .map((h) => h.stock_id)
      .filter((id) => !stockMap.has(id))
    if (missingStockIds.length > 0) {
      let extras = await api.stocksByIds(missingStockIds).catch(() => [] as Stock[])
      for (let s of extras) stockMap.set(s.id, s)
    }

    // Second wave — OHLCV for every held stock, in parallel. Each call
    // returns the full history; we only need the last two bars so we
    // sort and slice client-side. With <50 holdings this is fine; if
    // the user accumulates a long tail we'd add a `?days=N` parameter
    // or a batched endpoint.
    let ohlcvByStock = new Map<number, Ohlcv[]>()
    let [valueSeries, ohlcvResults] = await Promise.all([
      api
        .portfolioValueSeries({ from: range.from, to: range.to })
        .catch(() => [] as DailyValue[]),
      Promise.all(
        holdings.map((h) => api.stockOhlcv(h.stock_id).catch(() => [] as Ohlcv[])),
      ),
    ])
    holdings.forEach((h, i) => {
      // Sort ascending so the "latest" is at the end.
      let series = [...(ohlcvResults[i] ?? [])].sort((a, b) =>
        a.trade_date.localeCompare(b.trade_date),
      )
      ohlcvByStock.set(h.stock_id, series)
    })

    let snapshot = buildSnapshot(holdings, ohlcvByStock)
    let movers = buildMovers(holdings, stockMap, ohlcvByStock)
    let recentActivity = auditEntries.slice(0, 8)

    let healthy = markets.length > 0
    return render(
      <DashboardPage
        healthy={healthy}
        locale={locale}
        theme={theme}
        counts={{
          markets: markets.length,
          brokers: brokers.length,
          accounts: accounts.length,
          stocks: stocks.length,
          watchlist: watchlistItems.length,
          transactions: transactions.length,
          holdings: holdings.length,
          tradePlans: plans.length,
          openOrders: openOrders.length,
        }}
        snapshot={snapshot}
        movers={movers}
        recentActivity={recentActivity}
        valueSeries={valueSeries}
        range={range}
        search={url.searchParams}
      />,
      request,
      { locale, theme },
    )
  },
}

/// Roll up holdings + latest OHLCV into one current-snapshot view.
/// We don't try to draw the time series yet (would need a per-day
/// rollup of every historical position); this gets the agent a real
/// number to look at instead of "chart not wired".
function buildSnapshot(
  holdings: Holding[],
  ohlcv: Map<number, Ohlcv[]>,
): PortfolioSnapshot {
  let cost_basis = 0
  let market_value = 0
  let fully_priced = holdings.length > 0
  for (let h of holdings) {
    let qty = Number.parseFloat(h.quantity)
    let cost = Number.parseFloat(h.cost_base)
    if (Number.isFinite(cost)) cost_basis += cost

    let series = ohlcv.get(h.stock_id) ?? []
    let last = series[series.length - 1]
    if (last) {
      let close = Number.parseFloat(last.adjusted_close ?? last.close)
      if (Number.isFinite(close) && Number.isFinite(qty)) {
        market_value += qty * close
        continue
      }
    }
    // Missing price → use cost basis as the placeholder so the number
    // isn't artificially low. Flag the row as not fully priced.
    if (Number.isFinite(cost)) market_value += cost
    fully_priced = false
  }
  return {
    cost_basis,
    market_value,
    unrealized: market_value - cost_basis,
    fully_priced,
  }
}

/// Compute day-over-day change for every stock the user holds, then
/// take the top movers by absolute %-change. Returns up to 5 rows.
/// Stocks with fewer than 2 bars of history are skipped — there's no
/// "previous close" to compare against.
function buildMovers(
  holdings: Holding[],
  stocks: Map<number, Stock>,
  ohlcv: Map<number, Ohlcv[]>,
): Mover[] {
  let rows: Mover[] = []
  for (let h of holdings) {
    let series = ohlcv.get(h.stock_id) ?? []
    if (series.length < 2) continue
    let last = series[series.length - 1]
    let prev = series[series.length - 2]
    let close = Number.parseFloat(last.adjusted_close ?? last.close)
    let prev_close = Number.parseFloat(prev.adjusted_close ?? prev.close)
    if (!Number.isFinite(close) || !Number.isFinite(prev_close) || prev_close === 0) {
      continue
    }
    let s = stocks.get(h.stock_id)
    rows.push({
      stock_id: h.stock_id,
      symbol: s?.symbol ?? `#${h.stock_id}`,
      close,
      prev_close,
      change_pct: ((close - prev_close) / prev_close) * 100,
    })
  }
  rows.sort((a, b) => Math.abs(b.change_pct) - Math.abs(a.change_pct))
  return rows.slice(0, 5)
}

interface DashboardProps {
  healthy: boolean
  locale: string
  theme: Theme
  counts: Record<string, number>
  snapshot: PortfolioSnapshot
  movers: Mover[]
  recentActivity: AuditEntry[]
  valueSeries: DailyValue[]
  range: ChartRange
  /// The incoming query string, so the range controls can rebuild the
  /// URL without dropping locale / theme / country.
  search: URLSearchParams
}

function DashboardPage() {
  return ({
    healthy,
    locale,
    theme,
    counts,
    snapshot,
    movers,
    recentActivity,
    valueSeries,
    range,
    search,
  }: DashboardProps) => {
    let p = messages(locale).pages.dashboard
    return (
      <Layout title={p.title} subtitle={p.subtitle} locale={locale} theme={theme}>
        <SectionTitle hint={p.quickStatsHint}>{p.sectionQuickStats}</SectionTitle>
        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
            gap: space[3],
            marginBottom: space[8],
          })}
        >
          <Stat
            label={p.statApiStatus}
            value={healthy ? 'up' : 'down'}
            trend={healthy ? 'up' : 'down'}
          />
          <Stat label={p.statMarkets} value={String(counts.markets)} caption={p.captionOpen} />
          <Stat label={p.statBrokers} value={String(counts.brokers)} caption={p.captionActive} />
          <Stat label={p.statAccounts} value={String(counts.accounts)} caption={p.captionTotal} />
          <Stat label={p.statStocks} value={String(counts.stocks)} caption={p.captionTracked} />
          <Stat label={p.statWatchlist} value={String(counts.watchlist)} caption={p.captionStocks} />
          <Stat label={p.statTransactions} value={String(counts.transactions)} caption={p.captionRecorded} />
          <Stat label={p.statOpenPositions} value={String(counts.holdings)} caption={p.captionCurrent} />
          <Stat label={p.statTradePlans} value={String(counts.tradePlans)} caption={p.captionActive} />
          <Stat label={p.statOpenOrders} value={String(counts.openOrders)} caption={p.captionAtBroker} />
        </div>

        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: '2fr 1fr',
            gap: space[5],
            '@media (max-width: 1000px)': { gridTemplateColumns: '1fr' },
          })}
        >
          <PortfolioSnapshotCard
            snapshot={snapshot}
            series={valueSeries}
            locale={locale}
            range={range}
            search={search}
          />

          <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[4] })}>
            <RecentActivityCard rows={recentActivity} locale={locale} />
            <TopMoversCard movers={movers} locale={locale} />
          </div>
        </div>

        <p
          mix={css({
            marginTop: space[8],
            fontSize: font.sm,
            color: color.textMuted,
          })}
        >
          {p.apiBaseLabel}: <code>{api.base}</code>
        </p>
      </Layout>
    )
  }
}

function PortfolioSnapshotCard() {
  return ({
    snapshot,
    series,
    locale,
    range,
    search,
  }: {
    snapshot: PortfolioSnapshot
    series: DailyValue[]
    locale: string
    range: ChartRange
    search: URLSearchParams
  }) => {
    let p = messages(locale).pages.dashboard
    if (snapshot.cost_basis === 0 && snapshot.market_value === 0) {
      return (
        <Card>
          <SectionTitle>{p.sectionPortfolioSnapshot}</SectionTitle>
          <EmptyState title={p.emptyNoPositions} hint={p.emptyPositionsHint} />
        </Card>
      )
    }
    let pnlPct =
      snapshot.cost_basis === 0
        ? 0
        : (snapshot.unrealized / snapshot.cost_basis) * 100
    let upTrend = snapshot.unrealized >= 0
    let tone: BadgeTone = upTrend ? 'success' : 'danger'
    return (
      <Card>
        {/* The hint used to read "30-day window" unconditionally, which
            stopped being true the moment the range became selectable.
            Report the span actually plotted. */}
        <SectionTitle
          hint={
            snapshot.fully_priced ? p.rangeSpan(series.length) : p.windowPartial
          }
        >
          {p.portfolioPerformance}
        </SectionTitle>
        <NetWorthStrip locale={locale} />
        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: space[4],
            marginBottom: space[4],
          })}
        >
          <Metric label={p.metricMarketValue} value={fmtMoney(snapshot.market_value)} />
          <Metric label={p.metricCostBasis} value={fmtMoney(snapshot.cost_basis)} />
          <Metric
            label={p.metricUnrealizedPnl}
            value={`${upTrend ? '+' : ''}${fmtMoney(snapshot.unrealized)}`}
            badge={
              <Badge tone={tone}>
                {upTrend ? '+' : ''}
                {pnlPct.toFixed(2)}%
              </Badge>
            }
          />
        </div>
        <PortfolioChart series={series} />
        <RangeControls locale={locale} range={range} search={search} />
      </Card>
    )
  }
}

/// Preset chips plus a custom from/to picker, under the chart.
///
/// Below rather than above: the chart is the answer, these are how you
/// change the question. Putting them on top pushes the number people
/// came for further down the card.
///
/// Both controls are plain links / a GET form — no client JS, matching
/// how every other filter in this app works, and leaving the window in
/// the URL so a range is shareable and survives a refresh.
function RangeControls() {
  return ({
    locale,
    range,
    search,
  }: {
    locale: string
    range: ChartRange
    search: URLSearchParams
  }) => {
    let p = messages(locale).pages.dashboard
    let labels: Record<RangePreset, string> = {
      '7d': p.range7d,
      '30d': p.range30d,
      mtd: p.rangeMtd,
      ytd: p.rangeYtd,
      '1y': p.range1y,
      all: p.rangeAll,
    }
    return (
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: space[3],
          flexWrap: 'wrap',
          marginTop: space[4],
        })}
      >
        <div
          mix={css({
            display: 'inline-flex',
            gap: space[1],
            // Carved track with a raised pill for the active option —
            // the same soft-active read as the country / theme chips.
            background: color.hover,
            boxShadow: shadow.inset,
            padding: '3px',
            borderRadius: radius.pill,
          })}
        >
          {RANGE_PRESETS.map((r) => (
            <RangeChip
              href={rangeHref(search, { range: r })}
              active={range.preset === r}
              label={labels[r]}
            />
          ))}
        </div>

        <form
          method="get"
          action="/"
          mix={css({
            display: 'inline-flex',
            alignItems: 'center',
            gap: space[2],
            flexWrap: 'wrap',
          })}
        >
          {/* Carry the chrome params through the GET form, which would
              otherwise replace the whole query string with its own
              fields and reset language / theme. */}
          {carryParams(search).map(([k, v]) => (
            <input type="hidden" name={k} value={v} />
          ))}
          <input
            type="date"
            name="from"
            value={range.from}
            max={range.to}
            required
            mix={css(dateFieldStyle)}
          />
          <span mix={css({ color: color.textDim, fontSize: font.sm })}>→</span>
          <input
            type="date"
            name="to"
            value={range.to}
            min={range.from}
            required
            mix={css(dateFieldStyle)}
          />
          <button type="submit" mix={css(applyButton)}>
            {p.rangeApply}
          </button>
        </form>
      </div>
    )
  }
}

function RangeChip() {
  return ({ href, active, label }: { href: string; active: boolean; label: string }) => (
    <a
      href={href}
      mix={css({
        display: 'inline-flex',
        alignItems: 'center',
        padding: `${space[1]} ${space[3]}`,
        fontSize: font.sm,
        fontWeight: 600,
        borderRadius: radius.pill,
        textDecoration: 'none',
        color: active ? color.text : color.textMuted,
        background: active ? color.surface : 'transparent',
        boxShadow: active ? shadow.card : 'none',
        transition: 'background 120ms ease, color 120ms ease, transform 120ms ease',
        '&:hover': active ? undefined : { color: color.text },
        '&:active': { transform: 'scale(0.97)' },
      })}
    >
      {label}
    </a>
  )
}

/// Query-string keys that describe the *page chrome* rather than the
/// chart window. Flipping a range must not reset the user's language,
/// theme or market scope.
const CARRY_KEYS = ['locale', 'theme', 'country'] as const

function carryParams(search: URLSearchParams): Array<[string, string]> {
  return CARRY_KEYS.flatMap((k) => {
    let v = search.get(k)
    return v ? [[k, v] as [string, string]] : []
  })
}

/// Build a dashboard href that sets the range and drops any stale
/// custom window, while keeping the chrome params.
function rangeHref(search: URLSearchParams, next: { range: RangePreset }): string {
  let qs = new URLSearchParams(carryParams(search))
  qs.set('range', next.range)
  return `/?${qs.toString()}`
}

/// `cash + positions = total assets`, above the performance metrics.
///
/// Reads from the ambient summary the auth wrapper already fetched for
/// the sidebar, so the dashboard costs no extra round trip.
///
/// The three values are laid out as an equation rather than three peers
/// — the total is what you came for, and showing the two parts beside it
/// is what makes it trustworthy. An inset well separates "what am I
/// worth" from the performance numbers below without adding another
/// raised card.
function NetWorthStrip() {
  return ({ locale }: { locale: string }) => {
    let summary = ambientPortfolioSummary()
    if (!summary) return null
    let p = messages(locale).pages.dashboard
    let cash = Number.parseFloat(summary.cash)
    // Negative cash isn't a rendering bug, it's the "ledger has buys but
    // no deposits" case. Say so, with the fix, instead of showing a
    // mysterious minus sign.
    let cashNegative = Number.isFinite(cash) && cash < 0
    let estimated = summary.unpriced_count > 0
    return (
      <div
        mix={css({
          padding: space[4],
          marginBottom: space[4],
          background: color.hover,
          borderRadius: radius.md,
          boxShadow: shadow.inset,
        })}
      >
        <div
          mix={css({
            display: 'flex',
            alignItems: 'flex-end',
            flexWrap: 'wrap',
            gap: space[4],
          })}
        >
          <NetWorthPart label={p.metricCash} value={fmtMoney(summary.cash)} />
          <Operator>+</Operator>
          <NetWorthPart
            label={p.metricMarketValue}
            value={fmtMoney(summary.market_value)}
          />
          <Operator>=</Operator>
          <NetWorthPart
            label={p.metricTotalAssets}
            value={fmtMoney(summary.total_assets)}
            emphasis
          />
        </div>
        {(cashNegative || estimated) && (
          <div
            mix={css({
              marginTop: space[3],
              fontSize: font.xs,
              color: cashNegative ? color.warnText : color.textDim,
              lineHeight: 1.5,
            })}
          >
            {cashNegative ? p.cashNegativeHint : p.estimatedHint(summary.unpriced_count)}
          </div>
        )}
      </div>
    )
  }
}

function NetWorthPart() {
  return ({
    label,
    value,
    emphasis,
  }: {
    label: string
    value: string
    emphasis?: boolean
  }) => (
    <div>
      <div
        mix={css({
          fontSize: font.xs,
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          color: color.textMuted,
          marginBottom: space[1],
        })}
      >
        {label}
      </div>
      <div
        mix={css({
          fontFamily: font.mono,
          fontSize: emphasis ? font.xxl : font.lg,
          fontWeight: 700,
          color: color.text,
          fontVariantNumeric: 'tabular-nums',
          lineHeight: 1.1,
        })}
      >
        {value}
      </div>
    </div>
  )
}

function Operator() {
  return ({ children }: { children: RemixNode }) => (
    <div
      mix={css({
        fontFamily: font.mono,
        fontSize: font.lg,
        color: color.textDim,
        paddingBottom: space[1],
      })}
    >
      {children}
    </div>
  )
}

/// Hand-rolled SVG line chart for the portfolio value time series.
/// Two lines: solid = market value, dashed = cost basis. Both share
/// the same y-scale so the gap visualizes unrealized P&L directly.
/// Y-axis ticks are computed from `niceTicks` so they round to
/// human-readable values; x-axis shows three date labels (start /
/// middle / latest) — denser labels overlap at 30 days. Empty / too-
/// short series → a friendly hint instead of a one-pixel line.
function PortfolioChart() {
  return ({ series }: { series: DailyValue[] }) => {
    if (series.length < 2) {
      return (
        <p
          mix={css({
            margin: 0,
            padding: `${space[4]} 0`,
            textAlign: 'center',
            fontSize: font.sm,
            color: color.textMuted,
          })}
        >
          Need at least 2 days of OHLCV to draw the curve. Backfill more bars
          via <code>POST /api/v1/ohlcv/batch</code>.
        </p>
      )
    }

    // Convert decimal strings to numbers once.
    let pts = series.map((d) => ({
      date: d.date,
      mv: Number.parseFloat(d.market_value),
      cb: Number.parseFloat(d.cost_basis),
      total: Number.parseFloat(d.total_assets),
    }))
    // Total assets joins the y-range so the new line can't run off the
    // top of a scale computed for market value alone.
    let allValues = pts.flatMap((p) => [p.mv, p.cb, p.total])
    let dataMin = Math.min(...allValues)
    let dataMax = Math.max(...allValues)
    // Pad the y-range so the line never glues to the chart edge.
    let padFrac = 0.05
    let span = Math.max(dataMax - dataMin, 1)
    let yMin = dataMin - span * padFrac
    let yMax = dataMax + span * padFrac
    let ticks = niceTicks(yMin, yMax, 4)
    let plotMin = Math.min(yMin, ticks[0])
    let plotMax = Math.max(yMax, ticks[ticks.length - 1])

    // viewBox-based geometry — the SVG scales fluidly into whatever
    // width the card gives it. Height stays fixed so neighboring
    // cards align.
    let w = 600
    let h = 200
    let padL = 56 // room for y-axis labels
    let padR = 12
    let padT = 8
    let padB = 24 // room for x-axis labels
    let plotW = w - padL - padR
    let plotH = h - padT - padB

    let x = (i: number) =>
      pts.length === 1 ? padL + plotW / 2 : padL + (i / (pts.length - 1)) * plotW
    let y = (v: number) =>
      padT + plotH - ((v - plotMin) / (plotMax - plotMin || 1)) * plotH

    let mvPath = pathFor(pts.map((p, i) => [x(i), y(p.mv)] as const))
    let cbPath = pathFor(pts.map((p, i) => [x(i), y(p.cb)] as const))
    let totalPath = pathFor(pts.map((p, i) => [x(i), y(p.total)] as const))
    // Only worth a third line when cash actually moves the number. With
    // no cash the total sits exactly on market value and the two lines
    // would overprint, reading as a rendering glitch.
    let showTotal = pts.some((p) => Math.abs(p.total - p.mv) > 0.005)

    // X-axis labels: first, middle, last date (in browser TZ — those
    // are calendar dates, no conversion needed but locale formatting
    // shortens them).
    let labelIdxs = pts.length >= 3 ? [0, Math.floor(pts.length / 2), pts.length - 1] : [0, pts.length - 1]

    return (
      <svg
        viewBox={`0 0 ${w} ${h}`}
        preserveAspectRatio="none"
        mix={css({
          display: 'block',
          width: '100%',
          height: 'auto',
          marginBottom: space[2],
        })}
      >
        {/* Horizontal grid + y-axis labels */}
        {ticks.map((t, i) => {
          let ty = y(t)
          return (
            <g key={i}>
              <line
                x1={padL}
                x2={w - padR}
                y1={ty}
                y2={ty}
                stroke={color.borderSoft}
                stroke-width="1"
              />
              <text
                x={padL - 6}
                y={ty + 3}
                text-anchor="end"
                font-size="10"
                fill={color.textMuted}
                font-family={font.mono}
              >
                {fmtCompact(t)}
              </text>
            </g>
          )
        })}

        {/* Cost basis (dashed, behind) */}
        <path
          d={cbPath}
          fill="none"
          stroke={color.textMuted}
          stroke-width="1.5"
          stroke-dasharray="4 4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        {/* Market value (solid, in front) */}
        <path
          d={mvPath}
          fill="none"
          stroke={color.brand}
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        {/* Total assets — market value plus cash, so it rides above the
            brand line by exactly the cash on hand. Drawn last and
            lighter so it frames the position curve rather than
            competing with it. */}
        {showTotal && (
          <path
            d={totalPath}
            fill="none"
            stroke={color.info}
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        )}

        {/* X-axis labels */}
        {labelIdxs.map((i) => (
          <text
            key={i}
            x={x(i)}
            y={h - 6}
            text-anchor={
              i === 0 ? 'start' : i === pts.length - 1 ? 'end' : 'middle'
            }
            font-size="10"
            fill={color.textMuted}
            font-family={font.mono}
          >
            {pts[i].date.slice(5)}
          </text>
        ))}
      </svg>
    )
  }
}

/// Build an SVG path `d` string from a list of (x, y) points.
function pathFor(points: ReadonlyArray<readonly [number, number]>): string {
  return points
    .map(([px, py], i) => `${i === 0 ? 'M' : 'L'} ${px.toFixed(2)},${py.toFixed(2)}`)
    .join(' ')
}

/// Pick `count` human-readable ticks covering [min, max]. Aims for
/// round multiples of 1 / 2 / 5 × 10^N — the classic "nice number"
/// trick.
function niceTicks(min: number, max: number, count: number): number[] {
  let raw = (max - min) / Math.max(count - 1, 1)
  let mag = Math.pow(10, Math.floor(Math.log10(raw)))
  let norm = raw / mag
  let step = (norm >= 5 ? 10 : norm >= 2 ? 5 : norm >= 1 ? 2 : 1) * mag
  let start = Math.floor(min / step) * step
  let end = Math.ceil(max / step) * step
  let ticks: number[] = []
  for (let v = start; v <= end + step / 2; v += step) ticks.push(v)
  return ticks
}

/// Compact money string for axis labels (`12.3K`, `1.45M`). Keeps the
/// chart visually balanced when the y-range is large.
function fmtCompact(n: number): string {
  let abs = Math.abs(n)
  if (abs >= 1e9) return `${(n / 1e9).toFixed(1)}B`
  if (abs >= 1e6) return `${(n / 1e6).toFixed(1)}M`
  if (abs >= 1e3) return `${(n / 1e3).toFixed(1)}K`
  return n.toFixed(0)
}

function Metric() {
  return ({
    label,
    value,
    badge,
  }: {
    label: string
    value: string
    badge?: import('remix/ui').RemixNode
  }) => (
    <div>
      <div
        mix={css({
          fontSize: font.xs,
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          color: color.textMuted,
          marginBottom: space[1],
        })}
      >
        {label}
      </div>
      <div
        mix={css({
          fontSize: font.xl,
          fontWeight: 700,
          color: color.text,
          fontFamily: font.mono,
          fontVariantNumeric: 'tabular-nums',
          marginBottom: space[1],
        })}
      >
        {value}
      </div>
      {badge}
    </div>
  )
}

function RecentActivityCard() {
  return ({ rows, locale }: { rows: AuditEntry[]; locale: string }) => {
    let p = messages(locale).pages.dashboard
    if (rows.length === 0) {
      return (
        <Card>
          <SectionTitle>{p.sectionRecentActivity}</SectionTitle>
          <EmptyState title={p.emptyNoActivity} hint={p.emptyActivityHint} />
        </Card>
      )
    }
    return (
      <Card>
        <SectionTitle>{p.sectionRecentActivity}</SectionTitle>
        <ul
          mix={css({
            margin: 0,
            padding: 0,
            listStyle: 'none',
            display: 'flex',
            flexDirection: 'column',
            gap: space[2],
          })}
        >
          {rows.map((r) => (
            <li
              key={r.id}
              mix={css({
                display: 'flex',
                alignItems: 'baseline',
                justifyContent: 'space-between',
                gap: space[2],
                fontSize: font.sm,
              })}
            >
              <span
                mix={css({
                  display: 'inline-flex',
                  alignItems: 'baseline',
                  gap: space[2],
                  minWidth: 0,
                  overflow: 'hidden',
                })}
              >
                <Badge tone={actionTone(r.action)}>{r.action}</Badge>
                <span
                  mix={css({
                    color: color.text,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                  })}
                >
                  {r.entity_type}
                  <span mix={css({ color: color.textMuted })}> #{r.entity_id}</span>
                </span>
              </span>
              <span
                mix={css({
                  fontSize: font.xs,
                  color: color.textMuted,
                  fontFamily: font.mono,
                  whiteSpace: 'nowrap',
                })}
              >
                <LocalTime value={r.created_at} format="datetime" />
              </span>
            </li>
          ))}
        </ul>
      </Card>
    )
  }
}

function TopMoversCard() {
  return ({ movers, locale }: { movers: Mover[]; locale: string }) => {
    let p = messages(locale).pages.dashboard
    if (movers.length === 0) {
      return (
        <Card>
          <SectionTitle>{p.sectionTopMovers}</SectionTitle>
          <EmptyState title={p.emptyNoMovers} hint={p.emptyMoversHint} />
        </Card>
      )
    }
    return (
      <Card>
        <SectionTitle hint={p.topMoversHint}>{p.sectionTopMovers}</SectionTitle>
        <ul
          mix={css({
            margin: 0,
            padding: 0,
            listStyle: 'none',
            display: 'flex',
            flexDirection: 'column',
            gap: space[2],
          })}
        >
          {movers.map((m) => {
            let up = m.change_pct >= 0
            let tone: BadgeTone = up ? 'success' : 'danger'
            return (
              <li
                key={m.stock_id}
                mix={css({
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  gap: space[2],
                  fontSize: font.sm,
                })}
              >
                <a
                  href={`/stocks/${m.stock_id}`}
                  mix={css({
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: space[2],
                    color: color.text,
                    textDecoration: 'none',
                    '&:hover': { color: color.brandHover },
                  })}
                >
                  <StockBadge symbol={m.symbol} />
                  <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>
                    {m.symbol}
                  </span>
                </a>
                <span
                  mix={css({
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: space[2],
                  })}
                >
                  <span
                    mix={css({
                      fontFamily: font.mono,
                      color: color.textMuted,
                    })}
                  >
                    {fmtMoney(m.close)}
                  </span>
                  <Badge tone={tone}>
                    {up ? '+' : ''}
                    {m.change_pct.toFixed(2)}%
                  </Badge>
                </span>
              </li>
            )
          })}
        </ul>
      </Card>
    )
  }
}

function actionTone(action: string): BadgeTone {
  if (action === 'create') return 'success'
  if (action === 'delete') return 'danger'
  if (action === 'update') return 'info'
  return 'neutral'
}


const dateFieldStyle = {
  padding: `${space[1]} ${space[2]}`,
  background: color.hover,
  border: `1px solid ${color.border}`,
  borderRadius: radius.md,
  fontSize: font.sm,
  color: color.text,
  fontFamily: font.mono,
  boxShadow: shadow.inset,
  outline: 'none',
  '&:focus': { borderColor: color.brand },
}

const applyButton = {
  padding: `${space[1]} ${space[3]}`,
  background: color.surface,
  border: `1px solid ${color.edge}`,
  borderRadius: radius.md,
  color: color.text,
  fontSize: font.sm,
  fontWeight: 600,
  fontFamily: 'inherit',
  cursor: 'pointer',
  boxShadow: shadow.card,
  '&:hover': { boxShadow: shadow.cardHover },
  '&:active': { boxShadow: shadow.pressed, transform: 'scale(0.98)' },
}
