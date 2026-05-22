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
  radius,
  resolveLocale,
  resolveTheme,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

const PER_PAGE = 20

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
    let start = total === 0 ? 0 : (page - 1) * perPage + 1
    let end = Math.min(page * perPage, total)
    return (
      <Layout
        title={p.title}
        subtitle={p.paginationCount(start, end, total)}
        locale={locale}
        theme={theme}
      >
        <Card>
          <SearchBar locale={locale} query={query} />
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
            locale={locale}
            page={page}
            totalPages={totalPages}
            query={query}
          />
        )}
      </Layout>
    )
  }
}

/// GET form (no JS needed) that submits `?q=` and resets `?page` to 1.
/// Pressing Enter inside the input naturally triggers form submission;
/// the explicit Search button is the alternative for users who prefer
/// clicking.
function SearchBar() {
  return ({ locale, query }: { locale: string; query: string }) => {
    let p = messages(locale).pages.stocks
    return (
      <form
        method="get"
        action="/stocks"
        mix={css({
          display: 'flex',
          gap: space[2],
          alignItems: 'center',
          margin: 0,
        })}
      >
        <input
          type="text"
          name="q"
          value={query}
          placeholder={p.searchPlaceholder}
          autocomplete="off"
          mix={css({
            flex: 1,
            padding: `${space[2]} ${space[3]}`,
            fontSize: font.base,
            fontFamily: font.sans,
            color: color.text,
            background: color.bg,
            border: `1px solid ${color.border}`,
            borderRadius: radius.md,
            outline: 'none',
            '&:focus': { borderColor: color.brand },
            '&::placeholder': { color: color.textDim },
          })}
        />
        <button
          type="submit"
          mix={css({
            padding: `${space[2]} ${space[4]}`,
            fontSize: font.sm,
            fontWeight: 600,
            color: color.textOnBrand,
            background: color.brand,
            border: 'none',
            borderRadius: radius.md,
            cursor: 'pointer',
            transition: 'background 120ms ease',
            '&:hover': { background: color.brandHover },
          })}
        >
          {p.searchSubmit}
        </button>
        {query !== '' && (
          <a
            href="/stocks"
            mix={css({
              fontSize: font.sm,
              color: color.textMuted,
              textDecoration: 'none',
              padding: `${space[2]} ${space[3]}`,
              '&:hover': { color: color.text },
            })}
          >
            {p.searchClear}
          </a>
        )}
      </form>
    )
  }
}

function Pagination() {
  return ({
    locale,
    page,
    totalPages,
    query,
  }: {
    locale: string
    page: number
    totalPages: number
    query: string
  }) => {
    let p = messages(locale).pages.stocks
    let qs = (n: number) => {
      let params = new URLSearchParams({ page: String(n) })
      if (query) params.set('q', query)
      return `/stocks?${params.toString()}`
    }
    let pillBase = css({
      display: 'inline-flex',
      alignItems: 'center',
      padding: `${space[2]} ${space[4]}`,
      fontSize: font.sm,
      fontWeight: 600,
      borderRadius: radius.md,
      border: `1px solid ${color.border}`,
      background: color.surface,
      color: color.text,
      textDecoration: 'none',
      transition: 'background 120ms ease',
      '&:hover': { background: color.bg },
    })
    let pillDisabled = css({
      display: 'inline-flex',
      alignItems: 'center',
      padding: `${space[2]} ${space[4]}`,
      fontSize: font.sm,
      fontWeight: 600,
      borderRadius: radius.md,
      border: `1px solid ${color.borderSoft}`,
      background: 'transparent',
      color: color.textDim,
      cursor: 'not-allowed',
    })
    return (
      <div
        mix={css({
          marginTop: space[4],
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: space[3],
          flexWrap: 'wrap',
        })}
      >
        <span mix={css({ fontSize: font.sm, color: color.textMuted })}>
          {p.paginationPage(page, totalPages)}
        </span>
        <div mix={css({ display: 'inline-flex', gap: space[2] })}>
          {page > 1 ? (
            <a href={qs(page - 1)} mix={pillBase}>
              {p.paginationPrev}
            </a>
          ) : (
            <span mix={pillDisabled}>{p.paginationPrev}</span>
          )}
          {page < totalPages ? (
            <a href={qs(page + 1)} mix={pillBase}>
              {p.paginationNext}
            </a>
          ) : (
            <span mix={pillDisabled}>{p.paginationNext}</span>
          )}
        </div>
      </div>
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
