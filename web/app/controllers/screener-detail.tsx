import type { BuildAction } from 'remix/fetch-router'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { LocalTime } from '../ui/local-time.tsx'
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
        }
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}
