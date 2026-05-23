import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Catalyst, type Stock } from '../api.ts'
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
  StockBadge,
  type Theme,
  UnreadDot,
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

    let all = await api.catalysts({ country, locale }).catch(() => [])
    // Catalysts can be sector-wide (stock_id=null) or tied to a stock.
    // Only fetch the stocks we actually need to render.
    let stockIds = all
      .map((c) => c.stock_id)
      .filter((id): id is number => id != null)
    let stocks = await api.stocksByIds(stockIds, locale).catch(() => [] as Stock[])
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
  return ({ upcoming, past, country, locale, theme, today }: CatalystsProps) => {
    let p = messages(locale).pages.catalysts
    return (
    <Layout
      title={p.title}
      subtitle={p.subtitle(country)}
      country={country}
      locale={locale}
      theme={theme}
    >
      <SectionTitle hint={messages(locale).pages.macroEvents.hintFrom(today)}>
        {p.sectionUpcoming}
      </SectionTitle>
      {upcoming.length === 0 ? (
        <Card>
          <EmptyState
            title={p.noUpcomingTitle(country)}
            hint={<code>POST /api/v1/catalysts</code>}
          />
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
    <a
      href={`/catalysts/${catalyst.id}`}
      mix={css({
        display: 'block',
        padding: `${space[3]} ${space[4]}`,
        borderTop: `1px solid ${color.borderSoft}`,
        '&:first-child': { borderTop: 'none' },
        textDecoration: 'none',
        color: 'inherit',
        transition: 'background 120ms ease',
        '&:hover': { background: color.hover },
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
        <UnreadDot readAt={catalyst.read_at} />
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
          lineHeight: 1.4,
        })}
      >
        {catalyst.title ?? '(untitled)'}
      </div>
    </a>
  )
}

function Target() {
  return ({ catalyst, stock }: { catalyst: Catalyst; stock: Stock | undefined }) => {
    // Renders as plain content (no nested <a>) — the parent row is the
    // clickable surface to /catalysts/:id.
    if (stock) {
      return (
        <span
          mix={css({
            display: 'inline-flex',
            alignItems: 'center',
            gap: space[2],
            color: color.text,
          })}
        >
          <StockBadge symbol={stock.symbol} size={22} />
          <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>{stock.symbol}</span>
        </span>
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

