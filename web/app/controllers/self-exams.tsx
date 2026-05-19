import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type SelfExam } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const selfExams: BuildAction<'GET', typeof routes.selfExams> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let exams = await api.selfExams({ locale }).catch(() => [])
    exams.sort((a, b) => b.period_start.localeCompare(a.period_start))
    return render(
      <SelfExamsPage exams={exams} locale={locale} />,
      request,
      { locale },
    )
  },
}

interface SelfExamsProps {
  exams: SelfExam[]
  locale: string
}

function SelfExamsPage() {
  return ({ exams, locale }: SelfExamsProps) => (
    <Layout title="Self-exam" locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Reflective reviews by the agent on its own past calls — how previous
        recommendations played out, what went wrong, what to change. Stored via{' '}
        <code>POST /api/v1/self-exams</code>, upserted by (kind, period_start).
      </p>
      {exams.length === 0 ? (
        <p mix={css({ color: '#94a3b8', fontStyle: 'italic', fontSize: '13px' })}>
          No self-exams recorded yet.
        </p>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: '16px' })}>
          {exams.map((e) => (
            <ExamCard exam={e} />
          ))}
        </div>
      )}
    </Layout>
  )
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
      <div
        mix={css({
          background: '#fff',
          border: '1px solid #e2e8f0',
          borderLeft: '3px solid #7c3aed',
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
          <KindPill kind={exam.kind} />
          <span
            mix={css({
              fontSize: '12px',
              color: '#64748b',
              fontVariantNumeric: 'tabular-nums',
            })}
          >
            {exam.period_start} → {exam.period_end}
          </span>
          {recIds.length > 0 && (
            <span
              mix={css({
                fontSize: '11px',
                color: '#64748b',
              })}
            >
              reviewing {recIds.length} recommendation{recIds.length === 1 ? '' : 's'}
            </span>
          )}
          <span mix={css({ marginLeft: 'auto', fontSize: '11px', color: '#94a3b8' })}>
            {exam.source} · {exam.language}
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
          {exam.headline}
        </div>
        {exam.content_md && (
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
            {exam.content_md}
          </pre>
        )}
        {exam.notes && (
          <div
            mix={css({
              marginTop: '8px',
              fontSize: '12px',
              fontStyle: 'italic',
              color: '#64748b',
            })}
          >
            note: {exam.notes}
          </div>
        )}
        {recIds.length > 0 && (
          <div
            mix={css({
              marginTop: '10px',
              display: 'flex',
              gap: '6px',
              flexWrap: 'wrap',
            })}
          >
            {recIds.map((id) => (
              <a
                href={`/recommendations`}
                title={`Recommendation #${id}`}
                mix={css({
                  padding: '2px 8px',
                  borderRadius: '4px',
                  background: '#e0e7ff',
                  color: '#3730a3',
                  fontSize: '11px',
                  fontWeight: 600,
                  textDecoration: 'none',
                  fontFamily: 'ui-monospace, monospace',
                })}
              >
                rec#{id}
              </a>
            ))}
          </div>
        )}
      </div>
    )
  }
}

function KindPill() {
  return ({ kind }: { kind: string }) => {
    let palette: Record<string, [string, string]> = {
      weekly: ['#dbeafe', '#1e40af'],
      monthly: ['#e0e7ff', '#3730a3'],
      quarterly: ['#cffafe', '#155e75'],
      annual: ['#fef3c7', '#92400e'],
      ad_hoc: ['#e2e8f0', '#475569'],
    }
    let [bg, fg] = palette[kind] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          background: bg,
          color: fg,
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
