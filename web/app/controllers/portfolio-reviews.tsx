import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type PortfolioReview } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const portfolioReviews: BuildAction<'GET', typeof routes.portfolioReviews> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let reviews = await api.portfolioReviews(locale).catch(() => [])
    // Newest first.
    reviews.sort((a, b) => b.period_start.localeCompare(a.period_start))
    return render(
      <PortfolioReviewsPage reviews={reviews} locale={locale} />,
      request,
      { locale },
    )
  },
}

interface ReviewsProps {
  reviews: PortfolioReview[]
  locale: string
}

function PortfolioReviewsPage() {
  return ({ reviews, locale }: ReviewsProps) => (
    <Layout title="Portfolio reviews" locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Weekly and monthly portfolio reviews from the agent. Each review covers
        a period (week or month) and includes a headline, summary, full content,
        and explicit decisions. Agent writes via{' '}
        <code>POST /api/v1/portfolio-reviews</code> — upsert by (kind,
        period_start).
      </p>
      {reviews.length === 0 ? (
        <p mix={css({ color: '#94a3b8', fontStyle: 'italic', fontSize: '13px' })}>
          No reviews recorded yet.
        </p>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: '16px' })}>
          {reviews.map((r) => (
            <ReviewCard review={r} />
          ))}
        </div>
      )}
    </Layout>
  )
}

function ReviewCard() {
  return ({ review }: { review: PortfolioReview }) => (
    <div
      mix={css({
        background: '#fff',
        border: '1px solid #e2e8f0',
        borderLeft: `3px solid ${kindAccent(review.kind)}`,
        borderRadius: '8px',
        padding: '16px 20px',
      })}
    >
      <div
        mix={css({
          display: 'flex',
          alignItems: 'baseline',
          gap: '8px',
          marginBottom: '8px',
          flexWrap: 'wrap',
        })}
      >
        <KindPill kind={review.kind} />
        <span
          mix={css({
            fontSize: '12px',
            color: '#64748b',
            fontVariantNumeric: 'tabular-nums',
          })}
        >
          {review.period_start} → {review.period_end}
        </span>
        {review.sentiment && <SentimentChip sentiment={review.sentiment} />}
        <span mix={css({ marginLeft: 'auto', fontSize: '11px', color: '#94a3b8' })}>
          {review.source} · {review.language}
        </span>
      </div>
      <div
        mix={css({
          fontSize: '16px',
          fontWeight: 600,
          color: '#0f172a',
          marginBottom: '10px',
          lineHeight: 1.4,
        })}
      >
        {review.headline}
      </div>
      {review.summary_md && (
        <Block label="Summary" body={review.summary_md} accent="#475569" />
      )}
      {review.content_md && (
        <Block label="Full content" body={review.content_md} accent="#1f2937" />
      )}
      {review.decisions_md && (
        <Block label="Decisions" body={review.decisions_md} accent="#7c3aed" />
      )}
    </div>
  )
}

function Block() {
  return ({
    label,
    body,
    accent,
  }: {
    label: string
    body: string
    accent: string
  }) => (
    <div mix={css({ marginTop: '10px' })}>
      <div
        mix={css({
          fontSize: '10px',
          fontWeight: 700,
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          color: accent,
          marginBottom: '4px',
        })}
      >
        {label}
      </div>
      <pre
        mix={css({
          margin: 0,
          padding: '10px 12px',
          background: '#f8fafc',
          border: '1px solid #e2e8f0',
          borderRadius: '4px',
          fontSize: '13px',
          lineHeight: 1.6,
          color: '#1f2937',
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

function kindAccent(kind: string): string {
  return kind === 'weekly'
    ? '#1d4ed8'
    : kind === 'monthly'
      ? '#7c3aed'
      : kind === 'quarterly'
        ? '#0891b2'
        : '#64748b'
}

function KindPill() {
  return ({ kind }: { kind: string }) => {
    let bg = kindAccent(kind)
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          background: bg,
          color: '#fff',
          fontSize: '10px',
          fontWeight: 700,
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
        })}
      >
        {kind}
      </span>
    )
  }
}

function SentimentChip() {
  return ({ sentiment }: { sentiment: string }) => {
    let palette: Record<string, [string, string]> = {
      bullish: ['#dcfce7', '#166534'],
      positive: ['#dcfce7', '#166534'],
      bearish: ['#fee2e2', '#991b1b'],
      negative: ['#fee2e2', '#991b1b'],
      neutral: ['#e2e8f0', '#475569'],
      cautious: ['#fef3c7', '#92400e'],
    }
    let [bg, fg] = palette[sentiment] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          fontSize: '11px',
          fontWeight: 600,
          background: bg,
          color: fg,
        })}
      >
        {sentiment}
      </span>
    )
  }
}
