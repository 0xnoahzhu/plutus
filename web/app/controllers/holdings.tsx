import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Holding, type Stock } from '../api.ts'
import type { routes } from '../routes.ts'
import { filterByCountry, Layout, parseCountry, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const holdings: BuildAction<'GET', typeof routes.holdings> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let method = url.searchParams.get('method') ?? 'fifo'

    let [holdingsList, stocks] = await Promise.all([
      api.holdings({ method }).catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    let filtered = filterByCountry(holdingsList, country, (h) =>
      stockMap.get(h.stock_id)?.market_code,
    )

    return render(
      <HoldingsPage
        rows={filtered}
        stocks={stockMap}
        country={country}
        locale={locale}
        method={method}
      />,
      request,
      { locale },
    )
  },
}

interface HoldingsProps {
  rows: Holding[]
  stocks: Map<number, Stock>
  country: string
  locale: string
  method: string
}

function HoldingsPage() {
  return ({ rows, stocks, country, locale, method }: HoldingsProps) => (
    <Layout title="Holdings" country={country} locale={locale}>
      <p
        mix={css({
          fontSize: '12px',
          color: '#64748b',
          marginBottom: '12px',
        })}
      >
        Cost basis: <strong>{method}</strong>
      </p>
      {rows.length === 0 ? (
        <p mix={css({ color: '#64748b' })}>No open positions match the current filter.</p>
      ) : (
        <table
          mix={css({
            width: '100%',
            borderCollapse: 'collapse',
            background: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '8px',
            overflow: 'hidden',
            fontSize: '14px',
          })}
        >
          <thead mix={css({ background: '#f1f5f9' })}>
            <tr>
              <Th>Symbol</Th>
              <Th>Market</Th>
              <Th>Currency</Th>
              <Th align="right">Qty</Th>
              <Th align="right">Avg Cost</Th>
              <Th align="right">Cost (base)</Th>
              <Th align="right">Realized P&L</Th>
            </tr>
          </thead>
          <tbody>
            {rows.map((h) => {
              let s = stocks.get(h.stock_id)
              return (
                <tr mix={css({ borderTop: '1px solid #e2e8f0' })}>
                  <Td>
                    {s ? (
                      <a
                        href={`/stocks/${s.id}`}
                        mix={css({
                          fontFamily: 'ui-monospace, SFMono-Regular, monospace',
                          fontWeight: 600,
                          color: '#1d4ed8',
                          textDecoration: 'none',
                          '&:hover': { textDecoration: 'underline' },
                        })}
                      >
                        {s.symbol}
                      </a>
                    ) : (
                      <strong>#{h.stock_id}</strong>
                    )}
                  </Td>
                  <Td>
                    <Badge>{s?.market_code ?? '?'}</Badge>
                  </Td>
                  <Td>{s?.currency ?? '?'}</Td>
                  <Td align="right">{h.quantity}</Td>
                  <Td align="right">{h.avg_cost_trade}</Td>
                  <Td align="right">{h.cost_base}</Td>
                  <Td align="right">
                    <Pnl value={h.realized_pnl_base} />
                  </Td>
                </tr>
              )
            })}
          </tbody>
        </table>
      )}
    </Layout>
  )
}

function Th() {
  return ({ children, align = 'left' }: { children: string; align?: 'left' | 'right' }) => (
    <th
      mix={css({
        textAlign: align,
        padding: '10px 14px',
        fontSize: '11px',
        textTransform: 'uppercase',
        letterSpacing: '0.06em',
        color: '#64748b',
        fontWeight: 600,
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
  }: {
    children: import('remix/ui').RemixNode
    align?: 'left' | 'right'
  }) => (
    <td
      mix={css({
        padding: '10px 14px',
        textAlign: align,
        fontVariantNumeric: 'tabular-nums',
      })}
    >
      {children}
    </td>
  )
}

function Badge() {
  return ({ children }: { children: string }) => (
    <span
      mix={css({
        display: 'inline-block',
        padding: '2px 8px',
        background: '#e0e7ff',
        color: '#3730a3',
        borderRadius: '999px',
        fontSize: '11px',
        fontWeight: 600,
      })}
    >
      {children}
    </span>
  )
}

function Pnl() {
  return ({ value }: { value: string }) => {
    let n = Number.parseFloat(value)
    let positive = n > 0
    let negative = n < 0
    return (
      <span
        mix={css({
          color: positive ? '#059669' : negative ? '#dc2626' : '#475569',
          fontWeight: 500,
        })}
      >
        {value}
      </span>
    )
  }
}
