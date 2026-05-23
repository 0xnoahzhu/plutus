import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type SelfExam } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
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
  UnreadDot,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const selfExams: BuildAction<'GET', typeof routes.selfExams> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let exams = await api.selfExams({ locale }).catch(() => [])
    exams.sort((a, b) => b.period_start.localeCompare(a.period_start))
    return render(
      <SelfExamsPage exams={exams} locale={locale} theme={theme} />,
      request,
      { locale, theme },
    )
  },
}

interface SelfExamsProps {
  exams: SelfExam[]
  locale: string
  theme: Theme
}

function SelfExamsPage() {
  return ({ exams, locale, theme }: SelfExamsProps) => {
    let p = messages(locale).pages.selfExams
    return (
    <Layout
      title={p.title}
      subtitle={`${exams.length} ${exams.length === 1 ? 'entry' : 'entries'}`}
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
        Reflective reviews by the agent on its own past calls — how previous
        recommendations played out, what went wrong, what to change. Stored via{' '}
        <code>POST /api/v1/self-exams</code>, upserted by (kind, period_start).
      </p>
      {exams.length === 0 ? (
        <Card>
          <EmptyState
            title={p.emptyTitle}
            hint={<code>POST /api/v1/self-exams</code>}
          />
        </Card>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[4] })}>
          {exams.map((e) => (
            <ExamCard exam={e} />
          ))}
        </div>
      )}
    </Layout>
    )
  }
}

function ExamCard() {
  return ({ exam }: { exam: SelfExam }) => {
    let recIds: number[] = []
    if (exam.recommendation_ids) {
      try {
        let parsed = JSON.parse(exam.recommendation_ids)
        if (Array.isArray(parsed)) recIds = parsed
      } catch {}
    }
    return (
      <a
        href={`/self-exams/${exam.id}`}
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
          <UnreadDot readAt={exam.read_at} />
          <Badge tone="brand">{exam.kind}</Badge>
          <span
            mix={css({
              fontSize: font.sm,
              color: color.textMuted,
              fontVariantNumeric: 'tabular-nums',
            })}
          >
            {exam.period_start} → {exam.period_end}
          </span>
          {recIds.length > 0 && (
            <span mix={css({ fontSize: font.xs, color: color.textMuted })}>
              {recIds.length} rec{recIds.length === 1 ? '' : 's'}
            </span>
          )}
          <span
            mix={css({
              marginLeft: 'auto',
              fontSize: font.xs,
              color: color.textDim,
            })}
          >
            {exam.source}
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
          {exam.headline ?? '(untitled)'}
        </div>
      </a>
    )
  }
}
