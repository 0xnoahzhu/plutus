import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import { api, type Holding, type Stock } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  type BadgeTone,
  Card,
  color,
  EmptyState,
  filterByCountry,
  font,
  Layout,
  parseCountry,
  resolveLocale,
  resolveTheme,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { fmtMoney } from '../ui/format.ts'
import { render } from '../utils/render.tsx'

export const holdings: BuildAction<'GET', typeof routes.holdings> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let method = url.searchParams.get('method') ?? 'fifo'

    let [holdingsList, stocks] = await Promise.all([
      api.holdings({ method }).catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    // Order is set server-side by ticker; we just country-filter here.
    let filtered = filterByCountry(holdingsList, country, (h) =>
      stockMap.get(h.stock_id)?.market_code,
    )

    return render(
      <HoldingsPage
        rows={filtered}
        stocks={stockMap}
        country={country}
        locale={locale}
        theme={theme}
        method={method}
      />,
      request,
      { locale, theme },
    )
  },
}

interface HoldingsProps {
  rows: Holding[]
  stocks: Map<number, Stock>
  country: string
  locale: string
  theme: Theme
  method: string
}

function HoldingsPage() {
  return ({ rows, stocks, country, locale, theme, method }: HoldingsProps) => {
    let p = messages(locale).pages.holdings
    return (
    <Layout
      title={p.title}
      subtitle={`${p.costBasisLabel}: ${method.toUpperCase()}`}
      country={country}
      locale={locale}
      theme={theme}
    >
      {rows.length === 0 ? (
        <Card>
          <EmptyState title={p.emptyTitle} hint={p.emptyHint} />
        </Card>
      ) : (
        <Card padding="0">
          <table
            mix={css({
              width: '100%',
              borderCollapse: 'collapse',
              fontSize: font.base,
            })}
          >
            <thead>
              <tr>
                <Th>{p.columnSymbol}</Th>
                <Th>{p.columnMarket}</Th>
                <Th>{p.columnCurrency}</Th>
                <Th align="right">{p.columnQuantity}</Th>
                <Th align="right">{p.columnAvgCost}</Th>
                <Th align="right">{p.columnCostBasis}</Th>
                <Th align="right">{p.columnUnrealizedPnl}</Th>
                <Th align="right">{p.columnRealizedPnl}</Th>
              </tr>
            </thead>
            <tbody>
              {rows.map((h) => {
                let s = stocks.get(h.stock_id)
                let realized = Number.parseFloat(h.realized_pnl_base)
                let realizedTrend: 'up' | 'down' | 'flat' =
                  realized > 0 ? 'up' : realized < 0 ? 'down' : 'flat'
                let unrealizedNum =
                  h.unrealized_pnl_base != null
                    ? Number.parseFloat(h.unrealized_pnl_base)
                    : null
                let unrealizedTrend: 'up' | 'down' | 'flat' =
                  unrealizedNum == null
                    ? 'flat'
                    : unrealizedNum > 0
                      ? 'up'
                      : unrealizedNum < 0
                        ? 'down'
                        : 'flat'
                return (
                  <tr
                    mix={css({
                      borderTop: `1px solid ${color.borderSoft}`,
                      '&:hover td': { background: color.bg },
                    })}
                  >
                    <Td>
                      {s ? (
                        <a
                          href={`/stocks/${s.id}`}
                          mix={css({
                            display: 'inline-flex',
                            alignItems: 'center',
                            gap: space[2],
                            textDecoration: 'none',
                            color: color.text,
                            '&:hover': { color: color.brandHover },
                          })}
                        >
                          <StockBadge symbol={s.symbol} />
                          <span
                            mix={css({
                              fontFamily: font.mono,
                              fontWeight: 600,
                            })}
                          >
                            {s.symbol}
                          </span>
                        </a>
                      ) : (
                        <span mix={css({ color: color.textMuted })}>#{h.stock_id}</span>
                      )}
                    </Td>
                    <Td>
                      <Badge tone="neutral">{s?.market_code ?? '?'}</Badge>
                    </Td>
                    <Td>{s?.currency ?? '?'}</Td>
                    <Td align="right" mono>
                      {h.quantity}
                    </Td>
                    <Td align="right" mono>
                      {fmtMoney(h.avg_cost_trade)}
                    </Td>
                    <Td align="right" mono>
                      {fmtMoney(h.cost_base)}
                    </Td>
                    <Td align="right">
                      <PnlPill value={h.unrealized_pnl_base} trend={unrealizedTrend} />
                    </Td>
                    <Td align="right">
                      <PnlPill value={h.realized_pnl_base} trend={realizedTrend} />
                    </Td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </Card>
      )}
    </Layout>
    )
  }
}

function Th() {
  return ({
    children,
    align = 'left',
  }: {
    children: RemixNode
    align?: 'left' | 'right'
  }) => (
    <th
      mix={css({
        textAlign: align,
        padding: `${space[3]} ${space[4]}`,
        fontSize: font.xs,
        textTransform: 'uppercase',
        letterSpacing: '0.08em',
        color: color.textMuted,
        fontWeight: 600,
        background: color.bg,
        borderBottom: `1px solid ${color.border}`,
      })}
    >
      {children}
    </th>
  )
}

function Td() {
  return ({
    children,
    align = 'left',
    mono,
  }: {
    children: RemixNode
    align?: 'left' | 'right'
    mono?: boolean
  }) => (
    <td
      mix={css({
        padding: `${space[3]} ${space[4]}`,
        textAlign: align,
        fontVariantNumeric: 'tabular-nums',
        fontFamily: mono ? font.mono : 'inherit',
        color: color.text,
      })}
    >
      {children}
    </td>
  )
}

function PnlPill() {
  return ({
    value,
    trend,
  }: {
    /// Decimal string from the API, or `null` when the source field
    /// itself is null (e.g. no OHLCV yet → no market value → no
    /// unrealized P&L).
    value: string | null
    trend: 'up' | 'down' | 'flat'
  }) => {
    // Render `—` for:
    //   - null: the underlying field has no value (missing OHLCV bar).
    //   - 0: realized_pnl_base = 0 means "no sells yet" in practice
    //     (commissions make true zero break-even vanishingly rare);
    //     showing $0.00 reads as a bug. unrealized_pnl_base = 0 means
    //     the market value happens to equal cost basis to the cent,
    //     also rare; rendering `—` is consistent.
    let n = value == null ? NaN : Number.parseFloat(value)
    if (!Number.isFinite(n) || n === 0) {
      return (
        <span mix={css({ color: color.textDim, fontVariantNumeric: 'tabular-nums' })}>
          —
        </span>
      )
    }
    let tone: BadgeTone =
      trend === 'up' ? 'success' : trend === 'down' ? 'danger' : 'neutral'
    let sign = n > 0 ? '+' : ''
    return <Badge tone={tone}>{`${sign}${fmtMoney(value)}`}</Badge>
  }
}
