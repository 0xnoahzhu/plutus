import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import {
  api,
  type AuditEntry,
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
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  Stat,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
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

    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    // Second wave — OHLCV for every held stock, in parallel. Each call
    // returns the full history; we only need the last two bars so we
    // sort and slice client-side. With <50 holdings this is fine; if
    // the user accumulates a long tail we'd add a `?days=N` parameter
    // or a batched endpoint.
    let ohlcvByStock = new Map<number, Ohlcv[]>()
    if (holdings.length > 0) {
      let results = await Promise.all(
        holdings.map((h) =>
          api.stockOhlcv(h.stock_id).catch(() => [] as Ohlcv[]),
        ),
      )
      holdings.forEach((h, i) => {
        // Sort ascending so the "latest" is at the end.
        let series = [...results[i]].sort((a, b) =>
          a.trade_date.localeCompare(b.trade_date),
        )
        ohlcvByStock.set(h.stock_id, series)
      })
    }

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
  }: DashboardProps) => {
    let p = messages(locale).pages.dashboard
    return (
      <Layout title={p.title} subtitle={p.subtitle} locale={locale} theme={theme}>
        <SectionTitle hint="real-time">Quick Stats</SectionTitle>
        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
            gap: space[3],
            marginBottom: space[8],
          })}
        >
          <Stat
            label="API Status"
            value={healthy ? 'Connected' : 'Down'}
            caption={healthy ? 'all systems go' : 'check the server'}
            trend={healthy ? 'up' : 'down'}
          />
          <Stat label="Markets" value={String(counts.markets)} caption="open" />
          <Stat label="Brokers" value={String(counts.brokers)} caption="active" />
          <Stat label="Accounts" value={String(counts.accounts)} caption="total" />
          <Stat label="Stocks" value={String(counts.stocks)} caption="tracked" />
          <Stat label="Watchlist" value={String(counts.watchlist)} caption="stocks" />
          <Stat label="Transactions" value={String(counts.transactions)} caption="recorded" />
          <Stat label="Open Positions" value={String(counts.holdings)} caption="current" />
          <Stat label="Trade plans" value={String(counts.tradePlans)} caption="active" />
          <Stat label="Open orders" value={String(counts.openOrders)} caption="at broker" />
        </div>

        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: '2fr 1fr',
            gap: space[5],
            '@media (max-width: 1000px)': { gridTemplateColumns: '1fr' },
          })}
        >
          <PortfolioSnapshotCard snapshot={snapshot} />

          <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[4] })}>
            <RecentActivityCard rows={recentActivity} />
            <TopMoversCard movers={movers} />
          </div>
        </div>

        <p
          mix={css({
            marginTop: space[8],
            fontSize: font.sm,
            color: color.textMuted,
          })}
        >
          API base: <code>{api.base}</code>
        </p>
      </Layout>
    )
  }
}

function PortfolioSnapshotCard() {
  return ({ snapshot }: { snapshot: PortfolioSnapshot }) => {
    if (snapshot.cost_basis === 0 && snapshot.market_value === 0) {
      return (
        <Card>
          <SectionTitle>Portfolio Snapshot</SectionTitle>
          <EmptyState
            title="No open positions"
            hint="Post buy transactions to /api/v1/transactions and the rollup lands here."
          />
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
        <SectionTitle hint={snapshot.fully_priced ? 'live' : 'partial — missing prices'}>
          Portfolio Snapshot
        </SectionTitle>
        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: space[4],
            marginBottom: space[4],
          })}
        >
          <Metric label="Market Value" value={fmtMoney(snapshot.market_value)} />
          <Metric label="Cost Basis" value={fmtMoney(snapshot.cost_basis)} />
          <Metric
            label="Unrealized P&L"
            value={`${upTrend ? '+' : ''}${fmtMoney(snapshot.unrealized)}`}
            badge={
              <Badge tone={tone}>
                {upTrend ? '+' : ''}
                {pnlPct.toFixed(2)}%
              </Badge>
            }
          />
        </div>
        <p
          mix={css({
            margin: 0,
            fontSize: font.xs,
            color: color.textMuted,
            lineHeight: 1.5,
          })}
        >
          Snapshot at the latest available OHLCV close. A time-series chart needs
          per-day historical position rollups, which is the next phase.
        </p>
      </Card>
    )
  }
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
  return ({ rows }: { rows: AuditEntry[] }) => {
    if (rows.length === 0) {
      return (
        <Card>
          <SectionTitle>Recent Activity</SectionTitle>
          <EmptyState title="No activity yet" hint="agent writes will surface here" />
        </Card>
      )
    }
    return (
      <Card>
        <SectionTitle>Recent Activity</SectionTitle>
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
  return ({ movers }: { movers: Mover[] }) => {
    if (movers.length === 0) {
      return (
        <Card>
          <SectionTitle>Top Movers</SectionTitle>
          <EmptyState
            title="Need at least 2 days of OHLCV"
            hint="The day-over-day %-change needs a prior close to compare against. Backfill via POST /api/v1/ohlcv/batch."
          />
        </Card>
      )
    }
    return (
      <Card>
        <SectionTitle hint="latest close">Top Movers</SectionTitle>
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
                    {m.close.toFixed(2)}
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

function fmtMoney(n: number): string {
  // Format with thousands separators, two decimals. Currency-agnostic
  // for now (assumes the user's base currency; the snapshot doesn't
  // mix currencies because /holdings rolls up per-account base).
  let sign = n < 0 ? '-' : ''
  let abs = Math.abs(n)
  let int = Math.floor(abs)
  let cents = Math.round((abs - int) * 100)
    .toString()
    .padStart(2, '0')
  let intStr = int.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return `${sign}${intStr}.${cents}`
}
