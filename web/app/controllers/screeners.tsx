import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type ScreenerHit, type ScreenerRun, type Stock } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const screeners: BuildAction<'GET', typeof routes.screeners> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let [runs, stocks] = await Promise.all([
      api.screenerRuns(locale).catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    // The most-recent run gets its hits loaded inline. Older runs render as
    // cards with a link to drill into.
    let latest = runs[0]
    let hits: ScreenerHit[] = latest
      ? await api.screenerHits(latest.id, locale).catch(() => [])
      : []
    hits.sort((a, b) => (a.rank ?? 9999) - (b.rank ?? 9999))

    return render(
      <ScreenersPage
        runs={runs}
        latest={latest}
        hits={hits}
        stocks={stockMap}
        locale={locale}
      />,
      request,
      { locale },
    )
  },
}

interface ScreenersProps {
  runs: ScreenerRun[]
  latest: ScreenerRun | undefined
  hits: ScreenerHit[]
  stocks: Map<number, Stock>
  locale: string
}

function ScreenersPage() {
  return ({ runs, latest, hits, stocks, locale }: ScreenersProps) => (
    <Layout title="Screeners" locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Recurring screener runs (weekly value/quality/momentum scans, IPO
        watchlists, etc). Agent writes via{' '}
        <code>POST /api/v1/screener-runs</code> and adds hits to{' '}
        <code>POST /api/v1/screener-runs/:id/hits</code>.
      </p>

      {!latest ? (
        <Empty>
          No screener runs yet. Push one with{' '}
          <code>POST /api/v1/screener-runs</code>.
        </Empty>
      ) : (
        <>
          <SectionHeader label="Latest run" sub={latest.run_date} />
          <RunCard run={latest} hits={hits} stocks={stocks} expanded />

          {runs.length > 1 && (
            <div mix={css({ marginTop: '24px' })}>
              <SectionHeader label="Earlier runs" sub={`${runs.length - 1}`} />
              <div mix={css({ display: 'flex', flexDirection: 'column', gap: '8px' })}>
                {runs.slice(1).map((r) => (
                  <RunCard run={r} hits={[]} stocks={stocks} expanded={false} />
                ))}
              </div>
            </div>
          )}
        </>
      )}
    </Layout>
  )
}

function SectionHeader() {
  return ({ label, sub }: { label: string; sub: string }) => (
    <div
      mix={css({
        display: 'flex',
        alignItems: 'baseline',
        justifyContent: 'space-between',
        marginBottom: '8px',
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
        {label}
      </h3>
      <span mix={css({ fontSize: '11px', color: '#94a3b8' })}>{sub}</span>
    </div>
  )
}

function Empty() {
  return ({ children }: { children: import('remix/ui').RemixNode }) => (
    <p mix={css({ color: '#94a3b8', fontStyle: 'italic', fontSize: '13px' })}>
      {children}
    </p>
  )
}

function RunCard() {
  return ({
    run,
    hits,
    stocks,
    expanded,
  }: {
    run: ScreenerRun
    hits: ScreenerHit[]
    stocks: Map<number, Stock>
    expanded: boolean
  }) => (
    <div
      mix={css({
        background: '#fff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        overflow: 'hidden',
      })}
    >
      <div
        mix={css({
          padding: '12px 16px',
          borderBottom: expanded ? '1px solid #e2e8f0' : 'none',
        })}
      >
        <div
          mix={css({
            display: 'flex',
            alignItems: 'baseline',
            gap: '8px',
            marginBottom: '4px',
            flexWrap: 'wrap',
          })}
        >
          <span
            mix={css({
              fontFamily: 'ui-monospace, monospace',
              fontSize: '13px',
              fontWeight: 600,
              color: '#0f172a',
            })}
          >
            {run.run_date}
          </span>
          <KindPill kind={run.kind} />
          <span
            mix={css({
              fontSize: '11px',
              color: '#64748b',
            })}
          >
            universe: <strong>{run.universe}</strong>
            {run.universe_size != null && ` (n=${run.universe_size})`}
          </span>
          {run.sentiment && <SentimentChip sentiment={run.sentiment} />}
          <span mix={css({ marginLeft: 'auto', fontSize: '11px', color: '#94a3b8' })}>
            {run.source} · {run.language}
          </span>
        </div>
        <div
          mix={css({
            fontSize: '14px',
            fontWeight: 600,
            color: '#0f172a',
            marginBottom: '4px',
          })}
        >
          {run.name}
        </div>
        {run.summary_md && (
          <pre
            mix={css({
              margin: '6px 0 0',
              padding: '8px 10px',
              background: '#f8fafc',
              border: '1px solid #e2e8f0',
              borderRadius: '4px',
              fontSize: '12px',
              lineHeight: 1.55,
              color: '#1f2937',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              fontFamily: 'inherit',
            })}
          >
            {run.summary_md}
          </pre>
        )}
      </div>

      {expanded && (
        <div>
          {hits.length === 0 ? (
            <div
              mix={css({
                padding: '14px 16px',
                fontSize: '12px',
                color: '#94a3b8',
                fontStyle: 'italic',
              })}
            >
              No hits recorded for this run yet.
            </div>
          ) : (
            <table
              mix={css({
                width: '100%',
                borderCollapse: 'collapse',
                fontSize: '13px',
              })}
            >
              <tbody>
                {hits.map((h) => (
                  <HitRow hit={h} stock={stocks.get(h.stock_id)} />
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </div>
  )
}

function HitRow() {
  return ({ hit, stock }: { hit: ScreenerHit; stock: Stock | undefined }) => (
    <tr mix={css({ borderTop: '1px solid #f1f5f9' })}>
      <td
        mix={css({
          padding: '10px 14px',
          width: '50px',
          fontVariantNumeric: 'tabular-nums',
          fontSize: '12px',
          color: '#64748b',
        })}
      >
        {hit.rank != null ? `#${hit.rank}` : ''}
      </td>
      <td mix={css({ padding: '10px 14px', width: '20%' })}>
        {stock ? (
          <a
            href={`/stocks/${stock.id}`}
            mix={css({
              fontFamily: 'ui-monospace, monospace',
              fontWeight: 600,
              color: '#1d4ed8',
              textDecoration: 'none',
              '&:hover': { textDecoration: 'underline' },
            })}
          >
            {stock.symbol}
          </a>
        ) : (
          <span mix={css({ color: '#94a3b8' })}>#{hit.stock_id}</span>
        )}
        {stock && (
          <span mix={css({ marginLeft: '6px', fontSize: '10px', color: '#94a3b8' })}>
            {stock.market_code}
          </span>
        )}
      </td>
      <td
        mix={css({
          padding: '10px 14px',
          width: '80px',
          fontVariantNumeric: 'tabular-nums',
          fontSize: '12px',
          color: '#0f172a',
        })}
      >
        {hit.score ?? ''}
      </td>
      <td mix={css({ padding: '10px 14px', fontSize: '12px', color: '#475569' })}>
        {hit.rationale_md ?? ''}
      </td>
    </tr>
  )
}

function KindPill() {
  return ({ kind }: { kind: string }) => (
    <span
      mix={css({
        padding: '1px 8px',
        borderRadius: '999px',
        background: '#e0e7ff',
        color: '#3730a3',
        fontSize: '10px',
        fontWeight: 600,
      })}
    >
      {kind}
    </span>
  )
}

function SentimentChip() {
  return ({ sentiment }: { sentiment: string }) => {
    let palette: Record<string, [string, string]> = {
      bullish: ['#dcfce7', '#166534'],
      positive: ['#dcfce7', '#166534'],
      bearish: ['#fee2e2', '#991b1b'],
      negative: ['#fee2e2', '#991b1b'],
      neutral: ['#e2e8f0', '#475569'],
    }
    let [bg, fg] = palette[sentiment] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          fontSize: '11px',
          fontWeight: 600,
          background: bg,
          color: fg,
        })}
      >
        {sentiment}
      </span>
    )
  }
}
