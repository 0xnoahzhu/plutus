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
  font,
  Layout,
  resolveLocale,
  resolveTheme,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { Pagination, SearchBar } from '../ui/pagination.tsx'
import { render } from '../utils/render.tsx'

const PER_PAGE = 15

export const stocks: BuildAction<'GET', typeof routes.stocks> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let q = (url.searchParams.get('q') ?? '').trim()
    let pageParam = Number(url.searchParams.get('page') ?? '1')
    let page = Number.isFinite(pageParam) && pageParam > 0 ? Math.floor(pageParam) : 1
    let result = await api
      .stocksPage({ page, perPage: PER_PAGE, q: q || undefined, locale })
      .catch(() => ({ items: [] as Stock[], total: 0, page, perPage: PER_PAGE }))

    return render(
      <StocksPage
        rows={result.items}
        total={result.total}
        page={page}
        perPage={PER_PAGE}
        query={q}
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}

interface StocksProps {
  rows: Stock[]
  total: number
  page: number
  perPage: number
  query: string
  locale: string
  theme: Theme
}

function StocksPage() {
  return ({ rows, total, page, perPage, query, locale, theme }: StocksProps) => {
    let p = messages(locale).pages.stocks
    let totalPages = Math.max(1, Math.ceil(total / perPage))
    return (
      <Layout
        title={p.title}
        subtitle={messages(locale).common.paginationShowing(
          total === 0 ? 0 : (page - 1) * perPage + 1,
          Math.min(page * perPage, total),
          total,
        )}
        locale={locale}
        theme={theme}
      >
        <Card>
          <SearchBar
            action="/stocks"
            locale={locale}
            query={query}
            placeholder={p.searchPlaceholder}
          />
        </Card>
        <div mix={css({ marginTop: space[4] })}>
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
                    <Th>{p.columnAssetClass}</Th>
                    <Th align="right">{p.columnId}</Th>
                  </tr>
                </thead>
                <tbody>
                  {rows.map((s) => (
                    <tr
                      data-row-href={`/stocks/${s.id}`}
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
        </div>
        {totalPages > 1 && (
          <Pagination
            action="/stocks"
            locale={locale}
            page={page}
            totalPages={totalPages}
            total={total}
            perPage={perPage}
            query={query}
          />
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
