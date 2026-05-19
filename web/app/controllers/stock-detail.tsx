import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import {
  api,
  type NewsItem,
  type NewsStockLink,
  type Stock,
  type StockTranslation,
} from '../api.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  Card,
  color,
  EmptyState,
  font,
  Layout,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const stockDetail: BuildAction<'GET', typeof routes.stockDetail> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) {
      return new Response('Bad stock id', { status: 400 })
    }
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)

    let [stock, translations, newsLinks, allNews] = await Promise.all([
      api.stock(id).catch(() => null),
      api.stockTranslations(id).catch(() => [] as StockTranslation[]),
      api.newsForStock(id).catch(() => [] as NewsStockLink[]),
      api.news(locale).catch(() => [] as NewsItem[]),
    ])
    if (!stock) {
      return new Response('Stock not found', { status: 404 })
    }
    let newsById = new Map<number, NewsItem>(allNews.map((n) => [n.id, n]))
    let recentNews: Array<{ link: NewsStockLink; item: NewsItem }> = newsLinks
      .map((l) => ({ link: l, item: newsById.get(l.news_id)! }))
      .filter((p) => p.item)
    recentNews.sort((a, b) => b.item.published_at.localeCompare(a.item.published_at))
    let recentTrimmed = recentNews.slice(0, 10)

    return render(
      <StockDetailPage
        stock={stock}
        translations={translations}
        locale={locale}
        theme={theme}
        recentNews={recentTrimmed}
        totalNews={recentNews.length}
      />,
      request,
      { locale, theme },
    )
  },
}

interface StockDetailProps {
  stock: Stock
  translations: StockTranslation[]
  locale: string
  theme: Theme
  recentNews: Array<{ link: NewsStockLink; item: NewsItem }>
  totalNews: number
}

function StockDetailPage() {
  return ({ stock, translations, locale, theme, recentNews, totalNews }: StockDetailProps) => {
    let current =
      translations.find((t) => t.locale === locale) ?? translations[0] ?? null
    let displayName = current?.name ?? stock.symbol
    return (
      <Layout title={displayName} subtitle={`${stock.symbol} · ${stock.market_code}`} locale={locale} theme={theme}>
        <Breadcrumb stock={stock} />

        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: '2fr 1fr',
            gap: space[4],
            marginTop: space[4],
            '@media (max-width: 880px)': { gridTemplateColumns: '1fr' },
          })}
        >
          <Card>
            <div
              mix={css({
                display: 'flex',
                alignItems: 'center',
                gap: space[3],
                marginBottom: space[4],
              })}
            >
              <StockBadge symbol={stock.symbol} size={40} />
              <div>
                <div
                  mix={css({
                    fontSize: font.xl,
                    fontWeight: 700,
                    color: color.text,
                    letterSpacing: '-0.01em',
                  })}
                >
                  {current?.name ?? stock.symbol}
                </div>
                <div mix={css({ fontSize: font.sm, color: color.textMuted })}>
                  {current
                    ? `${current.locale} · updated ${current.updated_at.slice(0, 10)}`
                    : 'no translation yet'}
                </div>
              </div>
            </div>

            {current?.description_md ? (
              <Description text={current.description_md} />
            ) : (
              <EmptyState
                title="No description for this locale"
                hint={
                  <>
                    Add one with{' '}
                    <code>{`PUT /api/v1/stocks/${stock.id}/translations/${locale}`}</code>
                  </>
                }
              />
            )}
          </Card>

          <Card>
            <SectionTitle>Metadata</SectionTitle>
            <Metadata stock={stock} />
          </Card>
        </div>

        <div mix={css({ marginTop: space[4] })}>
          <Card>
            <SectionTitle hint={`${recentNews.length} of ${totalNews} shown`}>
              Recent news
            </SectionTitle>
            <NewsList items={recentNews} />
          </Card>
        </div>
      </Layout>
    )
  }
}

function Breadcrumb() {
  return ({ stock }: { stock: Stock }) => (
    <div
      mix={css({
        display: 'flex',
        alignItems: 'center',
        gap: space[2],
        fontSize: font.sm,
        color: color.textMuted,
      })}
    >
      <a
        href="/stocks"
        mix={css({
          color: color.textMuted,
          textDecoration: 'none',
          '&:hover': { color: color.text },
        })}
      >
        Stocks
      </a>
      <span>·</span>
      <span mix={css({ color: color.text, fontWeight: 500 })}>{stock.symbol}</span>
    </div>
  )
}

function NewsList() {
  return ({
    items,
  }: {
    items: Array<{ link: NewsStockLink; item: NewsItem }>
  }) => {
    if (items.length === 0) {
      return (
        <EmptyState
          title="No news linked"
          hint={
            <>
              Agent can attach via <code>POST /api/v1/news/:id/stock-links</code>.
            </>
          }
        />
      )
    }
    return (
      <ul mix={css({ listStyle: 'none', margin: 0, padding: 0 })}>
        {items.map(({ link, item: n }) => (
          <li
            mix={css({
              padding: `${space[3]} 0`,
              borderTop: `1px solid ${color.borderSoft}`,
              '&:first-child': { borderTop: 'none', paddingTop: 0 },
            })}
          >
            <div
              mix={css({
                display: 'flex',
                alignItems: 'baseline',
                gap: space[2],
                fontSize: font.xs,
                color: color.textDim,
                marginBottom: space[1],
              })}
            >
              <span>{n.published_at.slice(0, 10)}</span>
              <span>·</span>
              <span>{n.source}</span>
              <span>·</span>
              <Badge tone={link.relation === 'primary' ? 'brand' : 'neutral'}>
                {link.relation}
              </Badge>
              {n.sentiment && (
                <Badge
                  tone={
                    n.sentiment === 'positive'
                      ? 'success'
                      : n.sentiment === 'negative'
                        ? 'danger'
                        : 'neutral'
                  }
                >
                  {n.sentiment}
                </Badge>
              )}
            </div>
            <a
              href={`/news/${n.id}`}
              mix={css({
                fontSize: font.base,
                color: color.text,
                textDecoration: 'none',
                fontWeight: 500,
                '&:hover': { color: color.brandHover },
              })}
            >
              {n.title}
            </a>
          </li>
        ))}
      </ul>
    )
  }
}

function Description() {
  return ({ text }: { text: string }) => (
    <pre
      mix={css({
        margin: 0,
        padding: `${space[3]} ${space[4]}`,
        background: color.bg,
        border: `1px solid ${color.borderSoft}`,
        borderRadius: '6px',
        fontSize: font.base,
        lineHeight: 1.6,
        color: color.text,
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
        fontFamily: 'inherit',
      })}
    >
      {text}
    </pre>
  )
}

function Metadata() {
  return ({ stock }: { stock: Stock }) => (
    <dl
      mix={css({
        margin: 0,
        display: 'grid',
        gridTemplateColumns: 'auto 1fr',
        gap: `${space[2]} ${space[4]}`,
        fontSize: font.sm,
      })}
    >
      <Row label="Symbol" value={stock.symbol} mono />
      <Row label="Market" value={stock.market_code} mono />
      <Row label="Currency" value={stock.currency} mono />
      <Row label="Asset class" value={stock.asset_class} />
      <Row label="Lot size" value={stock.lot_size != null ? String(stock.lot_size) : '—'} />
      <Row label="ISIN" value={stock.isin ?? '—'} mono />
      <Row label="FIGI" value={stock.figi ?? '—'} mono />
      <Row label="ID" value={`#${stock.id}`} mono />
      <Row label="Created" value={stock.created_at.slice(0, 10)} />
      <Row label="Updated" value={stock.updated_at.slice(0, 10)} />
    </dl>
  )
}

function Row() {
  return ({
    label,
    value,
    mono = false,
  }: {
    label: string
    value: string
    mono?: boolean
  }) => (
    <>
      <dt
        mix={css({
          fontSize: font.xs,
          color: color.textMuted,
          textTransform: 'uppercase',
          letterSpacing: '0.06em',
          alignSelf: 'center',
        })}
      >
        {label}
      </dt>
      <dd
        mix={css({
          margin: 0,
          color: color.text,
          fontFamily: mono ? font.mono : 'inherit',
        })}
      >
        {value}
      </dd>
    </>
  )
}
