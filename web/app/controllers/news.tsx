import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type NewsItem } from '../api.ts'
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
} from '../ui/layout.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

export const news: BuildAction<'GET', typeof routes.news> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)

    let all = await api.news(locale).catch(() => [])
    let filtered = all.filter((n) => n.region === country || n.region === 'global')
    filtered.sort((a, b) => b.published_at.localeCompare(a.published_at))

    return render(
      <NewsListPage
        rows={filtered}
        totalRaw={all.length}
        country={country}
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}

interface NewsListProps {
  rows: NewsItem[]
  totalRaw: number
  country: string
  locale: string
  theme: Theme
}

function NewsListPage() {
  return ({ rows, totalRaw, country, locale, theme }: NewsListProps) => {
    let p = messages(locale).pages.news
    return (
    <Layout
      title={p.title}
      subtitle={p.subtitle(rows.length, totalRaw, country)}
      country={country}
      locale={locale}
      theme={theme}
    >
      {rows.length === 0 ? (
        <Card>
          <EmptyState title={p.emptyTitle} hint={p.emptyHint} />
        </Card>
      ) : (
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[2] })}>
          {rows.map((n) => (
            <NewsCard item={n} />
          ))}
        </div>
      )}
    </Layout>
    )
  }
}

function NewsCard() {
  return ({ item: n }: { item: NewsItem }) => (
    <a
      href={`/news/${n.id}`}
      mix={css({
        display: 'block',
        background: color.surface,
        border: `1px solid ${color.border}`,
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
        <ImportanceDot importance={n.importance} />
        {n.sentiment && <Badge tone={sentimentTone(n.sentiment)}>{n.sentiment}</Badge>}
        <Badge tone="brand">{n.category}</Badge>
        <Badge tone="neutral">{n.region}</Badge>
        <span
          mix={css({
            marginLeft: 'auto',
            fontSize: font.xs,
            color: color.textDim,
          })}
        >
          <LocalTime value={n.published_at} format="datetime" /> · {n.source}
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
        {n.title ?? '(untitled)'}
      </div>
      {n.summary && (
        <div
          mix={css({
            fontSize: font.sm,
            color: color.textMuted,
            marginTop: space[2],
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
  )
}

function sentimentTone(s: string): BadgeTone {
  if (s === 'positive' || s === 'bullish') return 'success'
  if (s === 'negative' || s === 'bearish') return 'danger'
  return 'neutral'
}

function ImportanceDot() {
  return ({ importance }: { importance: string }) => {
    let dot =
      importance === 'high'
        ? color.danger
        : importance === 'low'
          ? color.textDim
          : color.warn
    return (
      <span
        title={`importance: ${importance}`}
        mix={css({
          width: '8px',
          height: '8px',
          borderRadius: '999px',
          background: dot,
          display: 'inline-block',
        })}
      />
    )
  }
}

