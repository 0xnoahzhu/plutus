import type { BuildAction } from 'remix/fetch-router'

import { api, type Stock } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { render } from '../utils/render.tsx'

export const catalystDetail: BuildAction<'GET', typeof routes.catalystDetail> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.catalyst(id, locale).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    let stock: Stock | null = null
    if (item.stock_id != null) {
      let stocks = await api.stocksByIds([item.stock_id], locale).catch(() => [])
      stock = stocks[0] ?? null
    }
    let m = messages(locale).pages.catalysts
    let title = item.title ?? `${item.catalyst_kind} · ${item.catalyst_date}`
    let scope = stock?.symbol ?? item.sector_code ?? item.country ?? '—'
    return render(
      <EntityDetailPage
        title={title}
        subtitle={`${scope} · ${item.catalyst_date}`}
        back={{ href: '/catalysts', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.catalyst_kind}</Badge>
            <Badge tone="neutral">{item.impact_level}</Badge>
            <Badge tone="neutral">{item.status}</Badge>
            <Badge tone="neutral">{item.date_confidence}</Badge>
            <span>{item.source}</span>
          </>
        }
        sections={[
          { label: 'Summary', markdown: item.summary_md },
          { label: 'Bull case', markdown: item.bull_case_md },
          { label: 'Bear case', markdown: item.bear_case_md },
          { label: 'Notes', markdown: item.notes },
        ]}
        side={
          <MetaList
            items={[
              ['Date', item.catalyst_date],
              ['Confidence', item.date_confidence],
              ['Impact', item.impact_level],
              ['Status', item.status],
              ['Stock', stock?.symbol ?? null],
              ['Sector', item.sector_code],
              ['Country', item.country],
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
