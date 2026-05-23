import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type CorrelationPair, type Stock } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  Card,
  color,
  EmptyState,
  font,
  radius,
  resolveLocale,
  resolveTheme,
  space,
  StockBadge,
} from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

const TOP_PAIRS = 50

export const correlationDetail: BuildAction<
  'GET',
  typeof routes.correlationDetail
> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.correlationRun(id, locale).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    // Pairs come pre-sorted from the API by |correlation| desc.
    let pairs = await api.correlationPairs(id).catch(() => [] as CorrelationPair[])
    let topPairs = pairs.slice(0, TOP_PAIRS)
    let stockIds = topPairs.flatMap((p) => [p.stock_a_id, p.stock_b_id])
    let stocks = await api.stocksByIds(stockIds, locale).catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    let m = messages(locale).pages.correlations
    let title = `${item.kind} correlation · ${item.run_date}`
    return render(
      <EntityDetailPage
        title={title}
        subtitle={`${item.method} · ${item.lookback_days}d lookback · ${pairs.length} pairs`}
        back={{ href: '/correlations', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.kind}</Badge>
            <Badge tone="neutral">{item.method}</Badge>
            <span>
              <LocalTime value={item.updated_at} format="datetime" />
            </span>
            <span>{item.source}</span>
          </>
        }
        sections={[{ label: 'Summary', markdown: item.summary_md }]}
        side={
          <>
            <MetaList
              items={[
                ['Run date', item.run_date],
                ['Kind', item.kind],
                ['Method', item.method],
                ['Lookback', `${item.lookback_days} days`],
                ['Universe id', String(item.universe_id)],
                ['Metrics', item.metrics],
                ['Source', item.source],
              ]}
            />
            <PairsCard
              pairs={topPairs}
              totalPairs={pairs.length}
              stocks={stockMap}
              locale={locale}
            />
          </>
        }
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}

function PairsCard() {
  return ({
    pairs,
    totalPairs,
    stocks,
    locale,
  }: {
    pairs: CorrelationPair[]
    totalPairs: number
    stocks: Map<number, Stock>
    locale: string
  }) => {
    let p = messages(locale).pages.correlations
    if (pairs.length === 0) {
      return (
        <Card>
          <EmptyState title={p.noPairsTitle} />
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
            borderTopLeftRadius: radius.lg,
            borderTopRightRadius: radius.lg,
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
                })}
              >
                <td mix={css({ padding: `${space[2]} ${space[4]}`, width: '32%' })}>
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
                <td mix={css({ padding: `${space[2]} ${space[4]}`, width: '32%' })}>
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
