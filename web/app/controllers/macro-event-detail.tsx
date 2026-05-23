import type { BuildAction } from 'remix/fetch-router'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { render } from '../utils/render.tsx'

export const macroEventDetail: BuildAction<
  'GET',
  typeof routes.macroEventDetail
> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.macroEvent(id, locale).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    let m = messages(locale).pages.macroEvents
    let title = item.title ?? `${item.indicator_code} · ${item.event_date}`
    return render(
      <EntityDetailPage
        title={title}
        subtitle={`${item.indicator_code} · ${item.event_kind} · ${item.event_date}`}
        back={{ href: '/macro-events', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.indicator_code}</Badge>
            <Badge tone="neutral">{item.event_kind}</Badge>
            {item.decision && <Badge tone="neutral">{item.decision}</Badge>}
            <span>{item.source}</span>
          </>
        }
        sections={[{ label: 'Summary', markdown: item.summary_md }]}
        side={
          <MetaList
            items={[
              ['Date', item.event_date],
              ['Kind', item.event_kind],
              ['Decision', item.decision],
              [
                'Decision bps',
                item.decision_bps != null ? String(item.decision_bps) : null,
              ],
              ['New value', item.new_value],
              ['Consensus', item.consensus_estimate],
              ['Surprise', item.surprise],
              ['Previous', item.previous_value],
              ['Vote', item.vote],
              ['Dot plot', item.dot_plot],
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
