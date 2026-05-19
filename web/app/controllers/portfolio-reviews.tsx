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
  radius,
  resolveLocale,
  resolveTheme,
  space,
  type Theme,
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
      <p
        mix={css({
          fontSize: font.sm,
          color: color.textMuted,
          marginTop: 0,
          marginBottom: space[4],
          lineHeight: 1.55,
        })}
      >
        Weekly and monthly portfolio reviews from the agent. Each review covers
        a period (week or month) and includes a headline, summary, full content,
        and explicit decisions. Agent writes via{' '}
        <code>POST /api/v1/portfolio-reviews</code> — upsert by (kind,
        period_start).
      </p>
      {reviews.length === 0 ? (
        <Card>
          <EmptyState
            title="No reviews recorded yet"
            hint={
              <>
                Agent writes via <code>POST /api/v1/portfolio-reviews</code>.
              </>
            }
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
    <div
      mix={css({
        background: color.surface,
        border: `1px solid ${color.border}`,
        borderLeft: `3px solid ${color.brand}`,
        borderRadius: radius.lg,
        padding: `${space[4]} ${space[5]}`,
      })}
    >
      <div
        mix={css({
          display: 'flex',
          alignItems: 'baseline',
          gap: space[2],
          marginBottom: space[2],
          flexWrap: 'wrap',
        })}
      >
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
          marginBottom: space[2],
          lineHeight: 1.4,
        })}
      >
        {review.headline ?? '(untitled)'}
      </div>
      {review.summary_md && <Block label="Summary" body={review.summary_md} />}
      {review.content_md && <Block label="Full content" body={review.content_md} />}
      {review.decisions_md && <Block label="Decisions" body={review.decisions_md} />}
    </div>
  )
}

function Block() {
  return ({ label, body }: { label: string; body: string }) => (
    <div mix={css({ marginTop: space[3] })}>
      <div
        mix={css({
          fontSize: font.xs,
          fontWeight: 700,
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          color: color.textMuted,
          marginBottom: space[1],
        })}
      >
        {label}
      </div>
      <pre
        mix={css({
          margin: 0,
          padding: `${space[2]} ${space[3]}`,
          background: color.bg,
          border: `1px solid ${color.borderSoft}`,
          borderRadius: radius.md,
          fontSize: font.sm,
          lineHeight: 1.6,
          color: color.text,
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-word',
          fontFamily: 'inherit',
        })}
      >
        {body}
      </pre>
    </div>
  )
}

function sentimentTone(s: string): BadgeTone {
  if (s === 'positive' || s === 'bullish') return 'success'
  if (s === 'negative' || s === 'bearish') return 'danger'
  if (s === 'cautious') return 'warn'
  return 'neutral'
}
