import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import {
  api,
  type Stock,
  type Watchlist,
  type WatchlistItem,
  type WatchlistReport,
} from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
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

// A watchlist is a user-curated cross-market grouping — applying the country
// filter here would defeat its purpose, so the chip is omitted.
function WatchlistDetailPage() {
  return ({ watchlist: w, items, stocks, reports, reportTab, locale }: WatchlistDetailProps) => (
    <Layout title={w.name} locale={locale}>
      <a
        href="/watchlists"
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          textDecoration: 'none',
          '&:hover': { color: '#0f172a' },
        })}
      >
        ← Back to watchlists
      </a>

      {w.description && (
        <p mix={css({ marginTop: '12px', color: '#475569', fontSize: '14px' })}>{w.description}</p>
      )}

      <div
        mix={css({
          marginTop: '12px',
          fontSize: '12px',
          color: '#64748b',
        })}
      >
        {items.length} {items.length === 1 ? 'symbol' : 'symbols'}
      </div>

      {items.length === 0 ? (
        <p mix={css({ marginTop: '16px', color: '#64748b' })}>
          This group is empty. Add a stock with
          <code mix={css({ marginLeft: '4px' })}>{`POST /api/v1/watchlists/${w.id}/items`}</code>
        </p>
      ) : (
        <table
          mix={css({
            marginTop: '16px',
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
              <Th>Added</Th>
              <Th>Notes</Th>
            </tr>
          </thead>
          <tbody>
            {items.map((it) => {
              let s = stocks.get(it.stock_id)
              return (
                <tr mix={css({ borderTop: '1px solid #e2e8f0' })}>
                  <Td>
                    <a
                      href={s ? `/stocks/${s.id}` : '#'}
                      mix={css({
                        fontFamily: 'ui-monospace, SFMono-Regular, monospace',
                        fontWeight: 600,
                        color: '#1d4ed8',
                        textDecoration: 'none',
                        '&:hover': { textDecoration: 'underline' },
                      })}
                    >
                      {s?.symbol ?? `#${it.stock_id}`}
                    </a>
                  </Td>
                  <Td>
                    {s ? (
                      <Badge>{s.market_code}</Badge>
                    ) : (
                      <span mix={css({ color: '#94a3b8' })}>—</span>
                    )}
                  </Td>
                  <Td>{s?.currency ?? '—'}</Td>
                  <Td>{s?.asset_class ?? '—'}</Td>
                  <Td>
                    <span mix={css({ fontSize: '12px', color: '#64748b' })}>
                      {it.added_at.slice(0, 10)}
                    </span>
                  </Td>
                  <Td>
                    <span mix={css({ fontSize: '13px', color: '#475569' })}>
                      {it.notes ?? '—'}
                    </span>
                  </Td>
                </tr>
              )
            })}
          </tbody>
        </table>
      )}

      {/* Reports section */}
      <div mix={css({ marginTop: '24px' })}>
        <ReportsSection watchlistId={w.id} reports={reports} active={reportTab} />
      </div>
    </Layout>
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
    <div>
      <div
        mix={css({
          display: 'flex',
          alignItems: 'baseline',
          gap: '10px',
          marginBottom: '12px',
        })}
      >
        <h3
          mix={css({
            margin: 0,
            fontSize: '12px',
            fontWeight: 700,
            textTransform: 'uppercase',
            letterSpacing: '0.08em',
            color: '#0f172a',
          })}
        >
          Reports
        </h3>
        <div mix={css({ display: 'flex', gap: '6px' })}>
          <ReportTab href={`/watchlists/${watchlistId}?reports=daily`} label="Daily" active={active === 'daily'} />
          <ReportTab href={`/watchlists/${watchlistId}?reports=weekly`} label="Weekly" active={active === 'weekly'} />
        </div>
      </div>
      {reports.length === 0 ? (
        <p mix={css({ color: '#94a3b8', fontStyle: 'italic', fontSize: '13px', margin: 0 })}>
          No {active} reports yet. Agent writes via{' '}
          <code>POST /api/v1/watchlist-reports</code>.
        </p>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: '10px' })}>
          {reports.slice(0, 10).map((r) => (
            <ReportCard report={r} />
          ))}
        </div>
      )}
    </div>
  )
}

function ReportTab() {
  return ({ href, label, active }: { href: string; label: string; active: boolean }) => (
    <a
      href={href}
      mix={css({
        padding: '3px 12px',
        fontSize: '11px',
        fontWeight: 600,
        borderRadius: '6px',
        textDecoration: 'none',
        background: active ? '#0f172a' : '#e2e8f0',
        color: active ? '#fff' : '#475569',
        '&:hover': { background: active ? '#0f172a' : '#cbd5e1' },
      })}
    >
      {label}
    </a>
  )
}

function ReportCard() {
  return ({ report: r }: { report: WatchlistReport }) => {
    let palette: Record<string, [string, string]> = {
      bullish: ['#dcfce7', '#166534'],
      bearish: ['#fee2e2', '#991b1b'],
      neutral: ['#e2e8f0', '#475569'],
    }
    let [bg, fg] = r.sentiment ? palette[r.sentiment] ?? ['#e2e8f0', '#475569'] : ['#e2e8f0', '#475569']
    let periodLabel =
      r.kind === 'weekly'
        ? `${r.period_start} → ${r.period_end}`
        : r.period_start
    return (
      <div
        mix={css({
          background: '#fff',
          border: '1px solid #e2e8f0',
          borderLeft: '3px solid #1d4ed8',
          borderRadius: '8px',
          padding: '14px 18px',
        })}
      >
        <div
          mix={css({
            display: 'flex',
            alignItems: 'baseline',
            gap: '8px',
            marginBottom: '6px',
            fontSize: '11px',
            color: '#64748b',
          })}
        >
          <strong mix={css({ color: '#0f172a' })}>{periodLabel}</strong>
          <span mix={css({ textTransform: 'uppercase', letterSpacing: '0.06em' })}>
            {r.kind}
          </span>
          {r.sentiment && (
            <span
              mix={css({
                padding: '1px 8px',
                borderRadius: '4px',
                background: bg,
                color: fg,
                fontSize: '10px',
                fontWeight: 700,
                textTransform: 'uppercase',
              })}
            >
              {r.sentiment}
              {r.sentiment_score ? ` ${r.sentiment_score}` : ''}
            </span>
          )}
          <span mix={css({ marginLeft: 'auto' })}>
            {r.source} · {r.language}
          </span>
        </div>
        <div
          mix={css({
            fontSize: '14px',
            fontWeight: 600,
            color: '#0f172a',
            marginBottom: '6px',
            lineHeight: 1.4,
          })}
        >
          {r.headline}
        </div>
        {r.summary_md && (
          <p
            mix={css({
              margin: '0 0 8px',
              fontSize: '13px',
              color: '#475569',
              lineHeight: 1.55,
              fontStyle: 'italic',
            })}
          >
            {r.summary_md}
          </p>
        )}
        {r.content_md && (
          <pre
            mix={css({
              margin: '8px 0 0',
              padding: '10px 12px',
              background: '#f8fafc',
              border: '1px solid #e2e8f0',
              borderRadius: '4px',
              fontSize: '13px',
              lineHeight: 1.6,
              color: '#1f2937',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              fontFamily: 'inherit',
            })}
          >
            {r.content_md}
          </pre>
        )}
        {r.metrics && (
          <details mix={css({ marginTop: '8px', fontSize: '12px', color: '#64748b' })}>
            <summary mix={css({ cursor: 'pointer' })}>metrics</summary>
            <pre
              mix={css({
                margin: '6px 0 0',
                padding: '8px 10px',
                background: '#f1f5f9',
                borderRadius: '4px',
                fontSize: '11px',
                fontFamily: 'ui-monospace, monospace',
                overflowX: 'auto',
                whiteSpace: 'pre-wrap',
              })}
            >
              {r.metrics}
            </pre>
          </details>
        )}
      </div>
    )
  }
}

function Th() {
  return ({ children }: { children: string }) => (
    <th
      mix={css({
        textAlign: 'left',
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
  return ({ children }: { children: import('remix/ui').RemixNode }) => (
    <td
      mix={css({
        padding: '10px 14px',
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
