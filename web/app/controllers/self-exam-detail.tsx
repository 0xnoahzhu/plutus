import type { BuildAction } from 'remix/fetch-router'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { render } from '../utils/render.tsx'

export const selfExamDetail: BuildAction<'GET', typeof routes.selfExamDetail> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.selfExam(id, locale).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    let m = messages(locale).pages.selfExams
    let title = item.headline ?? `${item.kind} exam · ${item.period_start}`
    return render(
      <EntityDetailPage
        title={title}
        subtitle={`${item.period_start} → ${item.period_end}`}
        back={{ href: '/self-exams', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.kind}</Badge>
            <span>{item.source}</span>
          </>
        }
        sections={[
          { label: 'Reflection', markdown: item.content_md },
          { label: 'Notes', markdown: item.notes },
        ]}
        side={
          <MetaList
            items={[
              ['Kind', item.kind],
              ['Period', `${item.period_start} → ${item.period_end}`],
              ['Metrics', item.metrics],
              ['Recommendation IDs', item.recommendation_ids],
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
