import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import {
  api,
  type NewsCountryLink,
  type NewsItem,
  type NewsMacroLink,
  type NewsSectorLink,
  type NewsStockLink,
  type Sector,
  type Stock,
} from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  type BadgeTone,
  Card,
  color,
  font,
  Layout,
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { MarkdownToggle } from '../ui/markdown.tsx'
import { render } from '../utils/render.tsx'

export const newsDetail: BuildAction<'GET', typeof routes.newsDetail> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad news id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)

    let [item, stockLinks, sectorLinks, macroLinks, countryLinks, sectors] =
      await Promise.all([
        api.newsItem(id, locale).catch(() => null),
        api.newsStockLinks(id).catch(() => [] as NewsStockLink[]),
        api.newsSectorLinks(id).catch(() => [] as NewsSectorLink[]),
        api.newsMacroLinks(id).catch(() => [] as NewsMacroLink[]),
        api.newsCountryLinks(id).catch(() => [] as NewsCountryLink[]),
        api.sectors().catch(() => [] as Sector[]),
      ])
    if (!item) return new Response('News not found', { status: 404 })

    // Resolve only the stocks this news item references (avoids the
    // 200-row /stocks cap for users with thousands of tickers).
    let stocks = await api
      .stocksByIds(stockLinks.map((l) => l.stock_id), locale)
      .catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    let sectorMap = new Map<string, Sector>(sectors.map((s) => [s.code, s]))

    return render(
      <NewsDetailPage
        item={item}
        stockLinks={stockLinks}
        sectorLinks={sectorLinks}
        macroLinks={macroLinks}
        countryLinks={countryLinks}
        locale={locale}
        theme={theme}
        stocks={stockMap}
        sectors={sectorMap}
      />,
      request,
      { locale, theme },
    )
  },
}

interface NewsDetailProps {
  item: NewsItem
  stockLinks: NewsStockLink[]
  sectorLinks: NewsSectorLink[]
  macroLinks: NewsMacroLink[]
  countryLinks: NewsCountryLink[]
  locale: string
  theme: Theme
  stocks: Map<number, Stock>
  sectors: Map<string, Sector>
}

function NewsDetailPage() {
  return ({
    item: n,
    stockLinks,
    sectorLinks,
    macroLinks,
    countryLinks,
    locale,
    theme,
    stocks,
    sectors,
  }: NewsDetailProps) => {
    let p = messages(locale).pages.newsDetail
    let displayTitle = n.title ?? '(untitled)'
    let missingTitle = n.title == null

    return (
      <Layout title={displayTitle} locale={locale} theme={theme}>
        <Breadcrumb />

        <div
          mix={css({
            marginTop: space[3],
            display: 'grid',
            gridTemplateColumns: '2fr 1fr',
            gap: space[4],
            '@media (max-width: 880px)': { gridTemplateColumns: '1fr' },
          })}
        >
          <Card>
            {/* Meta strip */}
            <div
              mix={css({
                display: 'flex',
                flexWrap: 'wrap',
                gap: space[2],
                alignItems: 'center',
                fontSize: font.xs,
                color: color.textMuted,
                marginBottom: space[3],
              })}
            >
              <strong mix={css({ color: color.text })}>{n.source}</strong>
              <span>·</span>
              <span><LocalTime value={n.published_at} format="datetime" /></span>
              <span>·</span>
              <Badge tone="neutral">{n.region}</Badge>
              <Badge tone="brand">{n.category}</Badge>
              {n.sentiment && (
                <Badge tone={sentimentTone(n.sentiment)}>{n.sentiment}</Badge>
              )}
            </div>

            {/* Missing-locale hint */}
            {missingTitle ? (
              <div
                mix={css({
                  padding: `${space[2]} ${space[3]}`,
                  background: color.warnSoft,
                  borderRadius: radius.md,
                  fontSize: font.sm,
                  color: color.warnText,
                  marginBottom: space[3],
                })}
              >
                No content for <code>{locale}</code> (or English fallback) yet.
              </div>
            ) : null}

            <h1
              mix={css({
                margin: `${space[2]} 0 ${space[3]}`,
                fontSize: font.xl,
                fontWeight: 700,
                color: color.text,
                lineHeight: 1.3,
                letterSpacing: '-0.01em',
              })}
            >
              {displayTitle}
            </h1>

            {n.summary && (
              <p
                mix={css({
                  margin: `0 0 ${space[4]}`,
                  fontSize: font.md,
                  color: color.textMuted,
                  lineHeight: 1.6,
                })}
              >
                {n.summary}
              </p>
            )}
            {n.agent_summary_md && (
              <div
                mix={css({
                  margin: `0 0 ${space[4]}`,
                  padding: `${space[3]} ${space[4]}`,
                  background: color.brandSoft,
                  borderLeft: `3px solid ${color.brand}`,
                  borderRadius: radius.md,
                  color: color.brandSoftText,
                })}
              >
                <div
                  mix={css({
                    fontSize: font.xs,
                    fontWeight: 700,
                    textTransform: 'uppercase',
                    letterSpacing: '0.08em',
                    marginBottom: space[1],
                  })}
                >
                  Agent take
                </div>
                <BodyText text={n.agent_summary_md} />
              </div>
            )}
            {n.content_md ? (
              <BodyText text={n.content_md} />
            ) : (
              <p
                mix={css({
                  color: color.textDim,
                  fontStyle: 'italic',
                  fontSize: font.sm,
                })}
              >
                (no full content stored — follow the original link)
              </p>
            )}

            <Links item={n} />
          </Card>

          <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[4] })}>
            <Card>
              <SectionTitle>{p.sectionRelatedStocks}</SectionTitle>
              {stockLinks.length === 0 ? (
                <Dim>none</Dim>
              ) : (
                <ul mix={css({ listStyle: 'none', margin: 0, padding: 0 })}>
                  {stockLinks.map((l) => {
                    let s = stocks.get(l.stock_id)
                    return (
                      <li
                        mix={css({
                          display: 'flex',
                          alignItems: 'center',
                          gap: space[2],
                          marginBottom: space[2],
                          '&:last-child': { marginBottom: 0 },
                        })}
                      >
                        {s ? (
                          <a
                            href={`/stocks/${s.id}`}
                            mix={css({
                              display: 'inline-flex',
                              alignItems: 'center',
                              gap: space[2],
                              textDecoration: 'none',
                              color: color.text,
                              '&:hover': { color: color.brandHover },
                            })}
                          >
                            <StockBadge symbol={s.symbol} size={22} />
                            <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>
                              {s.symbol}
                            </span>
                          </a>
                        ) : (
                          <span mix={css({ color: color.textMuted })}>#{l.stock_id}</span>
                        )}
                        <span
                          mix={css({
                            marginLeft: 'auto',
                            fontSize: font.xs,
                            color: color.textDim,
                          })}
                        >
                          {l.relation}
                          {l.relevance ? ` · ${l.relevance}` : ''}
                        </span>
                      </li>
                    )
                  })}
                </ul>
              )}
            </Card>

            <Card>
              <SectionTitle>{p.sectionSectors}</SectionTitle>
              {sectorLinks.length === 0 ? (
                <Dim>none</Dim>
              ) : (
                <ChipRow>
                  {sectorLinks.map((l) => {
                    let s = sectors.get(l.sector_code)
                    return (
                      <Badge tone="neutral">
                        {l.sector_code}
                        {s ? ` · ${s.name}` : ''}
                      </Badge>
                    )
                  })}
                </ChipRow>
              )}

              <div mix={css({ marginTop: space[4] })}>
                <SectionTitle>{p.sectionMacroIndicators}</SectionTitle>
                {macroLinks.length === 0 ? (
                  <Dim>none</Dim>
                ) : (
                  <ChipRow>
                    {macroLinks.map((l) => (
                      <Badge tone="neutral">{l.indicator_code}</Badge>
                    ))}
                  </ChipRow>
                )}
              </div>

              <div mix={css({ marginTop: space[4] })}>
                <SectionTitle>{p.sectionCountries}</SectionTitle>
                {countryLinks.length === 0 ? (
                  <Dim>none</Dim>
                ) : (
                  <ChipRow>
                    {countryLinks.map((l) => (
                      <Badge tone="brand">{l.country}</Badge>
                    ))}
                  </ChipRow>
                )}
              </div>
            </Card>
          </div>
        </div>
      </Layout>
    )
  }
}

function Breadcrumb() {
  return () => (
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
        href="/news"
        mix={css({
          color: color.textMuted,
          textDecoration: 'none',
          '&:hover': { color: color.text },
        })}
      >
        News
      </a>
      <span>·</span>
      <span mix={css({ color: color.text, fontWeight: 500 })}>Detail</span>
    </div>
  )
}

function sentimentTone(s: string): BadgeTone {
  if (s === 'positive' || s === 'bullish') return 'success'
  if (s === 'negative' || s === 'bearish') return 'danger'
  return 'neutral'
}

function Links() {
  return ({ item: n }: { item: NewsItem }) => (
    <div
      mix={css({
        marginTop: space[5],
        paddingTop: space[4],
        borderTop: `1px solid ${color.borderSoft}`,
        fontSize: font.xs,
        color: color.textMuted,
        display: 'flex',
        flexDirection: 'column',
        gap: space[1],
      })}
    >
      <div>
        Source URL:{' '}
        <a
          href={n.url}
          target="_blank"
          rel="noopener noreferrer"
          mix={css({ color: color.brandHover })}
        >
          {n.url}
        </a>
        {n.url_status && (
          <span mix={css({ marginLeft: space[2], color: color.textDim })}>
            HTTP {n.url_status}
          </span>
        )}
      </div>
      {n.canonical_url && n.canonical_url !== n.url && (
        <div>
          Canonical:{' '}
          <a
            href={n.canonical_url}
            target="_blank"
            rel="noopener noreferrer"
            mix={css({ color: color.brandHover })}
          >
            {n.canonical_url}
          </a>
        </div>
      )}
      {n.archive_url && (
        <div>
          Archive:{' '}
          <a
            href={n.archive_url}
            target="_blank"
            rel="noopener noreferrer"
            mix={css({ color: color.brandHover })}
          >
            {n.archive_url}
          </a>
        </div>
      )}
      {n.last_verified_at && (
        <div>
          last verified <LocalTime value={n.last_verified_at} format="datetime" />
        </div>
      )}
    </div>
  )
}

function Dim() {
  return ({ children }: { children: RemixNode }) => (
    <span mix={css({ fontSize: font.sm, color: color.textDim, fontStyle: 'italic' })}>
      {children}
    </span>
  )
}

function ChipRow() {
  return ({ children }: { children: RemixNode }) => (
    <div mix={css({ display: 'flex', flexWrap: 'wrap', gap: space[1] })}>{children}</div>
  )
}

function BodyText() {
  return ({ text }: { text: string }) => <MarkdownToggle source={text} />
}
