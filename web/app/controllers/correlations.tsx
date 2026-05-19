import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import {
  api,
  type CorrelationPair,
  type CorrelationRun,
  type Stock,
  type UniverseDefinition,
} from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

const TOP_PAIRS = 30

export const correlations: BuildAction<'GET', typeof routes.correlations> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let [runs, universes, stocks] = await Promise.all([
      api.correlationRuns(locale).catch(() => []),
      api.universes().catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    let universeMap = new Map<number, UniverseDefinition>(universes.map((u) => [u.id, u]))

    let latest = runs[0]
    let pairs: CorrelationPair[] = latest
      ? await api.correlationPairs(latest.id).catch(() => [])
      : []
    // Sort by absolute correlation so the most-correlated pairs surface first.
    pairs.sort((a, b) => Math.abs(parseFloat(b.correlation)) - Math.abs(parseFloat(a.correlation)))
    let topPairs = pairs.slice(0, TOP_PAIRS)

    return render(
      <CorrelationsPage
        runs={runs}
        latest={latest}
        topPairs={topPairs}
        totalPairs={pairs.length}
        universes={universes}
        universeMap={universeMap}
        stocks={stockMap}
        locale={locale}
      />,
      request,
      { locale },
    )
  },
}

interface CorrelationsProps {
  runs: CorrelationRun[]
  latest: CorrelationRun | undefined
  topPairs: CorrelationPair[]
  totalPairs: number
  universes: UniverseDefinition[]
  universeMap: Map<number, UniverseDefinition>
  stocks: Map<number, Stock>
  locale: string
}

function CorrelationsPage() {
  return ({
    runs,
    latest,
    topPairs,
    totalPairs,
    universes,
    universeMap,
    stocks,
    locale,
  }: CorrelationsProps) => (
    <Layout title="Correlation map" locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Recurring correlation runs over user-defined universes. Define a universe
        with <code>POST /api/v1/universes</code>, kick off a run with{' '}
        <code>POST /api/v1/correlation-runs</code>, and push the pairwise
        correlations to <code>/correlation-runs/:id/pairs</code>.
      </p>

      <SectionHeader label="Universes" sub={`${universes.length}`} />
      {universes.length === 0 ? (
        <Empty>No universes defined yet.</Empty>
      ) : (
        <UniverseList universes={universes} />
      )}

      <div mix={css({ marginTop: '24px' })}>
        <SectionHeader label="Latest run" sub={latest ? latest.run_date : 'none'} />
      </div>
      {!latest ? (
        <Empty>No correlation runs yet.</Empty>
      ) : (
        <>
          <RunHeader run={latest} universe={universeMap.get(latest.universe_id)} totalPairs={totalPairs} />
          <PairTable pairs={topPairs} stocks={stocks} totalPairs={totalPairs} />
        </>
      )}

      {runs.length > 1 && (
        <div mix={css({ marginTop: '24px' })}>
          <SectionHeader label="Earlier runs" sub={`${runs.length - 1}`} />
          <div mix={css({ display: 'flex', flexDirection: 'column', gap: '6px' })}>
            {runs.slice(1).map((r) => (
              <RunRow run={r} universe={universeMap.get(r.universe_id)} />
            ))}
          </div>
        </div>
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
  return ({ children }: { children: string }) => (
    <p mix={css({ color: '#94a3b8', fontStyle: 'italic', fontSize: '13px', margin: 0 })}>
      {children}
    </p>
  )
}

function UniverseList() {
  return ({ universes }: { universes: UniverseDefinition[] }) => (
    <div
      mix={css({
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))',
        gap: '8px',
      })}
    >
      {universes.map((u) => {
        // stock_ids is JSON-encoded — parse defensively, fall back to 0.
        let n = 0
        try {
          let parsed = JSON.parse(u.stock_ids)
          if (Array.isArray(parsed)) n = parsed.length
        } catch {}
        return (
          <div
            mix={css({
              background: '#fff',
              border: '1px solid #e2e8f0',
              borderRadius: '8px',
              padding: '10px 14px',
            })}
          >
            <div
              mix={css({
                fontSize: '13px',
                fontWeight: 600,
                color: '#0f172a',
                marginBottom: '2px',
              })}
            >
              {u.name}
            </div>
            <div mix={css({ fontSize: '11px', color: '#64748b' })}>
              {n} stock{n === 1 ? '' : 's'}
            </div>
            {u.description_md && (
              <div
                mix={css({
                  marginTop: '6px',
                  fontSize: '12px',
                  color: '#475569',
                  lineHeight: 1.45,
                })}
              >
                {u.description_md}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}

function RunHeader() {
  return ({
    run,
    universe,
    totalPairs,
  }: {
    run: CorrelationRun
    universe: UniverseDefinition | undefined
    totalPairs: number
  }) => (
    <div
      mix={css({
        background: '#fff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        padding: '12px 16px',
        marginBottom: '8px',
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
        <span mix={css({ fontSize: '11px', color: '#64748b' })}>
          method: <strong>{run.method}</strong> · lookback{' '}
          <strong>{run.lookback_days}d</strong> · {totalPairs} pair
          {totalPairs === 1 ? '' : 's'}
        </span>
        <span mix={css({ marginLeft: 'auto', fontSize: '11px', color: '#94a3b8' })}>
          {run.source}
        </span>
      </div>
      <div mix={css({ fontSize: '12px', color: '#475569' })}>
        Universe: <strong>{universe ? universe.name : `#${run.universe_id}`}</strong>
      </div>
      {run.summary_md && (
        <pre
          mix={css({
            margin: '8px 0 0',
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
  )
}

function PairTable() {
  return ({
    pairs,
    stocks,
    totalPairs,
  }: {
    pairs: CorrelationPair[]
    stocks: Map<number, Stock>
    totalPairs: number
  }) => {
    if (pairs.length === 0) {
      return (
        <p mix={css({ fontSize: '12px', color: '#94a3b8', fontStyle: 'italic' })}>
          No pairs recorded for this run yet.
        </p>
      )
    }
    return (
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
            padding: '6px 14px',
            background: '#f8fafc',
            borderBottom: '1px solid #e2e8f0',
            fontSize: '11px',
            color: '#64748b',
          })}
        >
          Top {pairs.length} of {totalPairs} pairs by |ρ|
        </div>
        <table
          mix={css({
            width: '100%',
            borderCollapse: 'collapse',
            fontSize: '13px',
          })}
        >
          <tbody>
            {pairs.map((p) => (
              <tr mix={css({ borderTop: '1px solid #f1f5f9' })}>
                <td mix={css({ padding: '8px 14px', width: '25%' })}>
                  <StockLink id={p.stock_a_id} stock={stocks.get(p.stock_a_id)} />
                </td>
                <td
                  mix={css({
                    padding: '8px 4px',
                    width: '20px',
                    color: '#94a3b8',
                    textAlign: 'center',
                  })}
                >
                  ↔
                </td>
                <td mix={css({ padding: '8px 14px', width: '25%' })}>
                  <StockLink id={p.stock_b_id} stock={stocks.get(p.stock_b_id)} />
                </td>
                <td mix={css({ padding: '8px 14px' })}>
                  <CorrBar value={p.correlation} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    )
  }
}

function StockLink() {
  return ({ id, stock }: { id: number; stock: Stock | undefined }) => {
    if (!stock) {
      return <span mix={css({ color: '#94a3b8', fontSize: '12px' })}>#{id}</span>
    }
    return (
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
    )
  }
}

function CorrBar() {
  return ({ value }: { value: string }) => {
    let n = parseFloat(value)
    if (!Number.isFinite(n)) {
      return <span mix={css({ fontSize: '12px', color: '#94a3b8' })}>{value}</span>
    }
    // Center at 0; bar extends left for negative, right for positive, up to 100px.
    let widthPct = Math.min(100, Math.abs(n) * 100)
    let pos = n >= 0
    let color = pos ? '#166534' : '#991b1b'
    let bg = pos ? '#dcfce7' : '#fee2e2'
    return (
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
        })}
      >
        <div
          mix={css({
            flex: 1,
            height: '6px',
            background: '#f1f5f9',
            borderRadius: '3px',
            position: 'relative',
            overflow: 'hidden',
          })}
        >
          <div
            mix={css({
              position: 'absolute',
              top: 0,
              left: pos ? '50%' : `${50 - widthPct / 2}%`,
              width: `${widthPct / 2}%`,
              height: '100%',
              background: bg,
              borderRight: pos ? `2px solid ${color}` : undefined,
              borderLeft: pos ? undefined : `2px solid ${color}`,
            })}
          />
        </div>
        <span
          mix={css({
            width: '56px',
            textAlign: 'right',
            fontVariantNumeric: 'tabular-nums',
            fontSize: '12px',
            fontWeight: 600,
            color,
          })}
        >
          {n >= 0 ? '+' : ''}
          {n.toFixed(3)}
        </span>
      </div>
    )
  }
}

function RunRow() {
  return ({
    run,
    universe,
  }: {
    run: CorrelationRun
    universe: UniverseDefinition | undefined
  }) => (
    <div
      mix={css({
        background: '#fff',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        padding: '8px 14px',
        display: 'flex',
        alignItems: 'baseline',
        gap: '8px',
        flexWrap: 'wrap',
      })}
    >
      <span
        mix={css({
          fontFamily: 'ui-monospace, monospace',
          fontSize: '12px',
          fontWeight: 600,
          color: '#0f172a',
        })}
      >
        {run.run_date}
      </span>
      <KindPill kind={run.kind} />
      <span mix={css({ fontSize: '11px', color: '#64748b' })}>
        {universe ? universe.name : `universe#${run.universe_id}`} · {run.method} ·{' '}
        {run.lookback_days}d
      </span>
      <span mix={css({ marginLeft: 'auto', fontSize: '11px', color: '#94a3b8' })}>
        {run.source}
      </span>
    </div>
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
