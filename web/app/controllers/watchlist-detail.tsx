import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import {
  api,
  type Stock,
  type Watchlist,
  type WatchlistItem,
  type WatchlistReport,
} from '../api.ts'
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
  SectionTitle,
  space,
  StockBadge,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const watchlistDetail: BuildAction<'GET', typeof routes.watchlistDetail> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) {
      return new Response('Bad watchlist id', { status: 400 })
    }
    let url = new URL(request.url)
    let reportTab: 'daily' | 'weekly' =
      url.searchParams.get('reports') === 'weekly' ? 'weekly' : 'daily'
    let locale = resolveLocale(request, url.searchParams)

    let [list, items, allStocks, reports] = await Promise.all([
      api.watchlist(id).catch(() => null),
      api.watchlistItems(id).catch(() => [] as WatchlistItem[]),
      api.stocks().catch(() => []),
      api
        .watchlistReportsFor(id, { kind: reportTab, locale })
        .catch(() => [] as WatchlistReport[]),
    ])
    if (!list) {
      return new Response('Watchlist not found', { status: 404 })
    }
    let stockMap = new Map<number, Stock>(allStocks.map((s) => [s.id, s]))

    return render(
      <WatchlistDetailPage
        watchlist={list}
        items={items}
        stocks={stockMap}
        reports={reports}
        reportTab={reportTab}
        locale={locale}
      />,
      request,
      { locale },
    )
  },
}

interface WatchlistDetailProps {
  watchlist: Watchlist
  items: WatchlistItem[]
  stocks: Map<number, Stock>
  reports: WatchlistReport[]
  reportTab: 'daily' | 'weekly'
  locale: string
}

function WatchlistDetailPage() {
  return ({ watchlist: w, items, stocks, reports, reportTab, locale }: WatchlistDetailProps) => (
    <Layout
      title={w.name}
      subtitle={w.description ?? `${items.length} ${items.length === 1 ? 'symbol' : 'symbols'}`}
      locale={locale}
    >
      <Breadcrumb name={w.name} />

      <div mix={css({ marginTop: space[4] })}>
        {items.length === 0 ? (
          <Card>
            <EmptyState
              title="No symbols in this group"
              hint={
                <>
                  Add one with <code>{`POST /api/v1/watchlists/${w.id}/items`}</code>.
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
                  <Th>Symbol</Th>
                  <Th>Market</Th>
                  <Th>Currency</Th>
                  <Th>Asset class</Th>
                  <Th>Added</Th>
                  <Th>Notes</Th>
                </tr>
              </thead>
              <tbody>
                {items.map((it) => {
                  let s = stocks.get(it.stock_id)
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
                          {it.added_at.slice(0, 10)}
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

      <div mix={css({ marginTop: space[6] })}>
        <ReportsSection
          watchlistId={w.id}
          reports={reports}
          active={reportTab}
        />
      </div>
    </Layout>
  )
}

function Breadcrumb() {
  return ({ name }: { name: string }) => (
    <div
      mix={css({
        display: 'flex',
        alignItems: 'center',
        gap: space[2],
        fontSize: font.sm,
        color: color.textMuted,
      })}
    >
      <a
        href="/watchlists"
        mix={css({
          color: color.textMuted,
          textDecoration: 'none',
          '&:hover': { color: color.text },
        })}
      >
        Watchlists
      </a>
      <span>·</span>
      <span mix={css({ color: color.text, fontWeight: 500 })}>{name}</span>
    </div>
  )
}

function ReportsSection() {
  return ({
    watchlistId,
    reports,
    active,
  }: {
    watchlistId: number
    reports: WatchlistReport[]
    active: 'daily' | 'weekly'
  }) => (
    <Card>
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: space[3],
          flexWrap: 'wrap',
          gap: space[2],
        })}
      >
        <SectionTitle>Reports</SectionTitle>
        <div mix={css({ display: 'inline-flex', gap: space[1] })}>
          <Tab
            href={`/watchlists/${watchlistId}?reports=daily`}
            label="Daily"
            active={active === 'daily'}
          />
          <Tab
            href={`/watchlists/${watchlistId}?reports=weekly`}
            label="Weekly"
            active={active === 'weekly'}
          />
        </div>
      </div>
      {reports.length === 0 ? (
        <EmptyState
          title={`No ${active} reports`}
          hint={
            <>
              Agent writes via <code>POST /api/v1/watchlist-reports</code>.
            </>
          }
        />
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[3] })}>
          {reports.slice(0, 10).map((r) => (
            <ReportCard report={r} />
          ))}
        </div>
      )}
    </Card>
  )
}

function Tab() {
  return ({ href, label, active }: { href: string; label: string; active: boolean }) => (
    <a
      href={href}
      mix={css({
        padding: `${space[1]} ${space[3]}`,
        fontSize: font.xs,
        fontWeight: 600,
        borderRadius: radius.md,
        textDecoration: 'none',
        background: active ? color.text : color.bg,
        color: active ? '#fff' : color.textMuted,
        '&:hover': { background: active ? color.text : color.hover },
      })}
    >
      {label}
    </a>
  )
}

function ReportCard() {
  return ({ report: r }: { report: WatchlistReport }) => {
    let toneMap: Record<string, BadgeTone> = {
      bullish: 'success',
      positive: 'success',
      bearish: 'danger',
      negative: 'danger',
      neutral: 'neutral',
    }
    let tone = r.sentiment ? toneMap[r.sentiment] ?? 'neutral' : 'neutral'
    let periodLabel =
      r.kind === 'weekly' ? `${r.period_start} → ${r.period_end}` : r.period_start
    return (
      <div
        mix={css({
          background: color.surface,
          border: `1px solid ${color.border}`,
          borderLeft: `3px solid ${color.brand}`,
          borderRadius: radius.lg,
          padding: `${space[4]} ${space[5]}`,
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
          <strong mix={css({ color: color.text })}>{periodLabel}</strong>
          <span mix={css({ textTransform: 'uppercase', letterSpacing: '0.08em' })}>
            {r.kind}
          </span>
          {r.sentiment && <Badge tone={tone}>{r.sentiment}</Badge>}
          <span mix={css({ marginLeft: 'auto' })}>
            {r.source} · {r.language}
          </span>
        </div>
        <div
          mix={css({
            fontSize: font.md,
            fontWeight: 600,
            color: color.text,
            marginBottom: space[2],
            lineHeight: 1.4,
          })}
        >
          {r.headline}
        </div>
        {r.summary_md && (
          <p
            mix={css({
              margin: `0 0 ${space[2]}`,
              fontSize: font.sm,
              color: color.textMuted,
              lineHeight: 1.55,
            })}
          >
            {r.summary_md}
          </p>
        )}
        {r.content_md && (
          <pre
            mix={css({
              margin: `${space[2]} 0 0`,
              padding: `${space[3]} ${space[3]}`,
              background: color.bg,
              border: `1px solid ${color.borderSoft}`,
              borderRadius: radius.md,
              fontSize: font.sm,
              lineHeight: 1.6,
              color: color.text,
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              fontFamily: 'inherit',
            })}
          >
            {r.content_md}
          </pre>
        )}
      </div>
    )
  }
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
