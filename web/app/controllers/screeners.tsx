import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type ScreenerHit, type ScreenerRun, type Stock } from '../api.ts'
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
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const screeners: BuildAction<'GET', typeof routes.screeners> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
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
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}

interface ScreenersProps {
  runs: ScreenerRun[]
  latest: ScreenerRun | undefined
  hits: ScreenerHit[]
  stocks: Map<number, Stock>
  locale: string
  theme: Theme
}

function ScreenersPage() {
  return ({ runs, latest, hits, stocks, locale, theme }: ScreenersProps) => (
    <Layout
      title="Screeners"
      subtitle="Recurring screener runs — weekly value/quality/momentum scans, IPO watchlists, and ad-hoc filters."
      locale={locale}
      theme={theme}
    >
      {!latest ? (
        <Card>
          <EmptyState
            title="No screener runs yet"
            hint={
              <>
                Push one with <code>POST /api/v1/screener-runs</code>.
              </>
            }
          />
        </Card>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[6] })}>
          <div>
            <SectionTitle hint={latest.run_date}>Latest run</SectionTitle>
            <RunCard run={latest} hits={hits} stocks={stocks} expanded />
          </div>

          {runs.length > 1 && (
            <div>
              <SectionTitle hint={`${runs.length - 1}`}>Earlier runs</SectionTitle>
              <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[2] })}>
                {runs.slice(1).map((r) => (
                  <RunCard run={r} hits={[]} stocks={stocks} expanded={false} />
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </Layout>
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
    <Card padding="0">
      <div mix={css({ padding: `${space[4]} ${space[5]}` })}>
        <div
          mix={css({
            display: 'flex',
            alignItems: 'baseline',
            gap: space[2],
            marginBottom: space[2],
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
          <span
            mix={css({
              fontSize: font.xs,
              color: color.textMuted,
            })}
          >
            universe:{' '}
            <strong mix={css({ color: color.text })}>{run.universe}</strong>
            {run.universe_size != null && ` (n=${run.universe_size})`}
          </span>
          {run.sentiment && (
            <Badge tone={sentimentTone(run.sentiment)}>{run.sentiment}</Badge>
          )}
          <span
            mix={css({
              marginLeft: 'auto',
              fontSize: font.xs,
              color: color.textDim,
            })}
          >
            {run.source} · {run.language}
          </span>
        </div>
        <div
          mix={css({
            fontSize: font.md,
            fontWeight: 600,
            color: color.text,
          })}
        >
          {run.name}
        </div>
        {run.summary_md && (
          <pre
            mix={css({
              margin: `${space[2]} 0 0`,
              padding: `${space[2]} ${space[3]}`,
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
            {run.summary_md}
          </pre>
        )}
      </div>

      {expanded && (
        <div mix={css({ borderTop: `1px solid ${color.border}` })}>
          {hits.length === 0 ? (
            <EmptyState title="No hits recorded for this run yet" />
          ) : (
            <table
              mix={css({
                width: '100%',
                borderCollapse: 'collapse',
                fontSize: font.base,
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
    </Card>
  )
}

function HitRow() {
  let cellBase = {
    padding: `${space[3]} ${space[4]}`,
    fontVariantNumeric: 'tabular-nums',
    color: color.text,
    verticalAlign: 'top',
  } as const
  return ({ hit, stock }: { hit: ScreenerHit; stock: Stock | undefined }) => (
    <tr mix={css({ borderTop: `1px solid ${color.borderSoft}` })}>
      <td
        mix={css({
          ...cellBase,
          width: '64px',
          fontFamily: font.mono,
          fontSize: font.sm,
          color: color.textMuted,
        })}
      >
        {hit.rank != null ? `#${hit.rank}` : ''}
      </td>
      <td mix={css({ ...cellBase, width: '24%' })}>
        {stock ? (
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
            <span mix={css({ fontSize: font.xs, color: color.textDim })}>
              {stock.market_code}
            </span>
          </a>
        ) : (
          <span mix={css({ color: color.textMuted })}>#{hit.stock_id}</span>
        )}
      </td>
      <td
        mix={css({
          ...cellBase,
          width: '90px',
          fontFamily: font.mono,
          fontSize: font.sm,
          textAlign: 'right',
        })}
      >
        {hit.score ?? ''}
      </td>
      <td mix={css({ ...cellBase, fontSize: font.sm, color: color.textMuted })}>
        {hit.rationale_md ?? ''}
      </td>
    </tr>
  )
}

function sentimentTone(s: string): BadgeTone {
  if (s === 'positive' || s === 'bullish') return 'success'
  if (s === 'negative' || s === 'bearish') return 'danger'
  return 'neutral'
}
