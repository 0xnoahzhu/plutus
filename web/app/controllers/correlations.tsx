import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import {
  api,
  type CorrelationPair,
  type CorrelationRun,
  type Stock,
  type UniverseDefinition,
} from '../api.ts'
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
  SectionTitle,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { MarkdownToggle } from '../ui/markdown.tsx'
import { render } from '../utils/render.tsx'

const TOP_PAIRS = 30

export const correlations: BuildAction<'GET', typeof routes.correlations> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let [runs, universes] = await Promise.all([
      api.correlationRuns(locale).catch(() => []),
      api.universes().catch(() => []),
    ])
    let universeMap = new Map<number, UniverseDefinition>(universes.map((u) => [u.id, u]))

    let latest = runs[0]
    // Pairs come pre-sorted from the API by |correlation| desc, so the
    // first TOP_PAIRS slice is exactly the strongest-correlated rows.
    let pairs: CorrelationPair[] = latest
      ? await api.correlationPairs(latest.id).catch(() => [])
      : []
    let topPairs = pairs.slice(0, TOP_PAIRS)
    // Resolve symbols only for the stocks referenced in the top pairs
    // (each pair touches two stocks). Avoids the 200-row /stocks cap
    // for users with catalogs in the thousands.
    let pairStockIds = topPairs.flatMap((p) => [p.stock_a_id, p.stock_b_id])
    let stocks = await api.stocksByIds(pairStockIds, locale).catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

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
        theme={theme}
      />,
      request,
      { locale, theme },
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
  theme: Theme
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
    theme,
  }: CorrelationsProps) => {
    let p = messages(locale).pages.correlations
    return (
    <Layout
      title={p.title}
      subtitle={latest ? `${p.sectionLatestRun} ${latest.run_date}` : p.noRunsYetSubtitle}
      locale={locale}
      theme={theme}
    >
      <SectionTitle hint={`${universes.length}`}>{p.sectionUniverses}</SectionTitle>
      {universes.length === 0 ? (
        <Card>
          <EmptyState
            title={p.noUniversesTitle}
            hint={<code>POST /api/v1/universes</code>}
          />
        </Card>
      ) : (
        <UniverseList universes={universes} />
      )}

      <div mix={css({ marginTop: space[6] })}>
        <SectionTitle hint={latest ? latest.run_date : '—'}>
          {p.sectionLatestRun}
        </SectionTitle>
      </div>
      {!latest ? (
        <Card>
          <EmptyState
            title={p.noRunsTitle}
            hint={<code>POST /api/v1/correlation-runs</code>}
          />
        </Card>
      ) : (
        <>
          <RunHeader
            run={latest}
            universe={universeMap.get(latest.universe_id)}
            totalPairs={totalPairs}
          />
          <div mix={css({ marginTop: space[3] })}>
            <PairTable pairs={topPairs} stocks={stocks} totalPairs={totalPairs} />
          </div>
        </>
      )}

      {runs.length > 1 && (
        <div mix={css({ marginTop: space[6] })}>
          <SectionTitle hint={`${runs.length - 1}`}>{p.sectionEarlierRuns}</SectionTitle>
          <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[2] })}>
            {runs.slice(1).map((r) => (
              <RunRow run={r} universe={universeMap.get(r.universe_id)} />
            ))}
          </div>
        </div>
      )}
    </Layout>
    )
  }
}

function UniverseList() {
  return ({ universes }: { universes: UniverseDefinition[] }) => (
    <div
      mix={css({
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
        gap: space[3],
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
          <Card>
            <div
              mix={css({
                fontSize: font.base,
                fontWeight: 600,
                color: color.text,
                marginBottom: space[1],
              })}
            >
              {u.name}
            </div>
            <div mix={css({ fontSize: font.xs, color: color.textMuted })}>
              {n} stock{n === 1 ? '' : 's'}
            </div>
            {u.description_md && (
              <div mix={css({ marginTop: space[2] })}>
                <MarkdownToggle source={u.description_md} />
              </div>
            )}
          </Card>
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
    <Card>
      <div
        mix={css({
          display: 'flex',
          alignItems: 'baseline',
          gap: space[2],
          marginBottom: space[1],
          flexWrap: 'wrap',
        })}
      >
        <span
          mix={css({
            fontFamily: font.mono,
            fontSize: font.base,
            fontWeight: 600,
            color: color.text,
          })}
        >
          {run.run_date}
        </span>
        <Badge tone="brand">{run.kind}</Badge>
        <span mix={css({ fontSize: font.xs, color: color.textMuted })}>
          method: <strong>{run.method}</strong> · lookback{' '}
          <strong>{run.lookback_days}d</strong> · {totalPairs} pair
          {totalPairs === 1 ? '' : 's'}
        </span>
        <span
          mix={css({
            marginLeft: 'auto',
            fontSize: font.xs,
            color: color.textDim,
          })}
        >
          {run.source}
        </span>
      </div>
      <div mix={css({ fontSize: font.sm, color: color.textMuted })}>
        Universe:{' '}
        <strong mix={css({ color: color.text })}>
          {universe ? universe.name : `#${run.universe_id}`}
        </strong>
      </div>
      {run.summary_md && (
        <div mix={css({ marginTop: space[2] })}>
          <MarkdownToggle source={run.summary_md} />
        </div>
      )}
    </Card>
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
        <Card>
          <EmptyState
            title="No pairs recorded for this run yet"
            hint={
              <>
                Push pairwise correlations to{' '}
                <code>/correlation-runs/:id/pairs</code>.
              </>
            }
          />
        </Card>
      )
    }
    return (
      <Card padding="0">
        <div
          mix={css({
            padding: `${space[2]} ${space[4]}`,
            background: color.bg,
            borderBottom: `1px solid ${color.border}`,
            fontSize: font.xs,
            color: color.textMuted,
            textTransform: 'uppercase',
            letterSpacing: '0.08em',
            fontWeight: 600,
          })}
        >
          Top {pairs.length} of {totalPairs} pairs by |ρ|
        </div>
        <table
          mix={css({
            width: '100%',
            borderCollapse: 'collapse',
            fontSize: font.base,
          })}
        >
          <tbody>
            {pairs.map((p) => (
              <tr
                mix={css({
                  borderTop: `1px solid ${color.borderSoft}`,
                  '&:hover td': { background: color.bg },
                })}
              >
                <td mix={css({ padding: `${space[2]} ${space[4]}`, width: '28%' })}>
                  <StockLink id={p.stock_a_id} stock={stocks.get(p.stock_a_id)} />
                </td>
                <td
                  mix={css({
                    padding: `${space[2]} ${space[1]}`,
                    width: '24px',
                    color: color.textDim,
                    textAlign: 'center',
                  })}
                >
                  ↔
                </td>
                <td mix={css({ padding: `${space[2]} ${space[4]}`, width: '28%' })}>
                  <StockLink id={p.stock_b_id} stock={stocks.get(p.stock_b_id)} />
                </td>
                <td mix={css({ padding: `${space[2]} ${space[4]}` })}>
                  <CorrBar value={p.correlation} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </Card>
    )
  }
}

function StockLink() {
  return ({ id, stock }: { id: number; stock: Stock | undefined }) => {
    if (!stock) {
      return (
        <span mix={css({ color: color.textMuted, fontSize: font.sm })}>#{id}</span>
      )
    }
    return (
      <a
        href={`/stocks/${stock.id}`}
        mix={css({
          display: 'inline-flex',
          alignItems: 'center',
          gap: space[2],
          textDecoration: 'none',
          color: color.text,
          '&:hover': { color: color.brandHover },
        })}
      >
        <StockBadge symbol={stock.symbol} size={22} />
        <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>
          {stock.symbol}
        </span>
      </a>
    )
  }
}

function CorrBar() {
  return ({ value }: { value: string }) => {
    let n = parseFloat(value)
    if (!Number.isFinite(n)) {
      return (
        <span mix={css({ fontSize: font.sm, color: color.textMuted })}>{value}</span>
      )
    }
    // Center at 0; bar extends left for negative, right for positive, up to 100px.
    let widthPct = Math.min(100, Math.abs(n) * 100)
    let pos = n >= 0
    let strong = pos ? color.success : color.danger
    let soft = pos ? color.successSoft : color.dangerSoft
    return (
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          gap: space[2],
        })}
      >
        <div
          mix={css({
            flex: 1,
            height: '6px',
            background: color.borderSoft,
            borderRadius: radius.sm,
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
              background: soft,
              borderRight: pos ? `2px solid ${strong}` : undefined,
              borderLeft: pos ? undefined : `2px solid ${strong}`,
            })}
          />
        </div>
        <span
          mix={css({
            width: '56px',
            textAlign: 'right',
            fontVariantNumeric: 'tabular-nums',
            fontFamily: font.mono,
            fontSize: font.sm,
            fontWeight: 600,
            color: strong,
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
        background: color.surface,
        border: `1px solid ${color.border}`,
        borderRadius: radius.md,
        padding: `${space[2]} ${space[4]}`,
        display: 'flex',
        alignItems: 'baseline',
        gap: space[2],
        flexWrap: 'wrap',
      })}
    >
      <span
        mix={css({
          fontFamily: font.mono,
          fontSize: font.sm,
          fontWeight: 600,
          color: color.text,
        })}
      >
        {run.run_date}
      </span>
      <Badge tone="brand">{run.kind}</Badge>
      <span mix={css({ fontSize: font.xs, color: color.textMuted })}>
        {universe ? universe.name : `universe#${run.universe_id}`} · {run.method} ·{' '}
        {run.lookback_days}d
      </span>
      <span
        mix={css({
          marginLeft: 'auto',
          fontSize: font.xs,
          color: color.textDim,
        })}
      >
        {run.source}
      </span>
    </div>
  )
}
