import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Catalyst, type Stock } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, parseCountry, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

interface DayGroup {
  date: string
  rows: Array<{ catalyst: Catalyst; stock: Stock | undefined }>
}

export const catalysts: BuildAction<'GET', typeof routes.catalysts> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)

    let [all, stocks] = await Promise.all([
      api.catalysts({ country, locale }).catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    let today = new Date().toISOString().slice(0, 10)
    let upcoming = group(
      all.filter((c) => c.catalyst_date >= today),
      stockMap,
    )
    let past = group(
      all.filter((c) => c.catalyst_date < today),
      stockMap,
    ).reverse()

    return render(
      <CatalystsPage
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

function group(items: Catalyst[], stocks: Map<number, Stock>): DayGroup[] {
  let by = new Map<string, DayGroup>()
  for (let c of items) {
    let g = by.get(c.catalyst_date)
    if (!g) {
      g = { date: c.catalyst_date, rows: [] }
      by.set(c.catalyst_date, g)
    }
    g.rows.push({ catalyst: c, stock: c.stock_id != null ? stocks.get(c.stock_id) : undefined })
  }
  for (let g of by.values()) {
    g.rows.sort((a, b) => impactOrder(b.catalyst.impact_level) - impactOrder(a.catalyst.impact_level))
  }
  return Array.from(by.values()).sort((a, b) => a.date.localeCompare(b.date))
}

function impactOrder(level: string): number {
  return level === 'high' ? 3 : level === 'medium' ? 2 : level === 'low' ? 1 : 0
}

interface CatalystsProps {
  upcoming: DayGroup[]
  past: DayGroup[]
  country: string
  locale: string
  today: string
}

function CatalystsPage() {
  return ({ upcoming, past, country, locale, today }: CatalystsProps) => (
    <Layout title="Catalysts" country={country} locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Forward-looking catalysts for <strong>{country}</strong> — investor days,
        FDA decisions, tariff deadlines, policy meetings. Agent writes via{' '}
        <code>POST /api/v1/catalysts</code>. Each catalyst can target a stock, a
        sector, or a whole country.
      </p>

      <Section label="Upcoming" sub={`from ${today}`}>
        {upcoming.length === 0 ? (
          <Empty>No upcoming catalysts on file for {country}.</Empty>
        ) : (
          <DayList groups={upcoming} />
        )}
      </Section>

      <Section label="Past" sub={`before ${today}`}>
        {past.length === 0 ? (
          <Empty>No past catalysts recorded.</Empty>
        ) : (
          <DayList groups={past} />
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
  return ({ children }: { children: import('remix/ui').RemixNode }) => (
    <p mix={css({ color: '#94a3b8', fontStyle: 'italic', fontSize: '13px', margin: 0 })}>
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
          {g.rows.map(({ catalyst, stock }) => (
            <CatalystRow catalyst={catalyst} stock={stock} />
          ))}
        </div>
      ))}
    </div>
  )
}

function CatalystRow() {
  return ({ catalyst, stock }: { catalyst: Catalyst; stock: Stock | undefined }) => (
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
          flexWrap: 'wrap',
        })}
      >
        <TargetChip catalyst={catalyst} stock={stock} />
        <KindPill kind={catalyst.catalyst_kind} />
        <ImpactPill level={catalyst.impact_level} />
        <ConfidencePill confidence={catalyst.date_confidence} />
        <StatusPill status={catalyst.status} />
        <span mix={css({ marginLeft: 'auto', fontSize: '11px', color: '#94a3b8' })}>
          {catalyst.source}
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
        {catalyst.title}
      </div>
      {catalyst.summary_md && (
        <pre
          mix={css({
            marginTop: '6px',
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
          {catalyst.summary_md}
        </pre>
      )}
      {(catalyst.bull_case_md || catalyst.bear_case_md) && (
        <div
          mix={css({
            marginTop: '8px',
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: '8px',
            '@media (max-width: 720px)': { gridTemplateColumns: '1fr' },
          })}
        >
          {catalyst.bull_case_md && <CasePane kind="bull" body={catalyst.bull_case_md} />}
          {catalyst.bear_case_md && <CasePane kind="bear" body={catalyst.bear_case_md} />}
        </div>
      )}
    </div>
  )
}

function TargetChip() {
  return ({ catalyst, stock }: { catalyst: Catalyst; stock: Stock | undefined }) => {
    if (stock) {
      return (
        <a
          href={`/stocks/${stock.id}`}
          mix={css({
            fontFamily: 'ui-monospace, monospace',
            fontWeight: 600,
            color: '#1d4ed8',
            textDecoration: 'none',
            fontSize: '13px',
            '&:hover': { textDecoration: 'underline' },
          })}
        >
          {stock.symbol}
        </a>
      )
    }
    if (catalyst.sector_code) {
      return (
        <span
          mix={css({
            fontFamily: 'ui-monospace, monospace',
            fontSize: '11px',
            color: '#7c3aed',
            fontWeight: 600,
          })}
        >
          sector:{catalyst.sector_code}
        </span>
      )
    }
    if (catalyst.country) {
      return (
        <span
          mix={css({
            fontFamily: 'ui-monospace, monospace',
            fontSize: '11px',
            color: '#0891b2',
            fontWeight: 600,
          })}
        >
          country:{catalyst.country}
        </span>
      )
    }
    return <span mix={css({ fontSize: '11px', color: '#94a3b8' })}>(unspecified)</span>
  }
}

function KindPill() {
  return ({ kind }: { kind: string }) => (
    <span
      mix={css({
        padding: '1px 8px',
        borderRadius: '999px',
        background: '#e0e7ff',
        color: '#3730a3',
        fontSize: '10px',
        fontWeight: 600,
      })}
    >
      {kind}
    </span>
  )
}

function ImpactPill() {
  return ({ level }: { level: string }) => {
    let palette: Record<string, [string, string]> = {
      high: ['#fee2e2', '#991b1b'],
      medium: ['#fef3c7', '#92400e'],
      low: ['#e2e8f0', '#475569'],
    }
    let [bg, fg] = palette[level] ?? ['#e2e8f0', '#475569']
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
        })}
      >
        {level}
      </span>
    )
  }
}

function ConfidencePill() {
  return ({ confidence }: { confidence: string }) => {
    let palette: Record<string, [string, string]> = {
      scheduled: ['#dbeafe', '#1e40af'],
      expected: ['#fef3c7', '#92400e'],
      speculative: ['#e2e8f0', '#475569'],
    }
    let [bg, fg] = palette[confidence] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          background: bg,
          color: fg,
          fontSize: '10px',
          fontWeight: 600,
        })}
      >
        {confidence}
      </span>
    )
  }
}

function StatusPill() {
  return ({ status }: { status: string }) => {
    let palette: Record<string, [string, string]> = {
      upcoming: ['#dbeafe', '#1e40af'],
      released: ['#dcfce7', '#166534'],
      delayed: ['#fef3c7', '#92400e'],
      cancelled: ['#fee2e2', '#991b1b'],
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

function CasePane() {
  return ({ kind, body }: { kind: 'bull' | 'bear'; body: string }) => {
    let accent = kind === 'bull' ? '#166534' : '#991b1b'
    let bg = kind === 'bull' ? '#f0fdf4' : '#fef2f2'
    return (
      <div
        mix={css({
          padding: '8px 10px',
          background: bg,
          border: `1px solid ${accent}33`,
          borderRadius: '4px',
        })}
      >
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
          {kind} case
        </div>
        <pre
          mix={css({
            margin: 0,
            fontSize: '12px',
            lineHeight: 1.55,
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
}
