import type { BuildAction } from 'remix/fetch-router'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Badge, resolveLocale, resolveTheme } from '../ui/layout.tsx'
import { EntityDetailPage, MetaList } from '../ui/entity-detail.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

export const portfolioReviewDetail: BuildAction<
  'GET',
  typeof routes.portfolioReviewDetail
> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let item = await api.portfolioReview(id, locale).catch(() => null)
    if (!item) return new Response('Not found', { status: 404 })
    let m = messages(locale).pages.portfolioReviews
    let title = item.headline ?? `${item.kind} review · ${item.period_start}`
    return render(
      <EntityDetailPage
        title={title}
        subtitle={`${item.period_start} → ${item.period_end}`}
        back={{ href: '/portfolio-reviews', label: m.title }}
        meta={
          <>
            <Badge tone="brand">{item.kind}</Badge>
            {item.sentiment && <Badge tone="neutral">{item.sentiment}</Badge>}
            <span>{item.source}</span>
            <span>
              <LocalTime value={item.updated_at} format="datetime" />
            </span>
          </>
        }
        sections={[
          { label: 'Summary', markdown: item.summary_md },
          { label: 'Review', markdown: item.content_md },
          { label: 'Decisions', markdown: item.decisions_md },
        ]}
        side={
          <MetaList
            items={[
              ['Period', `${item.period_start} → ${item.period_end}`],
              ['Sentiment', item.sentiment],
              ['Sentiment score', item.sentiment_score],
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
