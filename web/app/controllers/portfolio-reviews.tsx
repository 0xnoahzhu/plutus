import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type PortfolioReview } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  type BadgeTone,
  Card,
  color,
  EmptyState,
  font,
  Layout,
  MarkAllReadStrip,
  PageIntro,
  radius,
  resolveLocale,
  resolveTheme,
  space,
  type Theme,
  UnreadDot,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const portfolioReviews: BuildAction<'GET', typeof routes.portfolioReviews> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let reviews = await api.portfolioReviews(locale).catch(() => [])
    // Newest first.
    reviews.sort((a, b) => b.period_start.localeCompare(a.period_start))
    return render(
      <PortfolioReviewsPage reviews={reviews} locale={locale} theme={theme} />,
      request,
      { locale, theme },
    )
  },
}

interface ReviewsProps {
  reviews: PortfolioReview[]
  locale: string
  theme: Theme
}

function PortfolioReviewsPage() {
  return ({ reviews, locale, theme }: ReviewsProps) => {
    let p = messages(locale).pages.portfolioReviews
    return (
    <Layout
      title={p.title}
      subtitle={`${reviews.length} ${reviews.length === 1 ? 'review' : 'reviews'}`}
      locale={locale}
      theme={theme}
    >
      <PageIntro>
        Weekly and monthly portfolio reviews from the agent. Each review covers
        a period (week or month) and includes a headline, summary, full content,
        and explicit decisions. Agent writes via{' '}
        <code>POST /api/v1/portfolio-reviews</code> — upsert by (kind,
        period_start).
      </PageIntro>
      <MarkAllReadStrip kind="portfolio_review" />
      {reviews.length === 0 ? (
        <Card>
          <EmptyState
            title={p.emptyTitle}
            hint={<code>POST /api/v1/portfolio-reviews</code>}
          />
        </Card>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[4] })}>
          {reviews.map((r) => (
            <ReviewCard review={r} />
          ))}
        </div>
      )}
    </Layout>
    )
  }
}

function ReviewCard() {
  return ({ review }: { review: PortfolioReview }) => (
    <a
      href={`/portfolio-reviews/${review.id}`}
      mix={css({
        display: 'block',
        background: color.surface,
        border: `1px solid ${color.border}`,
        borderLeft: `3px solid ${color.brand}`,
        borderRadius: radius.lg,
        padding: `${space[4]} ${space[5]}`,
        textDecoration: 'none',
        color: 'inherit',
        transition: 'border-color 120ms ease, transform 120ms ease',
        '&:hover': {
          borderColor: color.brand,
          transform: 'translateY(-1px)',
        },
      })}
    >
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          gap: space[2],
          marginBottom: space[2],
          flexWrap: 'wrap',
        })}
      >
        <UnreadDot readAt={review.read_at} />
        <Badge tone="brand">{review.kind}</Badge>
        <span
          mix={css({
            fontSize: font.sm,
            color: color.textMuted,
            fontVariantNumeric: 'tabular-nums',
          })}
        >
          {review.period_start} → {review.period_end}
        </span>
        {review.sentiment && (
          <Badge tone={sentimentTone(review.sentiment)}>{review.sentiment}</Badge>
        )}
        <span
          mix={css({
            marginLeft: 'auto',
            fontSize: font.xs,
            color: color.textDim,
          })}
        >
          {review.source}
        </span>
      </div>
      <div
        mix={css({
          fontSize: font.md,
          fontWeight: 600,
          color: color.text,
          lineHeight: 1.4,
        })}
      >
        {review.headline ?? '(untitled)'}
      </div>
    </a>
  )
}

function sentimentTone(s: string): BadgeTone {
  if (s === 'positive' || s === 'bullish') return 'success'
  if (s === 'negative' || s === 'bearish') return 'danger'
  if (s === 'cautious') return 'warn'
  return 'neutral'
}
