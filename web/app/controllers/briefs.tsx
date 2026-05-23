import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type MarketBrief } from '../api.ts'
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
  space,
  type Theme,
  UnreadDot,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

interface DayGroup {
  date: string
  pre: MarketBrief | null
  post: MarketBrief | null
  /// Smart-money scan output for the day.
  scan: MarketBrief | null
}

export const briefs: BuildAction<'GET', typeof routes.briefs> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let all = await api.marketBriefs(country, locale).catch(() => [])

    let byDate = new Map<string, DayGroup>()
    for (let b of all) {
      let g = byDate.get(b.trade_date)
      if (!g) {
        g = { date: b.trade_date, pre: null, post: null, scan: null }
        byDate.set(b.trade_date, g)
      }
      if (b.kind === 'pre_market') g.pre = b
      else if (b.kind === 'post_market') g.post = b
      else if (b.kind === 'smart_money_scan') g.scan = b
    }
    let days: DayGroup[] = Array.from(byDate.values()).sort((a, b) =>
      b.date.localeCompare(a.date),
    )
    return render(
      <BriefsPage days={days} country={country} locale={locale} theme={theme} />,
      request,
      { locale, theme },
    )
  },
}

interface BriefsProps {
  days: DayGroup[]
  country: string
  locale: string
  theme: Theme
}

function BriefsPage() {
  return ({ days, country, locale, theme }: BriefsProps) => {
    let p = messages(locale).pages.briefs
    return (
    <Layout
      title={p.title}
      subtitle={p.subtitle(country)}
      country={country}
      locale={locale}
      theme={theme}
    >
      {days.length === 0 ? (
        <Card>
          <EmptyState title={p.emptyTitle} hint={p.emptyHint} />
        </Card>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[5] })}>
          {days.map((d) => (
            <DayRow date={d.date} country={country} pre={d.pre} post={d.post} scan={d.scan} />
          ))}
        </div>
      )}
    </Layout>
    )
  }
}

function DayRow() {
  return ({
    date,
    country,
    pre,
    post,
    scan,
  }: {
    date: string
    country: string
    pre: MarketBrief | null
    post: MarketBrief | null
    scan: MarketBrief | null
  }) => (
    <div>
      <div
        mix={css({
          fontSize: font.sm,
          fontWeight: 600,
          color: color.text,
          marginBottom: space[2],
        })}
      >
        {date} <span mix={css({ color: color.textDim })}>· {country}</span>
      </div>
      <div
        mix={css({
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: space[3],
          '@media (max-width: 720px)': { gridTemplateColumns: '1fr' },
        })}
      >
        <BriefCard kind="pre_market" brief={pre} />
        <BriefCard kind="post_market" brief={post} />
      </div>
      {scan && (
        <div mix={css({ marginTop: space[3] })}>
          <BriefCard kind="smart_money_scan" brief={scan} />
        </div>
      )}
    </div>
  )
}

const KIND_META: Record<string, { label: string; accent: string }> = {
  pre_market: { label: 'Pre-market', accent: color.info },
  post_market: { label: 'Post-market', accent: '#7c3aed' },
  smart_money_scan: { label: 'Smart-money scan', accent: color.warn },
}

function BriefCard() {
  return ({ kind, brief }: { kind: string; brief: MarketBrief | null }) => {
    let meta = KIND_META[kind] ?? { label: kind, accent: color.textMuted }
    if (!brief) {
      return (
        <div
          mix={css({
            background: color.surface,
            border: `1px dashed ${color.border}`,
            borderRadius: radius.lg,
            padding: `${space[4]} ${space[5]}`,
            color: color.textDim,
            fontSize: font.sm,
          })}
        >
          <div
            mix={css({
              fontSize: font.xs,
              fontWeight: 700,
              textTransform: 'uppercase',
              letterSpacing: '0.08em',
              color: color.textDim,
              marginBottom: space[1],
            })}
          >
            {meta.label}
          </div>
          <em>(no brief recorded)</em>
        </div>
      )
    }
    return (
      <a
        href={`/briefs/${brief.id}`}
        mix={css({
          display: 'block',
          background: color.surface,
          border: `1px solid ${color.border}`,
          borderLeft: `3px solid ${meta.accent}`,
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
          <UnreadDot readAt={brief.read_at} />
          <span
            mix={css({
              fontSize: font.xs,
              fontWeight: 700,
              textTransform: 'uppercase',
              letterSpacing: '0.08em',
              color: meta.accent,
            })}
          >
            {meta.label}
          </span>
          {brief.sentiment && (
            <Badge tone={sentimentTone(brief.sentiment)}>{brief.sentiment}</Badge>
          )}
          <span
            mix={css({
              marginLeft: 'auto',
              fontSize: font.xs,
              color: color.textDim,
            })}
          >
            {brief.source}
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
          {brief.headline ?? '(untitled)'}
        </div>
      </a>
    )
  }
}

function sentimentTone(s: string): BadgeTone {
  if (s === 'positive' || s === 'bullish') return 'success'
  if (s === 'negative' || s === 'bearish') return 'danger'
  return 'neutral'
}
