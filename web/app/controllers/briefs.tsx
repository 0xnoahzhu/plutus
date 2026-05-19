import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type MarketBrief } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, parseCountry, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

interface DayGroup {
  date: string
  pre: MarketBrief | null
  post: MarketBrief | null
  /// Smart-money scan output for the day (insider buys, southbound flows,
  /// activist filings, etc.). Written by the agent's daily scanner job.
  scan: MarketBrief | null
}

export const briefs: BuildAction<'GET', typeof routes.briefs> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
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
      <BriefsPage days={days} country={country} locale={locale} />,
      request,
      { locale },
    )
  },
}

interface BriefsProps {
  days: DayGroup[]
  country: string
  locale: string
}

function BriefsPage() {
  return ({ days, country, locale }: BriefsProps) => (
    <Layout title="Market Briefs" country={country} locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Daily pre-market analysis and post-market summary for{' '}
        <strong>{country}</strong>. Agent writes via{' '}
        <code>POST /api/v1/market-briefs</code>.
      </p>
      {days.length === 0 ? (
        <p mix={css({ color: '#64748b' })}>
          No briefs yet for {country}. Push one with{' '}
          <code>POST /api/v1/market-briefs</code> using kind <code>pre_market</code>{' '}
          or <code>post_market</code>.
        </p>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: '20px' })}>
          {days.map((d) => (
            <DayRow date={d.date} country={country} pre={d.pre} post={d.post} scan={d.scan} />
          ))}
        </div>
      )}
    </Layout>
  )
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
          fontSize: '13px',
          fontWeight: 600,
          color: '#0f172a',
          marginBottom: '8px',
        })}
      >
        {date} · {country}
      </div>
      <div
        mix={css({
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: '12px',
          '@media (max-width: 720px)': { gridTemplateColumns: '1fr' },
        })}
      >
        <BriefCard kind="pre_market" brief={pre} />
        <BriefCard kind="post_market" brief={post} />
      </div>
      {/* Smart-money scan is its own full-width card below the pre/post pair.
          Only rendered when present so the layout doesn't grow a permanent
          dashed placeholder for users who don't run the scanner. */}
      {scan && (
        <div mix={css({ marginTop: '12px' })}>
          <BriefCard kind="smart_money_scan" brief={scan} />
        </div>
      )}
    </div>
  )
}

function BriefCard() {
  return ({ kind, brief }: { kind: string; brief: MarketBrief | null }) => {
    let label =
      kind === 'pre_market'
        ? 'Pre-market'
        : kind === 'post_market'
          ? 'Post-market'
          : kind === 'smart_money_scan'
            ? 'Smart-money scan'
            : kind
    let accent =
      kind === 'pre_market'
        ? '#1d4ed8'
        : kind === 'post_market'
          ? '#7c3aed'
          : kind === 'smart_money_scan'
            ? '#d97706' // amber — distinct from pre/post
            : '#64748b'
    if (!brief) {
      return (
        <div
          mix={css({
            background: '#fff',
            border: '1px dashed #cbd5e1',
            borderRadius: '8px',
            padding: '16px 20px',
            color: '#94a3b8',
            fontSize: '13px',
          })}
        >
          <div
            mix={css({
              fontSize: '10px',
              fontWeight: 700,
              textTransform: 'uppercase',
              letterSpacing: '0.08em',
              color: '#94a3b8',
              marginBottom: '6px',
            })}
          >
            {label}
          </div>
          <em>(no brief recorded)</em>
        </div>
      )
    }
    return (
      <div
        mix={css({
          background: '#fff',
          border: '1px solid #e2e8f0',
          borderLeft: `3px solid ${accent}`,
          borderRadius: '8px',
          padding: '16px 20px',
        })}
      >
        <div
          mix={css({
            display: 'flex',
            alignItems: 'baseline',
            gap: '8px',
            marginBottom: '8px',
          })}
        >
          <span
            mix={css({
              fontSize: '10px',
              fontWeight: 700,
              textTransform: 'uppercase',
              letterSpacing: '0.08em',
              color: accent,
            })}
          >
            {label}
          </span>
          {brief.sentiment && <SentimentChip sentiment={brief.sentiment} />}
          <span
            mix={css({
              marginLeft: 'auto',
              fontSize: '11px',
              color: '#94a3b8',
            })}
          >
            {brief.source} · {brief.language}
          </span>
        </div>
        <div
          mix={css({
            fontSize: '15px',
            fontWeight: 600,
            color: '#0f172a',
            marginBottom: '8px',
            lineHeight: 1.4,
          })}
        >
          {brief.headline}
        </div>
        {brief.content_md && (
          <pre
            mix={css({
              margin: 0,
              padding: '10px 12px',
              background: '#f8fafc',
              border: '1px solid #e2e8f0',
              borderRadius: '4px',
              fontSize: '13px',
              lineHeight: 1.6,
              color: '#1f2937',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              fontFamily: 'inherit',
            })}
          >
            {brief.content_md}
          </pre>
        )}
      </div>
    )
  }
}

function SentimentChip() {
  return ({ sentiment }: { sentiment: string }) => {
    let palette: Record<string, [string, string]> = {
      bullish: ['#dcfce7', '#166534'],
      positive: ['#dcfce7', '#166534'],
      bearish: ['#fee2e2', '#991b1b'],
      negative: ['#fee2e2', '#991b1b'],
      neutral: ['#e2e8f0', '#475569'],
    }
    let [bg, fg] = palette[sentiment] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          fontSize: '11px',
          fontWeight: 600,
          background: bg,
          color: fg,
        })}
      >
        {sentiment}
      </span>
    )
  }
}
