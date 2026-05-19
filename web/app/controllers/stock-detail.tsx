import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import {
  api,
  type NewsItem,
  type NewsStockLink,
  type Stock,
  type StockTranslation,
} from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const stockDetail: BuildAction<'GET', typeof routes.stockDetail> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) {
      return new Response('Bad stock id', { status: 400 })
    }
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)

    let [stock, translations, newsLinks, allNews] = await Promise.all([
      api.stock(id).catch(() => null),
      api.stockTranslations(id).catch(() => [] as StockTranslation[]),
      api.newsForStock(id).catch(() => [] as NewsStockLink[]),
      api.news(locale).catch(() => [] as NewsItem[]),
    ])
    if (!stock) {
      return new Response('Stock not found', { status: 404 })
    }
    // Join links to news items, newest first.
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
        recentNews={recentTrimmed}
        totalNews={recentNews.length}
      />,
      request,
      { locale },
    )
  },
}

interface StockDetailProps {
  stock: Stock
  translations: StockTranslation[]
  /** Active locale from the global switcher. */
  locale: string
  recentNews: Array<{ link: NewsStockLink; item: NewsItem }>
  totalNews: number
}

function StockDetailPage() {
  return ({ stock, translations, locale, recentNews, totalNews }: StockDetailProps) => {
    // Pick the translation that matches the global locale. Fall back to the
    // first one we have so a Chinese-only stock still renders in English mode
    // instead of an awkward empty card.
    let current =
      translations.find((t) => t.locale === locale) ?? translations[0] ?? null

    return (
      <Layout title={`${stock.symbol} · ${stock.market_code}`} locale={locale}>
        <a
          href="/stocks"
          mix={css({
            fontSize: '13px',
            color: '#64748b',
            textDecoration: 'none',
            '&:hover': { color: '#0f172a' },
          })}
        >
          ← Back to stocks
        </a>

        <div
          mix={css({
            marginTop: '12px',
            display: 'grid',
            gridTemplateColumns: '2fr 1fr',
            gap: '16px',
            '@media (max-width: 720px)': { gridTemplateColumns: '1fr' },
          })}
        >
          <Card>
            <h3
              mix={css({
                margin: '0 0 12px',
                fontSize: '14px',
                fontWeight: 600,
                color: '#0f172a',
                textTransform: 'uppercase',
                letterSpacing: '0.06em',
              })}
            >
              Company
            </h3>

            {current ? (
              <div>
                <div
                  mix={css({
                    fontSize: '22px',
                    fontWeight: 700,
                    color: '#0f172a',
                  })}
                >
                  {current.name}
                </div>
                <div
                  mix={css({
                    fontSize: '11px',
                    color: '#94a3b8',
                    marginTop: '4px',
                    textTransform: 'uppercase',
                    letterSpacing: '0.06em',
                  })}
                >
                  {current.locale} · updated {current.updated_at.slice(0, 10)}
                </div>
                <Description text={current.description_md} />
              </div>
            ) : (
              <EmptyState stockId={stock.id} locale={locale} />
            )}
          </Card>

          <Card>
            <h3
              mix={css({
                margin: '0 0 12px',
                fontSize: '14px',
                fontWeight: 600,
                color: '#0f172a',
                textTransform: 'uppercase',
                letterSpacing: '0.06em',
              })}
            >
              Metadata
            </h3>
            <Metadata stock={stock} />
          </Card>
        </div>

        {/* Recent news — full-width below the two-column row. */}
        <div mix={css({ marginTop: '16px' })}>
          <Card>
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
                  fontSize: '14px',
                  fontWeight: 600,
                  color: '#0f172a',
                  textTransform: 'uppercase',
                  letterSpacing: '0.06em',
                })}
              >
                Recent news
              </h3>
              <span mix={css({ fontSize: '12px', color: '#64748b' })}>
                {recentNews.length} of {totalNews} shown
              </span>
            </div>
            <NewsList items={recentNews} />
          </Card>
        </div>
      </Layout>
    )
  }
}

function NewsList() {
  return ({
    items,
  }: {
    items: Array<{ link: NewsStockLink; item: NewsItem }>
  }) => {
    if (items.length === 0) {
      return (
        <p mix={css({ color: '#94a3b8', fontSize: '13px', fontStyle: 'italic', margin: 0 })}>
          No news linked yet. Agent can attach with POST /api/v1/news/:id/stock-links.
        </p>
      )
    }
    return (
      <ul mix={css({ listStyle: 'none', margin: 0, padding: 0 })}>
        {items.map(({ link, item: n }) => (
          <li
            mix={css({
              padding: '8px 0',
              borderTop: '1px solid #f1f5f9',
              '&:first-child': { borderTop: 'none' },
            })}
          >
            <div
              mix={css({
                display: 'flex',
                alignItems: 'baseline',
                gap: '8px',
                fontSize: '11px',
                color: '#94a3b8',
                marginBottom: '2px',
              })}
            >
              <span>{n.published_at.slice(0, 10)}</span>
              <span>·</span>
              <span>{n.source}</span>
              <span>·</span>
              <span mix={css({ color: link.relation === 'primary' ? '#1d4ed8' : '#94a3b8' })}>
                {link.relation}
              </span>
              {n.sentiment && (
                <>
                  <span>·</span>
                  <span
                    mix={css({
                      color:
                        n.sentiment === 'positive'
                          ? '#166534'
                          : n.sentiment === 'negative'
                            ? '#991b1b'
                            : '#475569',
                    })}
                  >
                    {n.sentiment}
                  </span>
                </>
              )}
            </div>
            <a
              href={`/news/${n.id}`}
              mix={css({
                fontSize: '14px',
                color: '#0f172a',
                textDecoration: 'none',
                fontWeight: 500,
                '&:hover': { color: '#1d4ed8' },
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

function Card() {
  return ({ children }: { children: import('remix/ui').RemixNode }) => (
    <div
      mix={css({
        background: '#fff',
        borderRadius: '8px',
        padding: '20px',
        border: '1px solid #e2e8f0',
        boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
      })}
    >
      {children}
    </div>
  )
}

function Description() {
  return ({ text }: { text: string | null }) => {
    if (!text) {
      return (
        <p
          mix={css({
            marginTop: '12px',
            color: '#94a3b8',
            fontStyle: 'italic',
            fontSize: '14px',
          })}
        >
          (no description yet)
        </p>
      )
    }
    // Markdown rendering is out of scope for Phase 0 — show raw with whitespace
    // preserved. Markdown-it / similar can drop in later if we need formatting.
    return (
      <pre
        mix={css({
          marginTop: '12px',
          padding: '12px',
          background: '#f8fafc',
          border: '1px solid #e2e8f0',
          borderRadius: '6px',
          fontSize: '14px',
          lineHeight: 1.6,
          color: '#1f2937',
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-word',
          fontFamily: 'inherit',
        })}
      >
        {text}
      </pre>
    )
  }
}

function EmptyState() {
  return ({ stockId, locale }: { stockId: number; locale: string }) => (
    <div
      mix={css({
        marginTop: '16px',
        padding: '16px',
        background: '#fef9c3',
        border: '1px solid #fde68a',
        borderRadius: '6px',
        fontSize: '13px',
        lineHeight: 1.6,
        color: '#713f12',
      })}
    >
      <strong>No translation for this locale yet.</strong> Add one with:
      <pre
        mix={css({
          marginTop: '8px',
          padding: '10px',
          background: '#fffbeb',
          borderRadius: '4px',
          fontSize: '12px',
          color: '#451a03',
          overflowX: 'auto',
        })}
      >
        {`curl -X PUT ${api.base}/api/v1/stocks/${stockId}/translations/${locale} \\
  -H 'Content-Type: application/json' \\
  -d '{"name":"…","description_md":"…"}'`}
      </pre>
    </div>
  )
}

function Metadata() {
  return ({ stock }: { stock: Stock }) => (
    <dl
      mix={css({
        margin: 0,
        display: 'grid',
        gridTemplateColumns: 'auto 1fr',
        gap: '8px 16px',
        fontSize: '13px',
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
  return ({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) => (
    <>
      <dt
        mix={css({
          fontSize: '11px',
          color: '#64748b',
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
          color: '#0f172a',
          fontFamily: mono ? 'ui-monospace, SFMono-Regular, monospace' : 'inherit',
        })}
      >
        {value}
      </dd>
    </>
  )
}
