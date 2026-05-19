import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Stock, type Transaction } from '../api.ts'
import type { routes } from '../routes.ts'
import { filterByCountry, Layout, parseCountry, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const transactions: BuildAction<'GET', typeof routes.transactions> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let [txs, stocks] = await Promise.all([
      api.transactions().catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    let filtered = filterByCountry(txs, country, (t) =>
      t.stock_id != null ? stockMap.get(t.stock_id)?.market_code : undefined,
    )
    return render(
      <TransactionsPage rows={filtered} stocks={stockMap} country={country} locale={locale} />,
      request,
      { locale },
    )
  },
}

interface TxnProps {
  rows: Transaction[]
  stocks: Map<number, Stock>
  country: string
  locale: string
}

function TransactionsPage() {
  return ({ rows, stocks, country, locale }: TxnProps) => (
    <Layout title="Transactions" country={country} locale={locale}>
      {rows.length === 0 ? (
        <p mix={css({ color: '#64748b' })}>No transactions recorded yet.</p>
      ) : (
        <table
          mix={css({
            width: '100%',
            borderCollapse: 'collapse',
            background: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '8px',
            overflow: 'hidden',
            fontSize: '13px',
          })}
        >
          <thead mix={css({ background: '#f1f5f9' })}>
            <tr>
              <Th>Date</Th>
              <Th>Kind</Th>
              <Th>Symbol</Th>
              <Th>Market</Th>
              <Th align="right">Qty</Th>
              <Th align="right">Price</Th>
              <Th>Curr</Th>
              <Th align="right">Commission</Th>
              <Th>Source</Th>
            </tr>
          </thead>
          <tbody>
            {rows.map((t) => {
              let s = t.stock_id != null ? stocks.get(t.stock_id) : null
              return (
                <tr mix={css({ borderTop: '1px solid #e2e8f0' })}>
                  <Td>{t.executed_at.slice(0, 16).replace('T', ' ')}</Td>
                  <Td>
                    <Badge kind={t.kind}>{t.kind}</Badge>
                  </Td>
                  <Td>{s?.symbol ?? '—'}</Td>
                  <Td>{s ? <Pill>{s.market_code}</Pill> : '—'}</Td>
                  <Td align="right">{t.quantity}</Td>
                  <Td align="right">{t.price}</Td>
                  <Td>{t.trade_currency}</Td>
                  <Td align="right">
                    {t.commission} {t.commission_currency}
                  </Td>
                  <Td>
                    <SourceBadge source={t.source} />
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

function Pill() {
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

function Badge() {
  return ({ children, kind }: { children: string; kind: string }) => {
    let palette: Record<string, [string, string]> = {
      BUY: ['#dcfce7', '#166534'],
      SELL: ['#fee2e2', '#991b1b'],
      DIVIDEND: ['#fef3c7', '#92400e'],
      FEE: ['#fce7f3', '#9d174d'],
      INTEREST: ['#fef3c7', '#92400e'],
      DEPOSIT: ['#dbeafe', '#1e40af'],
      WITHDRAWAL: ['#e0e7ff', '#3730a3'],
      FX: ['#cffafe', '#155e75'],
      CORPORATE_ACTION: ['#f5f3ff', '#5b21b6'],
    }
    let [bg, fg] = palette[kind] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          display: 'inline-block',
          padding: '2px 8px',
          background: bg,
          color: fg,
          borderRadius: '4px',
          fontSize: '11px',
          fontWeight: 600,
        })}
      >
        {children}
      </span>
    )
  }
}

function SourceBadge() {
  return ({ source }: { source: string }) => {
    let agent = source === 'agent'
    return (
      <span
        mix={css({
          fontSize: '11px',
          color: agent ? '#1d4ed8' : '#64748b',
          fontWeight: agent ? 600 : 400,
        })}
      >
        {agent ? '🤖 agent' : source}
      </span>
    )
  }
}
