import type { BuildAction } from 'remix/fetch-router'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

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
    let m = messages(locale).pages.correlations
    let title = `${item.kind} correlation · ${item.run_date}`
    return render(
      <EntityDetailPage
        title={title}
        subtitle={`${item.method} · ${item.lookback_days}d lookback`}
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
        }
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}
