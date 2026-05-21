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
      subtitle={`Cost basis: ${method.toUpperCase()}`}
      country={country}
      locale={locale}
      theme={theme}
    >
      {rows.length === 0 ? (
        <Card>
          <EmptyState
            title="No open positions"
            hint="Add a buy transaction or change the country filter."
          />
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
                <Th>Symbol</Th>
                <Th>Market</Th>
                <Th>Currency</Th>
                <Th align="right">Quantity</Th>
                <Th align="right">Avg Cost</Th>
                <Th align="right">Cost Basis</Th>
                <Th align="right">Realized P&L</Th>
              </tr>
            </thead>
            <tbody>
              {rows.map((h) => {
                let s = stocks.get(h.stock_id)
                let pnl = Number.parseFloat(h.realized_pnl_base)
                let trend: 'up' | 'down' | 'flat' =
                  pnl > 0 ? 'up' : pnl < 0 ? 'down' : 'flat'
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
                      <PnlPill value={h.realized_pnl_base} trend={trend} />
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
  return ({ value, trend }: { value: string; trend: 'up' | 'down' | 'flat' }) => {
    let n = Number.parseFloat(value)
    // realized_pnl_base = 0 in practice means "no sells have happened
    // for this position yet" (you can only realize gains by selling),
    // not "the trades broke even to the cent" — that's vanishingly
    // rare once commissions are involved. Show `—` instead of $0.00
    // so the column is honest about the absence of data.
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
