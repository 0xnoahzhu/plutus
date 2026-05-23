import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type ScreenerHit, type Stock } from '../api.ts'
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
import { renderMarkdown } from '../ui/markdown.tsx'
import { render } from '../utils/render.tsx'

export const screenerDetail: BuildAction<
  'GET',
  typeof routes.screenerDetail
> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.screenerRun(id, locale).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    let hits = await api.screenerHits(id, locale).catch(() => [] as ScreenerHit[])
    hits.sort((a, b) => (a.rank ?? 9999) - (b.rank ?? 9999))
    let stocks = await api
      .stocksByIds(hits.map((h) => h.stock_id), locale)
      .catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    let m = messages(locale).pages.screeners
    let title = `${item.name} · ${item.run_date}`
    return render(
      <EntityDetailPage
        title={title}
        subtitle={`${item.kind} · universe: ${item.universe}`}
        back={{ href: '/screeners', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.kind}</Badge>
            {item.sentiment && <Badge tone="neutral">{item.sentiment}</Badge>}
            <span>
              <LocalTime value={item.updated_at} format="datetime" />
            </span>
            <span>{item.source}</span>
          </>
        }
        sections={[
          { label: 'Description', markdown: item.description_md },
          { label: 'Summary', markdown: item.summary_md },
        ]}
        side={
          <>
            <MetaList
              items={[
                ['Name', item.name],
                ['Kind', item.kind],
                ['Run date', item.run_date],
                ['Universe', item.universe],
                [
                  'Universe size',
                  item.universe_size != null ? String(item.universe_size) : null,
                ],
                ['Criteria', item.criteria],
                ['Sentiment', item.sentiment],
                ['Source', item.source],
              ]}
            />
            <HitsCard hits={hits} stocks={stockMap} locale={locale} />
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

function HitsCard() {
  return ({
    hits,
    stocks,
    locale,
  }: {
    hits: ScreenerHit[]
    stocks: Map<number, Stock>
    locale: string
  }) => {
    let p = messages(locale).pages.screeners
    if (hits.length === 0) {
      return (
        <Card>
          <EmptyState title={p.noHitsTitle} />
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
          Hits ({hits.length})
        </div>
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
      </Card>
    )
  }
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
      <td mix={css({ ...cellBase, width: '40%' })}>
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
        {hit.rationale_md ? renderMarkdown(hit.rationale_md) : ''}
      </td>
    </tr>
  )
}
