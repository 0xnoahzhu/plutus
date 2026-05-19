import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Stock } from '../api.ts'
import type { routes } from '../routes.ts'
import { filterByCountry, Layout, parseCountry, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const stocks: BuildAction<'GET', typeof routes.stocks> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let list = await api.stocks().catch(() => [])
    let filtered = filterByCountry(list, country, (s) => s.market_code)
    return render(
      <StocksPage rows={filtered} country={country} locale={locale} />,
      request,
      { locale },
    )
  },
}

interface StocksProps {
  rows: Stock[]
  country: string
  locale: string
}

function StocksPage() {
  return ({ rows, country, locale }: StocksProps) => (
    <Layout title="Stocks" country={country} locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Click a row for company name, description, and translations. Stocks are added via
        <code mix={css({ marginLeft: '4px' })}>POST /api/v1/stocks</code>.
      </p>
      {rows.length === 0 ? (
        <p mix={css({ color: '#64748b' })}>No stocks recorded yet for the current filter.</p>
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
              <Th>Asset class</Th>
              <Th align="right">ID</Th>
            </tr>
          </thead>
          <tbody>
            {rows.map((s) => (
              <tr
                mix={css({
                  borderTop: '1px solid #e2e8f0',
                  cursor: 'pointer',
                  '&:hover': { background: '#f8fafc' },
                })}
              >
                <Td>
                  <a
                    href={`/stocks/${s.id}`}
                    mix={css({
                      color: '#1d4ed8',
                      textDecoration: 'none',
                      fontWeight: 600,
                      '&:hover': { textDecoration: 'underline' },
                    })}
                  >
                    {s.symbol}
                  </a>
                </Td>
                <Td>
                  <Badge>{s.market_code}</Badge>
                </Td>
                <Td>{s.currency}</Td>
                <Td>{s.asset_class}</Td>
                <Td align="right">
                  <code mix={css({ color: '#64748b' })}>{s.id}</code>
                </Td>
              </tr>
            ))}
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
