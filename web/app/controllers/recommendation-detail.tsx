import type { BuildAction } from 'remix/fetch-router'

import { api, type Stock } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import {
  EntityDetailPage,
  MetaList,
} from '../ui/entity-detail.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

export const recommendationDetail: BuildAction<
  'GET',
  typeof routes.recommendationDetail
> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.recommendation(id, locale).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    let stock: Stock | null = null
    if (item.stock_id != null) {
      let stocks = await api.stocksByIds([item.stock_id], locale).catch(() => [])
      stock = stocks[0] ?? null
    }
    let m = messages(locale).pages.recommendations
    let title = stock
      ? `${stock.symbol} · ${item.action}`
      : item.sector_code
        ? `${item.sector_code} · ${item.action}`
        : item.action
    return render(
      <EntityDetailPage
        title={title}
        subtitle={undefined}
        back={{ href: '/recommendations', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.action}</Badge>
            {item.confidence && <span>conf {item.confidence}</span>}
            <span>horizon: {item.target_horizon}</span>
            <Badge tone="neutral">{item.status}</Badge>
            <span>
              issued <LocalTime value={item.issued_at} format="date" /> · {item.source}
            </span>
          </>
        }
        sections={[
          { label: 'Rationale', markdown: item.rationale_md },
          { label: 'Outcome', markdown: item.outcome_md },
        ]}
        side={
          <MetaList
            items={[
              ['Target price', item.target_price],
              ['Currency', item.target_currency],
              ['Horizon', item.target_horizon],
              ['PnL %', item.pnl_pct],
              [
                'Closed at',
                item.closed_at ? (
                  <LocalTime value={item.closed_at} format="datetime" />
                ) : null,
              ],
              ['Source', item.source],
            ]}
          />
        }
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}
