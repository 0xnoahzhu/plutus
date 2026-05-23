import type { BuildAction } from 'remix/fetch-router'

import { api, type Stock } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

export const earningsDetail: BuildAction<
  'GET',
  typeof routes.earningsDetail
> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.earningsEvent(id).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    let stocks = await api.stocksByIds([item.stock_id], locale).catch(() => [])
    let stock: Stock | null = stocks[0] ?? null
    let m = messages(locale).pages.earnings
    let title = stock
      ? `${stock.symbol} · ${item.fiscal_year} ${item.fiscal_period}`
      : `Earnings · ${item.fiscal_year} ${item.fiscal_period}`
    return render(
      <EntityDetailPage
        title={title}
        subtitle={item.announce_date}
        back={{ href: '/earnings', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.status}</Badge>
            <Badge tone="neutral">{item.announce_timing}</Badge>
            <span>
              {item.announce_at ? (
                <LocalTime value={item.announce_at} format="datetime" />
              ) : (
                item.announce_date
              )}
            </span>
            <span>{item.source}</span>
          </>
        }
        sections={[
          { label: 'Guidance', markdown: item.guidance_md },
          { label: 'Notes', markdown: item.notes },
        ]}
        side={
          <MetaList
            items={[
              ['Fiscal period', `${item.fiscal_year} ${item.fiscal_period}`],
              ['Date', item.announce_date],
              ['Timing', item.announce_timing],
              ['Status', item.status],
              ['EPS est.', item.eps_estimate],
              ['EPS actual', item.eps_actual],
              ['Revenue est.', item.revenue_estimate],
              ['Revenue actual', item.revenue_actual],
              ['Currency', item.currency],
              ['Stock', stock?.symbol ?? null],
              [
                'URL',
                item.url ? (
                  <a href={item.url} target="_blank" rel="noopener noreferrer">
                    open
                  </a>
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
