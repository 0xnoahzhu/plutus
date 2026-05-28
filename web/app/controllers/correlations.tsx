import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import {
  api,
  type CorrelationRun,
  type UniverseDefinition,
} from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  Card,
  color,
  EmptyState,
  font,
  Layout,
  MarkAllReadStrip,
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  type Theme,
  unreadCardStyle,
  UnreadDot,
} from '../ui/layout.tsx'
import { MarkdownToggle } from '../ui/markdown.tsx'
import { render } from '../utils/render.tsx'

export const correlations: BuildAction<'GET', typeof routes.correlations> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let [runs, universes] = await Promise.all([
      api.correlationRuns(locale).catch(() => []),
      api.universes().catch(() => []),
    ])
    let universeMap = new Map<number, UniverseDefinition>(universes.map((u) => [u.id, u]))

    return render(
      <CorrelationsPage
        runs={runs}
        universes={universes}
        universeMap={universeMap}
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}

interface CorrelationsProps {
  runs: CorrelationRun[]
  universes: UniverseDefinition[]
  universeMap: Map<number, UniverseDefinition>
  locale: string
  theme: Theme
}

function CorrelationsPage() {
  return ({
    runs,
    universes,
    universeMap,
    locale,
    theme,
  }: CorrelationsProps) => {
    let p = messages(locale).pages.correlations
    return (
    <Layout
      title={p.title}
      subtitle={runs[0] ? `${runs.length} run${runs.length === 1 ? '' : 's'}` : p.noRunsYetSubtitle}
      locale={locale}
      theme={theme}
    >
      <MarkAllReadStrip kind="correlation_run" />
      <SectionTitle hint={`${universes.length}`}>{p.sectionUniverses}</SectionTitle>
      {universes.length === 0 ? (
        <Card>
          <EmptyState
            title={p.noUniversesTitle}
            hint={<code>POST /api/v1/universes</code>}
          />
        </Card>
      ) : (
        <UniverseList universes={universes} />
      )}

      <div mix={css({ marginTop: space[6] })}>
        <SectionTitle hint={`${runs.length}`}>{p.sectionLatestRun}</SectionTitle>
      </div>
      {runs.length === 0 ? (
        <Card>
          <EmptyState
            title={p.noRunsTitle}
            hint={<code>POST /api/v1/correlation-runs</code>}
          />
        </Card>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[2] })}>
          {runs.map((r) => (
            <RunRow run={r} universe={universeMap.get(r.universe_id)} />
          ))}
        </div>
      )}
    </Layout>
    )
  }
}

function UniverseList() {
  return ({ universes }: { universes: UniverseDefinition[] }) => (
    <div
      mix={css({
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
        gap: space[3],
      })}
    >
      {universes.map((u) => {
        // stock_ids is JSON-encoded — parse defensively, fall back to 0.
        let n = 0
        try {
          let parsed = JSON.parse(u.stock_ids)
          if (Array.isArray(parsed)) n = parsed.length
        } catch {}
        return (
          <Card>
            <div
              mix={css({
                fontSize: font.base,
                fontWeight: 600,
                color: color.text,
                marginBottom: space[1],
              })}
            >
              {u.name}
            </div>
            <div mix={css({ fontSize: font.xs, color: color.textMuted })}>
              {n} stock{n === 1 ? '' : 's'}
            </div>
            {u.description_md && (
              <div mix={css({ marginTop: space[2] })}>
                <MarkdownToggle source={u.description_md} />
              </div>
            )}
          </Card>
        )
      })}
    </div>
  )
}

function RunRow() {
  return ({
    run,
    universe,
  }: {
    run: CorrelationRun
    universe: UniverseDefinition | undefined
  }) => (
    <a
      href={`/correlations/${run.id}`}
      mix={css({
        ...unreadCardStyle(run.read_at),
        borderRadius: radius.md,
        padding: `${space[3]} ${space[4]}`,
        display: 'flex',
        alignItems: 'center',
        gap: space[2],
        flexWrap: 'wrap',
        textDecoration: 'none',
        color: 'inherit',
        transition: 'border-color 120ms ease, transform 120ms ease',
        '&:hover': {
          borderColor: color.brand,
          transform: 'translateY(-1px)',
        },
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
        {universe ? universe.name : `universe#${run.universe_id}`} · {run.method} ·{' '}
        {run.lookback_days}d
      </span>
      <span
        mix={css({
          marginLeft: 'auto',
          fontSize: font.xs,
          color: color.textDim,
        })}
      >
        {run.source}
      </span>
    </a>
  )
}
