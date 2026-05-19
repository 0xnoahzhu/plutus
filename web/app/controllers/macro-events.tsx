import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type MacroEvent, type MacroIndicator } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, parseCountry, resolveLocale } from '../ui/layout.tsx'
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
        today={today}
      />,
      request,
      { locale },
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
  today: string
}

function MacroEventsPage() {
  return ({ upcoming, past, indicators, country, locale, today }: MacroEventsProps) => (
    <Layout title="Macro calendar" country={country} locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Discrete macro / policy events for <strong>{country}</strong> — FOMC
        decisions, CPI releases, LPR moves, etc. Agent writes via{' '}
        <code>POST /api/v1/macro/events</code>. Daily continuous series (yields,
        effective rates) live in <code>/api/v1/macro/observations</code>.
      </p>

      <Section label="Upcoming" sub={`from ${today}`}>
        {upcoming.length === 0 ? (
          <Empty>No upcoming events.</Empty>
        ) : (
          <DayList groups={upcoming} indicators={indicators} />
        )}
      </Section>

      <Section label="Past" sub={`before ${today}`}>
        {past.length === 0 ? (
          <Empty>No past events recorded.</Empty>
        ) : (
          <DayList groups={past} indicators={indicators} />
        )}
      </Section>
    </Layout>
  )
}

function Section() {
  return ({
    label,
    sub,
    children,
  }: {
    label: string
    sub: string
    children: import('remix/ui').RemixNode
  }) => (
    <div mix={css({ marginTop: '16px' })}>
      <div
        mix={css({
          display: 'flex',
          alignItems: 'baseline',
          justifyContent: 'space-between',
          marginBottom: '8px',
        })}
      >
        <h3
          mix={css({
            margin: 0,
            fontSize: '12px',
            fontWeight: 700,
            textTransform: 'uppercase',
            letterSpacing: '0.08em',
            color: '#0f172a',
          })}
        >
          {label}
        </h3>
        <span mix={css({ fontSize: '11px', color: '#94a3b8' })}>{sub}</span>
      </div>
      {children}
    </div>
  )
}

function Empty() {
  return ({ children }: { children: string }) => (
    <p mix={css({ color: '#94a3b8', fontStyle: 'italic', fontSize: '13px', margin: 0 })}>
      {children}
    </p>
  )
}

function DayList() {
  return ({
    groups,
    indicators,
  }: {
    groups: DayGroup[]
    indicators: Map<string, MacroIndicator>
  }) => (
    <div mix={css({ display: 'flex', flexDirection: 'column', gap: '12px' })}>
      {groups.map((g) => (
        <div
          mix={css({
            background: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '8px',
            overflow: 'hidden',
          })}
        >
          <div
            mix={css({
              background: '#f8fafc',
              padding: '6px 14px',
              fontSize: '12px',
              fontWeight: 600,
              color: '#0f172a',
              borderBottom: '1px solid #e2e8f0',
            })}
          >
            {g.date}
          </div>
          {g.events.map((e) => (
            <EventRow event={e} indicator={indicators.get(e.indicator_code)} />
          ))}
        </div>
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
          padding: '12px 16px',
          borderTop: '1px solid #f1f5f9',
          '&:first-child': { borderTop: 'none' },
        })}
      >
        <div
          mix={css({
            display: 'flex',
            alignItems: 'baseline',
            gap: '8px',
            marginBottom: '6px',
          })}
        >
          <code
            mix={css({
              fontSize: '11px',
              fontFamily: 'ui-monospace, monospace',
              color: '#1d4ed8',
              fontWeight: 600,
            })}
          >
            {event.indicator_code}
          </code>
          <EventKindPill kind={event.event_kind} />
          {event.decision && <DecisionPill decision={event.decision} bps={event.decision_bps} />}
          {indicator && (
            <span mix={css({ fontSize: '11px', color: '#94a3b8' })}>
              {indicator.unit} · {indicator.frequency}
            </span>
          )}
          <span mix={css({ marginLeft: 'auto', fontSize: '11px', color: '#94a3b8' })}>
            {event.source}
          </span>
        </div>
        <div
          mix={css({
            fontSize: '14px',
            fontWeight: 600,
            color: '#0f172a',
            marginBottom: '4px',
            lineHeight: 1.4,
          })}
        >
          {event.title}
        </div>
        {isReleased && (
          <div
            mix={css({
              display: 'flex',
              gap: '14px',
              flexWrap: 'wrap',
              fontSize: '12px',
              color: '#475569',
              fontVariantNumeric: 'tabular-nums',
              marginTop: '6px',
            })}
          >
            <Stat label="actual" value={event.new_value ?? '—'} accent="#0f172a" />
            {event.consensus_estimate && (
              <Stat label="consensus" value={event.consensus_estimate} />
            )}
            {event.previous_value && (
              <Stat label="prev" value={event.previous_value} />
            )}
            {event.surprise && <SurprisePill surprise={event.surprise} unit={indicator?.unit} />}
            {event.vote && <Stat label="vote" value={event.vote} />}
          </div>
        )}
        {event.summary_md && (
          <pre
            mix={css({
              marginTop: '8px',
              padding: '8px 10px',
              background: '#f8fafc',
              border: '1px solid #e2e8f0',
              borderRadius: '4px',
              fontSize: '12px',
              lineHeight: 1.55,
              color: '#1f2937',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              fontFamily: 'inherit',
            })}
          >
            {event.summary_md}
          </pre>
        )}
      </div>
    )
  }
}

function EventKindPill() {
  return ({ kind }: { kind: string }) => {
    let palette: Record<string, [string, string]> = {
      fomc_decision: ['#fef3c7', '#92400e'],
      ecb_decision: ['#fef3c7', '#92400e'],
      boj_decision: ['#fef3c7', '#92400e'],
      lpr_decision: ['#fef3c7', '#92400e'],
      cpi_release: ['#dbeafe', '#1e40af'],
      ppi_release: ['#dbeafe', '#1e40af'],
      nfp_release: ['#dbeafe', '#1e40af'],
      gdp_release: ['#dbeafe', '#1e40af'],
      pmi_release: ['#dbeafe', '#1e40af'],
    }
    let [bg, fg] = palette[kind] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '999px',
          background: bg,
          color: fg,
          fontSize: '10px',
          fontWeight: 600,
        })}
      >
        {kind}
      </span>
    )
  }
}

function DecisionPill() {
  return ({ decision, bps }: { decision: string; bps: number | null }) => {
    let palette: Record<string, [string, string]> = {
      hike: ['#fee2e2', '#991b1b'],
      cut: ['#dcfce7', '#166534'],
      hold: ['#e2e8f0', '#475569'],
    }
    let [bg, fg] = palette[decision] ?? ['#e2e8f0', '#475569']
    let label =
      decision === 'hike' && bps
        ? `+${bps} bps`
        : decision === 'cut' && bps
          ? `${bps} bps`
          : decision === 'hold'
            ? 'HOLD'
            : decision
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          background: bg,
          color: fg,
          fontSize: '11px',
          fontWeight: 700,
          textTransform: 'uppercase',
        })}
      >
        {label}
      </span>
    )
  }
}

function Stat() {
  return ({
    label,
    value,
    accent,
  }: {
    label: string
    value: string
    accent?: string
  }) => (
    <span>
      <span
        mix={css({
          fontSize: '10px',
          textTransform: 'uppercase',
          letterSpacing: '0.06em',
          color: '#94a3b8',
          marginRight: '4px',
        })}
      >
        {label}
      </span>
      <strong mix={css({ color: accent ?? '#475569', fontWeight: 600 })}>{value}</strong>
    </span>
  )
}

function SurprisePill() {
  return ({ surprise, unit }: { surprise: string; unit?: string }) => {
    let n = parseFloat(surprise)
    let pos = n > 0
    let color = pos ? '#166534' : n < 0 ? '#991b1b' : '#475569'
    let bg = pos ? '#dcfce7' : n < 0 ? '#fee2e2' : '#e2e8f0'
    let sign = pos ? '+' : ''
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          background: bg,
          color,
          fontSize: '11px',
          fontWeight: 700,
        })}
        title="surprise = actual − consensus"
      >
        Δ {sign}{surprise}{unit ? ` ${unit}` : ''}
      </span>
    )
  }
}
