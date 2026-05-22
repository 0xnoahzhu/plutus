import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import { api, type Stock, type Transaction } from '../api.ts'
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
import { LocalTime } from '../ui/local-time.tsx'
import { Pagination, SearchBar } from '../ui/pagination.tsx'
import { render } from '../utils/render.tsx'

const PER_PAGE = 15

export const transactions: BuildAction<'GET', typeof routes.transactions> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let q = (url.searchParams.get('q') ?? '').trim()
    let pageParam = Number(url.searchParams.get('page') ?? '1')
    let page = Number.isFinite(pageParam) && pageParam > 0 ? Math.floor(pageParam) : 1

    // Backend handles country + q + pagination; we only need to
    // resolve stock symbols for the page slice so the table can
    // render market_code / symbol.
    let result = await api
      .transactionsPage({
        country: country || undefined,
        page,
        perPage: PER_PAGE,
        q: q || undefined,
      })
      .catch(() => ({
        items: [] as Transaction[],
        total: 0,
        page,
        perPage: PER_PAGE,
      }))

    let stockIds = result.items
      .map((t) => t.stock_id)
      .filter((id): id is number => id != null)
    let stocks = await api.stocksByIds(stockIds, locale).catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    return render(
      <TransactionsPage
        rows={result.items}
        total={result.total}
        page={page}
        perPage={PER_PAGE}
        query={q}
        stocks={stockMap}
        country={country}
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}

interface TxnProps {
  rows: Transaction[]
  total: number
  page: number
  perPage: number
  query: string
  stocks: Map<number, Stock>
  country: string
  locale: string
  theme: Theme
}

function TransactionsPage() {
  return ({
    rows,
    total,
    page,
    perPage,
    query,
    stocks,
    country,
    locale,
    theme,
  }: TxnProps) => {
    let p = messages(locale).pages.transactions
    let totalPages = Math.max(1, Math.ceil(total / perPage))
    return (
    <Layout
      title={p.title}
      subtitle={p.subtitle(total, country)}
      country={country}
      locale={locale}
      theme={theme}
    >
      <Card>
        <SearchBar
          action="/transactions"
          locale={locale}
          query={query}
          placeholder="Symbol"
          extraParams={{ country }}
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
                <Th>{p.columnDate}</Th>
                <Th>{p.columnKind}</Th>
                <Th>{p.columnSymbol}</Th>
                <Th>{p.columnMarket}</Th>
                <Th align="right">{p.columnQty}</Th>
                <Th align="right">{p.columnPrice}</Th>
                <Th>{p.columnCurrency}</Th>
                <Th align="right">{p.columnCommission}</Th>
                <Th>{p.columnSource}</Th>
              </tr>
            </thead>
            <tbody>
              {rows.map((t) => {
                let s = t.stock_id != null ? stocks.get(t.stock_id) : null
                return (
                  <tr
                    data-row-href={t.stock_id != null ? `/stocks/${t.stock_id}` : undefined}
                    mix={css({
                      borderTop: `1px solid ${color.borderSoft}`,
                      cursor: t.stock_id != null ? 'pointer' : 'default',
                      '&:hover td': { background: color.bg },
                    })}
                  >
                    <Td>
                      <span mix={css({ color: color.textMuted, fontFamily: font.mono })}>
                        <LocalTime value={t.executed_at} format="datetime" />
                      </span>
                    </Td>
                    <Td>
                      <KindBadge kind={t.kind} />
                    </Td>
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
                          <StockBadge symbol={s.symbol} size={22} />
                          <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>
                            {s.symbol}
                          </span>
                        </a>
                      ) : (
                        <span mix={css({ color: color.textMuted })}>—</span>
                      )}
                    </Td>
                    <Td>{s ? <Badge tone="neutral">{s.market_code}</Badge> : '—'}</Td>
                    <Td align="right" mono>{t.quantity}</Td>
                    <Td align="right" mono>{fmtMoney(t.price)}</Td>
                    <Td>{t.trade_currency}</Td>
                    <Td align="right" mono>
                      {fmtMoney(t.commission)} {t.commission_currency}
                    </Td>
                    <Td>
                      <span
                        mix={css({
                          fontSize: font.xs,
                          color: t.source === 'agent' ? color.brandHover : color.textMuted,
                          fontWeight: t.source === 'agent' ? 600 : 400,
                        })}
                      >
                        {t.source}
                      </span>
                    </Td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </Card>
      )}
      </div>
      {totalPages > 1 && (
        <Pagination
          action="/transactions"
          locale={locale}
          page={page}
          totalPages={totalPages}
          total={total}
          perPage={perPage}
          query={query}
          extraParams={{ country }}
        />
      )}
    </Layout>
    )
  }
}

function KindBadge() {
  return ({ kind }: { kind: string }) => {
    let toneMap: Record<string, BadgeTone> = {
      BUY: 'success',
      SELL: 'danger',
      DIVIDEND: 'warn',
      FEE: 'danger',
      INTEREST: 'warn',
      DEPOSIT: 'info',
      WITHDRAWAL: 'neutral',
      FX: 'brand',
      CORPORATE_ACTION: 'info',
    }
    return <Badge tone={toneMap[kind] ?? 'neutral'}>{kind}</Badge>
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
      })}
    >
      {children}
    </td>
  )
}
