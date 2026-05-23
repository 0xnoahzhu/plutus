import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import {
  api,
  type Stock,
  type WatchlistItem,
  type WatchlistReport,
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
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  shadow,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { MarkdownToggle } from '../ui/markdown.tsx'
import { Pagination, SearchBar } from '../ui/pagination.tsx'
import { render } from '../utils/render.tsx'

const PER_PAGE = 15

/// `/watchlists` is split into three top-level tabs (`?tab=...`) so
/// the items table and the agent reports each get the full page
/// canvas. With a large watchlist, mixing the two on one page meant
/// scrolling past the items table to reach the report archive; tabs
/// remove that cost entirely.
type Tab = 'items' | 'daily' | 'weekly'

export const watchlists: BuildAction<'GET', typeof routes.watchlists> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let tab = resolveTab(url.searchParams.get('tab'))
    let q = (url.searchParams.get('q') ?? '').trim()
    let pageParam = Number(url.searchParams.get('page') ?? '1')
    let page = Number.isFinite(pageParam) && pageParam > 0 ? Math.floor(pageParam) : 1

    // Items page is paginated/searchable; reports come back in full.
    // Search and pagination only apply when the items tab is active —
    // for the reports tabs we skip the params (page=N would still
    // shift the URL but the items endpoint just ignores it for the
    // hidden tab).
    let [itemsResult, dailyReports, weeklyReports] = await Promise.all([
      api
        .watchlistItemsPage({ page, perPage: PER_PAGE, q: q || undefined })
        .catch(() => ({ items: [] as WatchlistItem[], total: 0, page, perPage: PER_PAGE })),
      api.watchlistReports({ kind: 'daily', locale }).catch(() => [] as WatchlistReport[]),
      api.watchlistReports({ kind: 'weekly', locale }).catch(() => [] as WatchlistReport[]),
    ])
    let allStocks = await api
      .stocksByIds(
        itemsResult.items.map((it) => it.stock_id),
        locale,
      )
      .catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>(allStocks.map((s) => [s.id, s]))

    return render(
      <WatchlistPage
        items={itemsResult.items}
        itemsTotal={itemsResult.total}
        page={page}
        perPage={PER_PAGE}
        query={q}
        stocks={stockMap}
        dailyReports={dailyReports}
        weeklyReports={weeklyReports}
        tab={tab}
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}

function resolveTab(raw: string | null): Tab {
  if (raw === 'daily' || raw === 'weekly') return raw
  return 'items'
}

interface WatchlistPageProps {
  items: WatchlistItem[]
  itemsTotal: number
  page: number
  perPage: number
  query: string
  stocks: Map<number, Stock>
  dailyReports: WatchlistReport[]
  weeklyReports: WatchlistReport[]
  tab: Tab
  locale: string
  theme: Theme
}

function WatchlistPage() {
  return ({
    items,
    itemsTotal,
    page,
    perPage,
    query,
    stocks,
    dailyReports,
    weeklyReports,
    tab,
    locale,
    theme,
  }: WatchlistPageProps) => {
    let p = messages(locale).pages.watchlist
    let subtitle =
      tab === 'items'
        ? p.subtitleStocks(itemsTotal)
        : tab === 'daily'
          ? p.subtitleReports(dailyReports.length)
          : p.subtitleReports(weeklyReports.length)
    return (
      <Layout
        title={p.title}
        subtitle={subtitle}
        nav={<TabStrip active={tab} locale={locale} />}
        locale={locale}
        theme={theme}
      >
        {tab === 'items' && (
          <ItemsView
            items={items}
            total={itemsTotal}
            page={page}
            perPage={perPage}
            query={query}
            stocks={stocks}
            locale={locale}
          />
        )}
        {tab === 'daily' && (
          <ReportsView reports={dailyReports} kind="daily" locale={locale} />
        )}
        {tab === 'weekly' && (
          <ReportsView reports={weeklyReports} kind="weekly" locale={locale} />
        )}
      </Layout>
    )
  }
}

function TabStrip() {
  return ({ active, locale }: { active: Tab; locale: string }) => {
    let p = messages(locale).pages.watchlist
    return (
      <div
        mix={css({
          display: 'inline-flex',
          gap: space[1],
          padding: '3px',
          background: color.bg,
          borderRadius: radius.pill,
        })}
      >
        <PillTab href={`/watchlists`} label={p.tabItems} active={active === 'items'} />
        <PillTab
          href={`/watchlists?tab=daily`}
          label={p.tabDaily}
          active={active === 'daily'}
        />
        <PillTab
          href={`/watchlists?tab=weekly`}
          label={p.tabWeekly}
          active={active === 'weekly'}
        />
      </div>
    )
  }
}

function ItemsView() {
  return ({
    items,
    total,
    page,
    perPage,
    query,
    stocks,
    locale,
  }: {
    items: WatchlistItem[]
    total: number
    page: number
    perPage: number
    query: string
    stocks: Map<number, Stock>
    locale: string
  }) => {
    let p = messages(locale).pages.watchlist
    let totalPages = Math.max(1, Math.ceil(total / perPage))
    return (
      <>
        <Card>
          <SearchBar
            action="/watchlists"
            locale={locale}
            query={query}
            placeholder={p.columnSymbol}
            extraParams={{ tab: 'items' }}
          />
        </Card>
        <div mix={css({ marginTop: space[4] })}>
        {items.length === 0 ? (
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
              <Th>{p.columnAdded}</Th>
              <Th>{p.columnNotes}</Th>
            </tr>
          </thead>
          <tbody>
            {items.map((it) => {
              let s = stocks.get(it.stock_id)
              return (
                <tr
                  data-row-href={`/stocks/${it.stock_id}`}
                  mix={css({
                    borderTop: `1px solid ${color.borderSoft}`,
                    cursor: 'pointer',
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
                        <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>
                          {s.symbol}
                        </span>
                      </a>
                    ) : (
                      <span mix={css({ color: color.textMuted })}>#{it.stock_id}</span>
                    )}
                  </Td>
                  <Td>{s ? <Badge tone="neutral">{s.market_code}</Badge> : '—'}</Td>
                  <Td>{s?.currency ?? '—'}</Td>
                  <Td>{s?.asset_class ?? '—'}</Td>
                  <Td>
                    <span mix={css({ fontSize: font.sm, color: color.textMuted })}>
                      <LocalTime value={it.added_at} format="date" />
                    </span>
                  </Td>
                  <Td>
                    <span mix={css({ fontSize: font.sm, color: color.textMuted })}>
                      {it.notes ?? '—'}
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
            action="/watchlists"
            locale={locale}
            page={page}
            totalPages={totalPages}
            total={total}
            perPage={perPage}
            query={query}
            extraParams={{ tab: 'items' }}
          />
        )}
      </>
    )
  }
}

/// One of the two report-flavored tabs. The first report renders as a
/// prominent hero card; the rest stack below under an "Older reports"
/// heading. Both share `ReportCard` (the hero pass enables the larger
/// `hero` styling). Reports are always shown with full content
/// inline — this view is dedicated to reading, so no collapse needed.
function ReportsView() {
  return ({
    reports,
    kind,
    locale,
  }: {
    reports: WatchlistReport[]
    kind: 'daily' | 'weekly'
    locale: string
  }) => {
    let p = messages(locale).pages.watchlist
    if (reports.length === 0) {
      return (
        <Card>
          <EmptyState
            title={kind === 'daily' ? p.noDaily : p.noWeekly}
            hint={
              <>
                Agent writes via <code>POST /api/v1/watchlist/reports</code>.
              </>
            }
          />
        </Card>
      )
    }
    let hero = reports[0]
    let rest = reports.slice(1)
    return (
      <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[6] })}>
        <ReportCard report={hero} hero locale={locale} />
        {rest.length > 0 && (
          <section>
            <div
              mix={css({
                display: 'flex',
                alignItems: 'baseline',
                gap: space[2],
                marginBottom: space[3],
              })}
            >
              <SectionTitle>{p.olderReports}</SectionTitle>
              <span mix={css({ fontSize: font.xs, color: color.textDim })}>
                {rest.length}
              </span>
            </div>
            <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[3] })}>
              {rest.map((r) => (
                <ReportCard report={r} locale={locale} />
              ))}
            </div>
          </section>
        )}
      </div>
    )
  }
}

function sentimentTone(s: string | null | undefined): BadgeTone {
  if (!s) return 'neutral'
  if (s === 'bullish' || s === 'positive') return 'success'
  if (s === 'bearish' || s === 'negative') return 'danger'
  return 'neutral'
}

function periodLabel(r: WatchlistReport): string {
  return r.kind === 'weekly' ? `${r.period_start} → ${r.period_end}` : r.period_start
}

/// One report rendered as a card. `hero` flips the larger styling
/// used at the top of a reports tab: thicker accent border, card
/// shadow, a "LATEST" badge, and bigger headline.
function ReportCard() {
  return ({
    report: r,
    hero,
    locale,
  }: {
    report: WatchlistReport
    hero?: boolean
    locale: string
  }) => {
    let p = messages(locale).pages.watchlist
    return (
      <div
        mix={css({
          background: color.surface,
          border: `1px solid ${color.border}`,
          borderLeft: `${hero ? '4px' : '3px'} solid ${color.brand}`,
          borderRadius: radius.lg,
          padding: hero ? `${space[5]} ${space[6]}` : `${space[4]} ${space[5]}`,
          boxShadow: hero ? shadow.card : 'none',
        })}
      >
        <div
          mix={css({
            display: 'flex',
            alignItems: 'baseline',
            gap: space[2],
            marginBottom: space[2],
            fontSize: font.xs,
            color: color.textMuted,
            flexWrap: 'wrap',
          })}
        >
          {hero && <Badge tone="info">{p.latestReport}</Badge>}
          <strong mix={css({ color: color.text })}>{periodLabel(r)}</strong>
          <span mix={css({ textTransform: 'uppercase', letterSpacing: '0.08em' })}>
            {r.kind}
          </span>
          {r.sentiment && <Badge tone={sentimentTone(r.sentiment)}>{r.sentiment}</Badge>}
          <span mix={css({ marginLeft: 'auto' })}>{r.source}</span>
        </div>
        <div
          mix={css({
            fontSize: hero ? font.lg : font.md,
            fontWeight: hero ? 700 : 600,
            color: color.text,
            marginBottom: space[2],
            lineHeight: 1.4,
          })}
        >
          {r.headline ?? '(untitled)'}
        </div>
        {r.summary_md && (
          <div mix={css({ marginBottom: space[2] })}>
            <MarkdownToggle source={r.summary_md} />
          </div>
        )}
        {r.content_md && <MarkdownToggle source={r.content_md} />}
      </div>
    )
  }
}

/// Soft segmented control item — matches the `ChipLink` look used for
/// country / locale chips so the design language stays consistent and
/// the active state reads correctly in both light and dark themes.
function PillTab() {
  return ({ href, label, active }: { href: string; label: string; active: boolean }) => (
    <a
      href={href}
      mix={css({
        display: 'inline-flex',
        alignItems: 'center',
        padding: `${space[1]} ${space[3]}`,
        fontSize: font.sm,
        fontWeight: 600,
        borderRadius: radius.pill,
        textDecoration: 'none',
        color: active ? color.text : color.textMuted,
        background: active ? color.surface : 'transparent',
        boxShadow: active ? shadow.card : 'none',
        transition: 'background 120ms ease, color 120ms ease',
        '&:hover': active ? undefined : { color: color.text },
      })}
    >
      {label}
    </a>
  )
}

function Th() {
  return ({ children }: { children: RemixNode }) => (
    <th
      mix={css({
        textAlign: 'left',
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
  return ({ children }: { children: RemixNode }) => (
    <td mix={css({ padding: `${space[3]} ${space[4]}` })}>{children}</td>
  )
}
