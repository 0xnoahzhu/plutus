import type { BuildAction } from 'remix/fetch-router'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

export const briefDetail: BuildAction<'GET', typeof routes.briefDetail> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.marketBrief(id, locale).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    let m = messages(locale).pages.briefs
    let title = item.headline ?? `${item.country} ${item.kind} · ${item.trade_date}`
    return render(
      <EntityDetailPage
        title={title}
        subtitle={`${item.country} · ${item.kind} · ${item.trade_date}`}
        back={{ href: '/briefs', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.country}</Badge>
            <Badge tone="neutral">{item.kind}</Badge>
            {item.sentiment && <Badge tone="neutral">{item.sentiment}</Badge>}
            <span>
              <LocalTime value={item.updated_at} format="datetime" />
            </span>
          </>
        }
        sections={[{ label: undefined, markdown: item.content_md }]}
        side={
          <MetaList
            items={[
              ['Country', item.country],
              ['Kind', item.kind],
              ['Trade date', item.trade_date],
              ['Sentiment', item.sentiment],
              ['Sentiment score', item.sentiment_score],
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
