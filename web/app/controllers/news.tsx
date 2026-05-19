import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type NewsItem } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, parseCountry, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const news: BuildAction<'GET', typeof routes.news> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)

    let all = await api.news(locale).catch(() => [])
    // News carries a `region` field directly (US/HK/CN/global). Filter on
    // that — falls through `global` regardless of which country is picked.
    let filtered = all.filter(
      (n) => n.region === country || n.region === 'global',
    )
    // Newest first.
    filtered.sort((a, b) => b.published_at.localeCompare(a.published_at))

    return render(
      <NewsListPage
        rows={filtered}
        totalRaw={all.length}
        country={country}
        locale={locale}
      />,
      request,
      { locale },
    )
  },
}

interface NewsListProps {
  rows: NewsItem[]
  totalRaw: number
  country: string
  locale: string
}

function NewsListPage() {
  return ({ rows, totalRaw, country, locale }: NewsListProps) => (
    <Layout title="News" country={country} locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Showing {rows.length} of {totalRaw} items (region: {country} or global). Items are
        created by the agent via <code>POST /api/v1/news</code>.
      </p>
      {rows.length === 0 ? (
        <p mix={css({ color: '#64748b' })}>
          No news yet. Push items in with <code>POST /api/v1/news</code>.
        </p>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: '10px' })}>
          {rows.map((n) => (
            <a
              href={`/news/${n.id}`}
              mix={css({
                display: 'block',
                background: '#fff',
                border: '1px solid #e2e8f0',
                borderRadius: '8px',
                padding: '14px 18px',
                textDecoration: 'none',
                color: 'inherit',
                transition: 'border-color 120ms ease',
                '&:hover': { borderColor: '#1d4ed8' },
              })}
            >
              <div
                mix={css({
                  display: 'flex',
                  alignItems: 'baseline',
                  gap: '10px',
                  marginBottom: '6px',
                })}
              >
                <ImportanceDot importance={n.importance} />
                <SentimentChip sentiment={n.sentiment} />
                <CategoryPill>{n.category}</CategoryPill>
                <RegionPill>{n.region}</RegionPill>
                <span
                  mix={css({
                    marginLeft: 'auto',
                    fontSize: '11px',
                    color: '#94a3b8',
                  })}
                >
                  {fmtDate(n.published_at)} · {n.source}
                </span>
              </div>
              <div
                mix={css({
                  fontSize: '15px',
                  fontWeight: 600,
                  color: '#0f172a',
                  lineHeight: 1.4,
                })}
              >
                {n.title}
              </div>
              {n.summary && (
                <div
                  mix={css({
                    fontSize: '13px',
                    color: '#475569',
                    marginTop: '6px',
                    lineHeight: 1.55,
                    display: '-webkit-box',
                    WebkitLineClamp: 2,
                    WebkitBoxOrient: 'vertical',
                    overflow: 'hidden',
                  })}
                >
                  {n.summary}
                </div>
              )}
            </a>
          ))}
        </div>
      )}
    </Layout>
  )
}

function fmtDate(iso: string): string {
  return iso.slice(0, 16).replace('T', ' ')
}

function ImportanceDot() {
  return ({ importance }: { importance: string }) => {
    let color =
      importance === 'high' ? '#dc2626' : importance === 'low' ? '#94a3b8' : '#f59e0b'
    return (
      <span
        title={`importance: ${importance}`}
        mix={css({
          width: '8px',
          height: '8px',
          borderRadius: '999px',
          background: color,
          display: 'inline-block',
        })}
      />
    )
  }
}

function SentimentChip() {
  return ({ sentiment }: { sentiment: string | null }) => {
    if (!sentiment) return null
    let palette: Record<string, [string, string]> = {
      positive: ['#dcfce7', '#166534'],
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

function CategoryPill() {
  return ({ children }: { children: string }) => (
    <span
      mix={css({
        padding: '1px 8px',
        borderRadius: '999px',
        background: '#eef2ff',
        color: '#3730a3',
        fontSize: '11px',
        fontWeight: 600,
      })}
    >
      {children}
    </span>
  )
}

function RegionPill() {
  return ({ children }: { children: string }) => (
    <span
      mix={css({
        padding: '1px 8px',
        borderRadius: '999px',
        background: '#f1f5f9',
        color: '#475569',
        fontSize: '11px',
        fontWeight: 600,
      })}
    >
      {children}
    </span>
  )
}
