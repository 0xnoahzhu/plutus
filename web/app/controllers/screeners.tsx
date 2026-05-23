import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type ScreenerRun } from '../api.ts'
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

export const screeners: BuildAction<'GET', typeof routes.screeners> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let runs = await api.screenerRuns(locale).catch(() => [])
    return render(
      <ScreenersPage runs={runs} locale={locale} theme={theme} />,
      request,
      { locale, theme },
    )
  },
}

interface ScreenersProps {
  runs: ScreenerRun[]
  locale: string
  theme: Theme
}

function ScreenersPage() {
  return ({ runs, locale, theme }: ScreenersProps) => {
    let p = messages(locale).pages.screeners
    return (
      <Layout title={p.title} locale={locale} theme={theme}>
        <PageIntro>{p.subtitle}</PageIntro>
        <MarkAllReadStrip kind="screener_run" />
        {runs.length === 0 ? (
          <Card>
            <EmptyState
              title={p.noRunsTitle}
              hint={<code>POST /api/v1/screener-runs</code>}
            />
          </Card>
        ) : (
          <div
            mix={css({ display: 'flex', flexDirection: 'column', gap: space[2] })}
          >
            {runs.map((r) => (
              <RunCard run={r} />
            ))}
          </div>
        )}
      </Layout>
    )
  }
}

function RunCard() {
  return ({ run }: { run: ScreenerRun }) => (
    <a
      href={`/screeners/${run.id}`}
      mix={css({
        display: 'block',
        background: color.surface,
        border: `1px solid ${color.border}`,
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
        <UnreadDot readAt={run.read_at} />
        <span
          mix={css({
            fontFamily: font.mono,
            fontSize: font.sm,
            fontWeight: 600,
            color: color.text,
          })}
        >
          {run.run_date}
        </span>
        <Badge tone="brand">{run.kind}</Badge>
        <span mix={css({ fontSize: font.xs, color: color.textMuted })}>
          universe:{' '}
          <strong mix={css({ color: color.text })}>{run.universe}</strong>
          {run.universe_size != null && ` (n=${run.universe_size})`}
        </span>
        {run.sentiment && (
          <Badge tone={sentimentTone(run.sentiment)}>{run.sentiment}</Badge>
        )}
        <span
          mix={css({
            marginLeft: 'auto',
            fontSize: font.xs,
            color: color.textDim,
          })}
        >
          {run.source}
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
        {run.name}
      </div>
    </a>
  )
}

function sentimentTone(s: string): BadgeTone {
  if (s === 'positive' || s === 'bullish') return 'success'
  if (s === 'negative' || s === 'bearish') return 'danger'
  return 'neutral'
}
