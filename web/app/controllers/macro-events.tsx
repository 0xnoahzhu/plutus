import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import { api, type MacroEvent, type MacroIndicator } from '../api.ts'
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
  parseCountry,
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  type Theme,
} from '../ui/layout.tsx'
import { MarkdownToggle } from '../ui/markdown.tsx'
import { render } from '../utils/render.tsx'

interface DayGroup {
  date: string
  events: MacroEvent[]
}

export const macroEvents: BuildAction<'GET', typeof routes.macroEvents> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)

    let [events, indicators] = await Promise.all([
      api.macroEvents(country, locale).catch(() => []),
      api.macroIndicators().catch(() => []),
    ])
    let indicatorMap = new Map<string, MacroIndicator>(
      indicators.map((i) => [i.code, i]),
    )

    let today = new Date().toISOString().slice(0, 10)
    let upcoming = group(events.filter((e) => e.event_date >= today))
    let past = group(events.filter((e) => e.event_date < today)).reverse()

    return render(
      <MacroEventsPage
        upcoming={upcoming}
        past={past}
        indicators={indicatorMap}
        country={country}
        locale={locale}
        theme={theme}
        today={today}
      />,
      request,
      { locale, theme },
    )
  },
}

function group(events: MacroEvent[]): DayGroup[] {
  let by = new Map<string, DayGroup>()
  for (let e of events) {
    let g = by.get(e.event_date)
    if (!g) {
      g = { date: e.event_date, events: [] }
      by.set(e.event_date, g)
    }
    g.events.push(e)
  }
  for (let g of by.values()) {
    g.events.sort((a, b) => a.indicator_code.localeCompare(b.indicator_code))
  }
  return Array.from(by.values()).sort((a, b) => a.date.localeCompare(b.date))
}

interface MacroEventsProps {
  upcoming: DayGroup[]
  past: DayGroup[]
  indicators: Map<string, MacroIndicator>
  country: string
  locale: string
  theme: Theme
  today: string
}

function MacroEventsPage() {
  return ({ upcoming, past, indicators, country, locale, theme, today }: MacroEventsProps) => {
    let p = messages(locale).pages.macroEvents
    return (
    <Layout
      title={p.title}
      subtitle={`Discrete macro and policy events for ${country}`}
      country={country}
      locale={locale}
      theme={theme}
    >
      <SectionTitle hint={`from ${today}`}>Upcoming</SectionTitle>
      {upcoming.length === 0 ? (
        <Card>
          <EmptyState
            title="No upcoming events"
            hint={
              <>
                Agent writes via <code>POST /api/v1/macro/events</code>. Daily
                continuous series (yields, effective rates) live in{' '}
                <code>/api/v1/macro/observations</code>.
              </>
            }
          />
        </Card>
      ) : (
        <DayList groups={upcoming} indicators={indicators} />
      )}

      <div mix={css({ marginTop: space[6] })}>
        <SectionTitle hint={`before ${today}`}>Past</SectionTitle>
      </div>
      {past.length === 0 ? (
        <Card>
          <EmptyState title="No past events recorded" />
        </Card>
      ) : (
        <DayList groups={past} indicators={indicators} />
      )}
    </Layout>
    )
  }
}

function DayList() {
  return ({
    groups,
    indicators,
  }: {
    groups: DayGroup[]
    indicators: Map<string, MacroIndicator>
  }) => (
    <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[3] })}>
      {groups.map((g) => (
        <Card padding="0">
          <div
            mix={css({
              padding: `${space[2]} ${space[4]}`,
              fontSize: font.sm,
              fontWeight: 600,
              color: color.text,
              background: color.bg,
              borderBottom: `1px solid ${color.borderSoft}`,
              borderRadius: `${radius.lg} ${radius.lg} 0 0`,
            })}
          >
            {g.date}
          </div>
          <div>
            {g.events.map((e) => (
              <EventRow event={e} indicator={indicators.get(e.indicator_code)} />
            ))}
          </div>
        </Card>
      ))}
    </div>
  )
}

function EventRow() {
  return ({ event, indicator }: { event: MacroEvent; indicator: MacroIndicator | undefined }) => {
    let isReleased = event.new_value != null
    return (
      <div
        mix={css({
          padding: `${space[3]} ${space[4]}`,
          borderTop: `1px solid ${color.borderSoft}`,
          '&:first-child': { borderTop: 'none' },
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
          <code
            mix={css({
              fontSize: font.xs,
              fontFamily: font.mono,
              color: color.brandSoftText,
              fontWeight: 600,
            })}
          >
            {event.indicator_code}
          </code>
          <Badge tone={eventKindTone(event.event_kind)}>{event.event_kind}</Badge>
          {event.decision && (
            <DecisionBadge decision={event.decision} bps={event.decision_bps} />
          )}
          {indicator && (
            <span mix={css({ fontSize: font.xs, color: color.textDim })}>
              {indicator.unit} · {indicator.frequency}
            </span>
          )}
          <span
            mix={css({
              marginLeft: 'auto',
              fontSize: font.xs,
              color: color.textDim,
            })}
          >
            {event.source}
          </span>
        </div>
        <div
          mix={css({
            fontSize: font.base,
            fontWeight: 600,
            color: color.text,
            marginBottom: space[1],
            lineHeight: 1.4,
          })}
        >
          {event.title ?? '(untitled)'}
        </div>
        {isReleased && (
          <div
            mix={css({
              display: 'flex',
              gap: space[4],
              flexWrap: 'wrap',
              fontSize: font.sm,
              color: color.textMuted,
              fontVariantNumeric: 'tabular-nums',
              marginTop: space[2],
            })}
          >
            <StatItem label="actual" value={event.new_value ?? '—'} strong />
            {event.consensus_estimate && (
              <StatItem label="consensus" value={event.consensus_estimate} />
            )}
            {event.previous_value && (
              <StatItem label="prev" value={event.previous_value} />
            )}
            {event.surprise && (
              <SurpriseBadge surprise={event.surprise} unit={indicator?.unit} />
            )}
            {event.vote && <StatItem label="vote" value={event.vote} />}
          </div>
        )}
        {event.summary_md && (
          <div mix={css({ marginTop: space[2] })}>
            <MarkdownToggle source={event.summary_md} />
          </div>
        )}
      </div>
    )
  }
}

function eventKindTone(kind: string): BadgeTone {
  // Policy decisions get a warning tone — they're rate-setting actions.
  if (
    kind === 'fomc_decision' ||
    kind === 'ecb_decision' ||
    kind === 'boj_decision' ||
    kind === 'lpr_decision'
  ) {
    return 'warn'
  }
  // Data releases get an info tone.
  if (
    kind === 'cpi_release' ||
    kind === 'ppi_release' ||
    kind === 'nfp_release' ||
    kind === 'gdp_release' ||
    kind === 'pmi_release'
  ) {
    return 'info'
  }
  return 'neutral'
}

function DecisionBadge() {
  return ({ decision, bps }: { decision: string; bps: number | null }) => {
    let tone: BadgeTone =
      decision === 'hike' ? 'danger'
      : decision === 'cut' ? 'success'
      : decision === 'hold' ? 'neutral'
      : 'neutral'
    let label =
      decision === 'hike' && bps
        ? `+${bps} bps`
        : decision === 'cut' && bps
          ? `${bps} bps`
          : decision === 'hold'
            ? 'HOLD'
            : decision
    return <Badge tone={tone}>{label}</Badge>
  }
}

function StatItem() {
  return ({
    label,
    value,
    strong,
  }: {
    label: string
    value: RemixNode
    strong?: boolean
  }) => (
    <span>
      <span
        mix={css({
          fontSize: font.xs,
          textTransform: 'uppercase',
          letterSpacing: '0.06em',
          color: color.textDim,
          marginRight: space[1],
        })}
      >
        {label}
      </span>
      <strong
        mix={css({
          color: strong ? color.text : color.textMuted,
          fontWeight: strong ? 700 : 600,
        })}
      >
        {value}
      </strong>
    </span>
  )
}

function SurpriseBadge() {
  return ({ surprise, unit }: { surprise: string; unit?: string }) => {
    let n = parseFloat(surprise)
    let tone: BadgeTone = n > 0 ? 'success' : n < 0 ? 'danger' : 'neutral'
    let sign = n > 0 ? '+' : ''
    return (
      <Badge tone={tone} title="surprise = actual − consensus">
        Δ {sign}
        {surprise}
        {unit ? ` ${unit}` : ''}
      </Badge>
    )
  }
}
