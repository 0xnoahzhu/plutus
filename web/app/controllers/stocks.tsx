import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import { api, type Stock } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
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
import { render } from '../utils/render.tsx'

export const stocks: BuildAction<'GET', typeof routes.stocks> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let list = await api.stocks(locale).catch(() => [])
    let filtered = filterByCountry(list, country, (s) => s.market_code)
    return render(
      <StocksPage rows={filtered} country={country} locale={locale} theme={theme} />,
      request,
      { locale, theme },
    )
  },
}

interface StocksProps {
  rows: Stock[]
  country: string
  locale: string
  theme: Theme
}

function StocksPage() {
  return ({ rows, country, locale, theme }: StocksProps) => {
    let p = messages(locale).pages.stocks
    return (
    <Layout
      title={p.title}
      subtitle={`${rows.length} tracked in ${country}`}
      country={country}
      locale={locale}
      theme={theme}
    >
      {rows.length === 0 ? (
        <Card>
          <EmptyState
            title="No stocks yet"
            hint={
              <>
                Add one with <code>POST /api/v1/stocks</code>.
              </>
            }
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
                <Th>{p.columnSymbol}</Th>
                <Th>{p.columnMarket}</Th>
                <Th>{p.columnCurrency}</Th>
                <Th>{p.columnAssetClass}</Th>
                <Th align="right">{p.columnId}</Th>
              </tr>
            </thead>
            <tbody>
              {rows.map((s) => (
                <tr
                  mix={css({
                    borderTop: `1px solid ${color.borderSoft}`,
                    cursor: 'pointer',
                    '&:hover td': { background: color.bg },
                  })}
                >
                  <Td>
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
                      <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>
                        {s.symbol}
                      </span>
                    </a>
                  </Td>
                  <Td>
                    <Badge tone="neutral">{s.market_code}</Badge>
                  </Td>
                  <Td>{s.currency}</Td>
                  <Td>{s.asset_class}</Td>
                  <Td align="right">
                    <span mix={css({ color: color.textMuted, fontFamily: font.mono })}>
                      #{s.id}
                    </span>
                  </Td>
                </tr>
              ))}
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
  }: {
    children: RemixNode
    align?: 'left' | 'right'
  }) => (
    <td
      mix={css({
        padding: `${space[3]} ${space[4]}`,
        textAlign: align,
      })}
    >
      {children}
    </td>
  )
}
