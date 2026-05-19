import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type EarningsEvent, type Stock } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, parseCountry, resolveLocale } from '../ui/layout.tsx'
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

    let [events, stocks] = await Promise.all([
      api.earnings(country).catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    // Server-side "today" boundary for upcoming/past split. Doesn't have to
    // be perfect — the UI re-renders on next visit.
    let today = new Date().toISOString().slice(0, 10)

    let upcoming: DayGroup[] = groupByDate(
      events.filter((e) => e.announce_date >= today),
      stockMap,
    )
    let past: DayGroup[] = groupByDate(
      events.filter((e) => e.announce_date < today),
      stockMap,
    ).reverse() // most-recent past first

    return render(
      <EarningsPage
        upcoming={upcoming}
        past={past}
        country={country}
        locale={locale}
        today={today}
      />,
      request,
      { locale },
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
  // sort within each day by stock symbol
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
  today: string
}

function EarningsPage() {
  return ({ upcoming, past, country, locale, today }: EarningsProps) => (
    <Layout title="Earnings calendar" country={country} locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Earnings events for <strong>{country}</strong>. Agent writes via{' '}
        <code>POST /api/v1/earnings</code>. Upsert by (stock, fiscal_year,
        fiscal_period) — re-POST the same key as the event progresses from
        scheduled → confirmed → released.
      </p>

      <SectionHeader label="Upcoming" sub={`from ${today}`} />
      {upcoming.length === 0 ? (
        <Empty>No upcoming earnings on file.</Empty>
      ) : (
        <DayList groups={upcoming} />
      )}

      <div mix={css({ marginTop: '24px' })}>
        <SectionHeader label="Past" sub={`before ${today}`} />
      </div>
      {past.length === 0 ? (
        <Empty>No past earnings recorded.</Empty>
      ) : (
        <DayList groups={past} />
      )}
    </Layout>
  )
}

function SectionHeader() {
  return ({ label, sub }: { label: string; sub: string }) => (
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
  )
}

function Empty() {
  return ({ children }: { children: string }) => (
    <p
      mix={css({
        color: '#94a3b8',
        fontStyle: 'italic',
        fontSize: '13px',
        margin: '0 0 12px',
      })}
    >
      {children}
    </p>
  )
}

function DayList() {
  return ({ groups }: { groups: DayGroup[] }) => (
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
          <table
            mix={css({
              width: '100%',
              borderCollapse: 'collapse',
              fontSize: '13px',
            })}
          >
            <tbody>
              {g.events.map(({ event, stock }) => (
                <EarningsRow event={event} stock={stock} />
              ))}
            </tbody>
          </table>
        </div>
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
      <tr mix={css({ borderTop: '1px solid #f1f5f9' })}>
        <td mix={css({ padding: '10px 14px', width: '20%' })}>
          {stock ? (
            <a
              href={`/stocks/${stock.id}`}
              mix={css({
                fontFamily: 'ui-monospace, monospace',
                fontWeight: 600,
                color: '#1d4ed8',
                textDecoration: 'none',
                '&:hover': { textDecoration: 'underline' },
              })}
            >
              {stock.symbol}
            </a>
          ) : (
            <span mix={css({ color: '#94a3b8' })}>#{event.stock_id}</span>
          )}
          {stock && (
            <span mix={css({ marginLeft: '6px', fontSize: '10px', color: '#94a3b8' })}>
              {stock.market_code}
            </span>
          )}
        </td>
        <td mix={css({ padding: '10px 14px', width: '12%' })}>
          <span mix={css({ fontSize: '12px', color: '#475569' })}>
            {event.fiscal_period} {event.fiscal_year}
          </span>
        </td>
        <td mix={css({ padding: '10px 14px', width: '12%' })}>
          <TimingPill timing={event.announce_timing} />
        </td>
        <td mix={css({ padding: '10px 14px', width: '12%' })}>
          <StatusPill status={event.status} />
        </td>
        <td
          mix={css({
            padding: '10px 14px',
            fontSize: '12px',
            fontVariantNumeric: 'tabular-nums',
          })}
        >
          {event.eps_estimate && (
            <span mix={css({ color: '#64748b' })}>
              est {event.eps_estimate}
              {event.currency ? ` ${event.currency}` : ''}
            </span>
          )}
          {event.eps_actual && (
            <span mix={css({ marginLeft: '8px', color: '#0f172a', fontWeight: 600 })}>
              actual {event.eps_actual}
            </span>
          )}
          {beat && <BeatPill kind={beat} />}
        </td>
        <td mix={css({ padding: '10px 14px', fontSize: '12px', color: '#64748b' })}>
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
    let label: Record<string, string> = {
      bmo: 'BMO',
      amc: 'AMC',
      during: 'mid-day',
      unknown: '—',
    }
    let color = timing === 'bmo' ? '#0891b2' : timing === 'amc' ? '#7c3aed' : '#94a3b8'
    return (
      <span
        title={
          timing === 'bmo'
            ? 'Before market open'
            : timing === 'amc'
              ? 'After market close'
              : timing
        }
        mix={css({
          fontSize: '10px',
          fontWeight: 600,
          padding: '1px 6px',
          borderRadius: '4px',
          background: 'transparent',
          color,
          border: `1px solid ${color}`,
        })}
      >
        {label[timing] ?? timing}
      </span>
    )
  }
}

function StatusPill() {
  return ({ status }: { status: string }) => {
    let palette: Record<string, [string, string]> = {
      scheduled: ['#fef3c7', '#92400e'],
      confirmed: ['#dbeafe', '#1e40af'],
      released: ['#dcfce7', '#166534'],
      postponed: ['#fee2e2', '#991b1b'],
    }
    let [bg, fg] = palette[status] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '999px',
          background: bg,
          color: fg,
          fontSize: '11px',
          fontWeight: 600,
        })}
      >
        {status}
      </span>
    )
  }
}

function BeatPill() {
  return ({ kind }: { kind: 'beat' | 'miss' | 'inline' }) => {
    let palette: Record<string, [string, string]> = {
      beat: ['#dcfce7', '#166534'],
      miss: ['#fee2e2', '#991b1b'],
      inline: ['#e2e8f0', '#475569'],
    }
    let [bg, fg] = palette[kind]
    return (
      <span
        mix={css({
          marginLeft: '8px',
          padding: '1px 6px',
          borderRadius: '4px',
          background: bg,
          color: fg,
          fontSize: '10px',
          fontWeight: 700,
          textTransform: 'uppercase',
        })}
      >
        {kind}
      </span>
    )
  }
}
