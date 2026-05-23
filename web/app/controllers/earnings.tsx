import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import { api, type EarningsEvent, type Stock } from '../api.ts'
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
  parseCountry,
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  StockBadge,
  type Theme,
  UnreadDot,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

interface DayGroup {
  date: string
  events: Array<{ event: EarningsEvent; stock: Stock | undefined }>
}

export const earnings: BuildAction<'GET', typeof routes.earnings> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)

    let events = await api.earnings(country).catch(() => [])
    let stocks = await api
      .stocksByIds(events.map((e) => e.stock_id), locale)
      .catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    let today = new Date().toISOString().slice(0, 10)
    let upcoming = groupByDate(
      events.filter((e) => e.announce_date >= today),
      stockMap,
    )
    let past = groupByDate(
      events.filter((e) => e.announce_date < today),
      stockMap,
    ).reverse()

    return render(
      <EarningsPage
        upcoming={upcoming}
        past={past}
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

function groupByDate(
  events: EarningsEvent[],
  stocks: Map<number, Stock>,
): DayGroup[] {
  let by = new Map<string, DayGroup>()
  for (let e of events) {
    let g = by.get(e.announce_date)
    if (!g) {
      g = { date: e.announce_date, events: [] }
      by.set(e.announce_date, g)
    }
    g.events.push({ event: e, stock: stocks.get(e.stock_id) })
  }
  for (let g of by.values()) {
    g.events.sort((a, b) =>
      (a.stock?.symbol ?? '').localeCompare(b.stock?.symbol ?? ''),
    )
  }
  return Array.from(by.values()).sort((a, b) => a.date.localeCompare(b.date))
}

interface EarningsProps {
  upcoming: DayGroup[]
  past: DayGroup[]
  country: string
  locale: string
  theme: Theme
  today: string
}

function EarningsPage() {
  return ({ upcoming, past, country, locale, theme, today }: EarningsProps) => {
    let p = messages(locale).pages.earnings
    return (
    <Layout
      title={p.title}
      subtitle={p.subtitle(country)}
      country={country}
      locale={locale}
      theme={theme}
    >
      <MarkAllReadStrip kind="earnings_event" />
      <SectionTitle hint={messages(locale).pages.macroEvents.hintFrom(today)}>
        {p.sectionUpcoming}
      </SectionTitle>
      {upcoming.length === 0 ? (
        <Card>
          <EmptyState title={p.noUpcomingTitle} />
        </Card>
      ) : (
        <DayList groups={upcoming} />
      )}

      <div mix={css({ marginTop: space[6] })}>
        <SectionTitle hint={messages(locale).pages.macroEvents.hintBefore(today)}>
          {p.sectionPast}
        </SectionTitle>
      </div>
      {past.length === 0 ? (
        <Card>
          <EmptyState title={p.noPastTitle} />
        </Card>
      ) : (
        <DayList groups={past} />
      )}
    </Layout>
    )
  }
}

function DayList() {
  return ({ groups }: { groups: DayGroup[] }) => (
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
          <table
            mix={css({
              width: '100%',
              borderCollapse: 'collapse',
              fontSize: font.base,
            })}
          >
            <tbody>
              {g.events.map(({ event, stock }) => (
                <EarningsRow event={event} stock={stock} />
              ))}
            </tbody>
          </table>
        </Card>
      ))}
    </div>
  )
}

function EarningsRow() {
  return ({ event, stock }: { event: EarningsEvent; stock: Stock | undefined }) => {
    let beat: 'beat' | 'miss' | 'inline' | null = null
    if (event.eps_actual && event.eps_estimate) {
      let a = parseFloat(event.eps_actual)
      let e = parseFloat(event.eps_estimate)
      if (Number.isFinite(a) && Number.isFinite(e)) {
        if (a > e * 1.005) beat = 'beat'
        else if (a < e * 0.995) beat = 'miss'
        else beat = 'inline'
      }
    }
    return (
      <tr
        mix={css({
          borderTop: `1px solid ${color.borderSoft}`,
          '&:first-child': { borderTop: 'none' },
        })}
      >
        <td mix={css({ padding: `${space[3]} ${space[4]}`, width: '24%' })}>
          <div
            mix={css({
              display: 'inline-flex',
              alignItems: 'center',
              gap: space[2],
            })}
          >
            <UnreadDot readAt={event.read_at} />
            {stock ? (
              <a
                href={`/stocks/${stock.id}`}
                mix={css({
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: space[2],
                  textDecoration: 'none',
                  color: color.text,
                  '&:hover': { color: color.brandHover },
                })}
              >
                <StockBadge symbol={stock.symbol} size={22} />
                <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>{stock.symbol}</span>
              </a>
            ) : (
              <span mix={css({ color: color.textDim })}>#{event.stock_id}</span>
            )}
          </div>
        </td>
        <td mix={css({ padding: `${space[3]} ${space[4]}`, width: '12%' })}>
          <a
            href={`/earnings/${event.id}`}
            mix={css({
              fontSize: font.sm,
              color: color.textMuted,
              textDecoration: 'none',
              '&:hover': { color: color.brandHover },
            })}
          >
            {event.fiscal_period} {event.fiscal_year}
          </a>
        </td>
        <td mix={css({ padding: `${space[3]} ${space[4]}`, width: '12%' })}>
          <TimingPill timing={event.announce_timing} />
        </td>
        <td mix={css({ padding: `${space[3]} ${space[4]}`, width: '14%' })}>
          <Badge tone={statusTone(event.status)}>{event.status}</Badge>
        </td>
        <td
          mix={css({
            padding: `${space[3]} ${space[4]}`,
            fontSize: font.sm,
            fontVariantNumeric: 'tabular-nums',
            color: color.text,
          })}
        >
          {event.eps_estimate && (
            <span mix={css({ color: color.textMuted })}>
              est {event.eps_estimate}
              {event.currency ? ` ${event.currency}` : ''}
            </span>
          )}
          {event.eps_actual && (
            <span mix={css({ marginLeft: space[2], fontWeight: 600 })}>
              actual {event.eps_actual}
            </span>
          )}
          {beat && (
            <span mix={css({ marginLeft: space[2] })}>
              <Badge
                tone={beat === 'beat' ? 'success' : beat === 'miss' ? 'danger' : 'neutral'}
              >
                {beat}
              </Badge>
            </span>
          )}
        </td>
        <td
          mix={css({
            padding: `${space[3]} ${space[4]}`,
            fontSize: font.sm,
            color: color.textMuted,
          })}
        >
          {event.notes && (
            <span title={event.notes} mix={css({ fontStyle: 'italic' })}>
              {event.notes.length > 40 ? event.notes.slice(0, 40) + '…' : event.notes}
            </span>
          )}
        </td>
      </tr>
    )
  }
}

function TimingPill() {
  return ({ timing }: { timing: string }) => {
    let toneMap: Record<string, BadgeTone> = {
      bmo: 'info',
      amc: 'brand',
      during: 'neutral',
      unknown: 'neutral',
    }
    let labelMap: Record<string, string> = {
      bmo: 'BMO',
      amc: 'AMC',
      during: 'mid-day',
      unknown: '—',
    }
    return (
      <Badge tone={toneMap[timing] ?? 'neutral'}>
        {labelMap[timing] ?? timing}
      </Badge>
    )
  }
}

function statusTone(s: string): BadgeTone {
  if (s === 'released') return 'success'
  if (s === 'confirmed') return 'info'
  if (s === 'postponed') return 'danger'
  if (s === 'scheduled') return 'warn'
  return 'neutral'
}
