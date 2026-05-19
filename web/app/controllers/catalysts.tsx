import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Catalyst, type Stock } from '../api.ts'
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
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
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
    let theme = resolveTheme(request, url.searchParams)

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
        theme={theme}
        today={today}
      />,
      request,
      { locale, theme },
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
  theme: Theme
  today: string
}

function CatalystsPage() {
  return ({ upcoming, past, country, locale, theme, today }: CatalystsProps) => (
    <Layout
      title="Catalysts"
      subtitle={`Forward-looking catalysts for ${country}`}
      country={country}
      locale={locale}
      theme={theme}
    >
      <SectionTitle hint={`from ${today}`}>Upcoming</SectionTitle>
      {upcoming.length === 0 ? (
        <Card>
          <EmptyState
            title={`No upcoming catalysts on file for ${country}`}
            hint={
              <>
                Agent writes via <code>POST /api/v1/catalysts</code>. Each
                catalyst can target a stock, a sector, or a whole country.
              </>
            }
          />
        </Card>
      ) : (
        <DayList groups={upcoming} />
      )}

      <div mix={css({ marginTop: space[6] })}>
        <SectionTitle hint={`before ${today}`}>Past</SectionTitle>
      </div>
      {past.length === 0 ? (
        <Card>
          <EmptyState title="No past catalysts recorded" />
        </Card>
      ) : (
        <DayList groups={past} />
      )}
    </Layout>
  )
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
          <div>
            {g.rows.map(({ catalyst, stock }) => (
              <CatalystRow catalyst={catalyst} stock={stock} />
            ))}
          </div>
        </Card>
      ))}
    </div>
  )
}

function CatalystRow() {
  return ({ catalyst, stock }: { catalyst: Catalyst; stock: Stock | undefined }) => (
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
        <Target catalyst={catalyst} stock={stock} />
        <Badge tone="brand">{catalyst.catalyst_kind}</Badge>
        <Badge tone={impactTone(catalyst.impact_level)}>{catalyst.impact_level}</Badge>
        <Badge tone={confidenceTone(catalyst.date_confidence)}>
          {catalyst.date_confidence}
        </Badge>
        <Badge tone={statusTone(catalyst.status)}>{catalyst.status}</Badge>
        <span
          mix={css({
            marginLeft: 'auto',
            fontSize: font.xs,
            color: color.textDim,
          })}
        >
          {catalyst.source}
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
        {catalyst.title}
      </div>
      {catalyst.summary_md && (
        <pre
          mix={css({
            margin: `${space[2]} 0 0`,
            padding: `${space[2]} ${space[3]}`,
            background: color.bg,
            border: `1px solid ${color.borderSoft}`,
            borderRadius: radius.md,
            fontSize: font.sm,
            lineHeight: 1.6,
            color: color.text,
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
            marginTop: space[2],
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: space[2],
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

function Target() {
  return ({ catalyst, stock }: { catalyst: Catalyst; stock: Stock | undefined }) => {
    if (stock) {
      return (
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
      )
    }
    if (catalyst.sector_code) {
      return <Badge tone="brand">sector:{catalyst.sector_code}</Badge>
    }
    if (catalyst.country) {
      return <Badge tone="info">country:{catalyst.country}</Badge>
    }
    return <Badge tone="neutral">(unspecified)</Badge>
  }
}

function impactTone(level: string): BadgeTone {
  if (level === 'high') return 'danger'
  if (level === 'medium') return 'warn'
  return 'neutral'
}

function confidenceTone(confidence: string): BadgeTone {
  if (confidence === 'scheduled' || confidence === 'confirmed') return 'info'
  if (confidence === 'expected') return 'warn'
  return 'neutral'
}

function statusTone(status: string): BadgeTone {
  if (status === 'upcoming' || status === 'scheduled') return 'info'
  if (status === 'released' || status === 'happened_positive') return 'success'
  if (status === 'happened_negative') return 'danger'
  if (status === 'delayed') return 'warn'
  return 'neutral'
}

function CasePane() {
  return ({ kind, body }: { kind: 'bull' | 'bear'; body: string }) => {
    let accent = kind === 'bull' ? color.success : color.danger
    let bg = kind === 'bull' ? color.successSoft : color.dangerSoft
    let fg = kind === 'bull' ? color.successText : color.dangerText
    return (
      <div
        mix={css({
          padding: `${space[2]} ${space[3]}`,
          background: bg,
          border: `1px solid ${accent}33`,
          borderRadius: radius.md,
        })}
      >
        <div
          mix={css({
            fontSize: font.xs,
            fontWeight: 700,
            textTransform: 'uppercase',
            letterSpacing: '0.08em',
            color: fg,
            marginBottom: space[1],
          })}
        >
          {kind} case
        </div>
        <pre
          mix={css({
            margin: 0,
            fontSize: font.sm,
            lineHeight: 1.6,
            color: color.text,
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
