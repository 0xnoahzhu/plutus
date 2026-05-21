import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import {
  api,
  type Account,
  type NewsItem,
  type NewsStockLink,
  type PendingOrder,
  type Stock,
  type TradePlan,
  type TradePlanLevel,
} from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { OrdersTable } from './orders.tsx'
import {
  Badge,
  type BadgeTone,
  Card,
  color,
  EmptyState,
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
import { fmtMoney } from '../ui/format.ts'
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

    let [stock, newsLinks, allNews, plans, openOrders, accounts] = await Promise.all([
      api.stock(id, locale).catch(() => null),
      api.newsForStock(id).catch(() => [] as NewsStockLink[]),
      api.news(locale).catch(() => [] as NewsItem[]),
      api.tradePlans({ stock_id: id }).catch(() => [] as TradePlan[]),
      api.pendingOrders({ stock_id: id, status: 'open' }).catch(
        () => [] as PendingOrder[],
      ),
      api.accounts().catch(() => [] as Account[]),
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

    // Pull the levels for each plan in parallel. Plans sorted with active
    // first, then most-recent created within each status group.
    plans.sort(
      (a, b) =>
        Number(b.status === 'active') - Number(a.status === 'active') ||
        b.created_at.localeCompare(a.created_at),
    )
    let levelsPerPlan = await Promise.all(
      plans.map((p) => api.tradePlanLevels(p.id).catch(() => [] as TradePlanLevel[])),
    )
    let plansWithLevels = plans.map((p, i) => ({ plan: p, levels: levelsPerPlan[i] ?? [] }))
    let accountMap = new Map<number, Account>(accounts.map((a) => [a.id, a]))
    let stockMap = new Map<number, Stock>([[stock.id, stock]])

    return render(
      <StockDetailPage
        stock={stock}
        locale={locale}
        theme={theme}
        recentNews={recentTrimmed}
        totalNews={recentNews.length}
        plans={plansWithLevels}
        openOrders={openOrders}
        accountMap={accountMap}
        stockMap={stockMap}
      />,
      request,
      { locale, theme },
    )
  },
}

interface StockDetailProps {
  stock: Stock
  locale: string
  theme: Theme
  recentNews: Array<{ link: NewsStockLink; item: NewsItem }>
  totalNews: number
  plans: Array<{ plan: TradePlan; levels: TradePlanLevel[] }>
  openOrders: PendingOrder[]
  accountMap: Map<number, Account>
  stockMap: Map<number, Stock>
}

function StockDetailPage() {
  return ({
    stock,
    locale,
    theme,
    recentNews,
    totalNews,
    plans,
    openOrders,
    accountMap,
    stockMap,
  }: StockDetailProps) => {
    let displayName = stock.name ?? stock.symbol
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
                  {displayName}
                </div>
                <div mix={css({ fontSize: font.sm, color: color.textMuted })}>
                  {stock.name
                    ? `${locale} · updated ${stock.updated_at.slice(0, 10)}`
                    : 'no name for this locale'}
                </div>
              </div>
            </div>

            {stock.description_md ? (
              <Description text={stock.description_md} />
            ) : (
              <EmptyState
                title="No description for this locale"
                hint={
                  <>
                    Update via{' '}
                    <code>{`PATCH /api/v1/stocks/${stock.id}`}</code>{' '}
                    with the full multi-locale <code>content</code> blob.
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
            <div
              mix={css({
                display: 'flex',
                alignItems: 'baseline',
                justifyContent: 'space-between',
                gap: space[3],
              })}
            >
              <SectionTitle
                hint={
                  plans.length === 0
                    ? 'none yet'
                    : `${plans.length} plan${plans.length === 1 ? '' : 's'}`
                }
              >
                Trade plans
              </SectionTitle>
              <a
                href="/trade-plans"
                mix={css({
                  fontSize: font.xs,
                  color: color.brand,
                  textDecoration: 'none',
                  fontWeight: 600,
                  '&:hover': { textDecoration: 'underline' },
                })}
              >
                manage →
              </a>
            </div>
            <TradePlansSection plans={plans} stock={stock} />
          </Card>
        </div>

        <div mix={css({ marginTop: space[4] })}>
          <Card>
            <div
              mix={css({
                display: 'flex',
                alignItems: 'baseline',
                justifyContent: 'space-between',
                gap: space[3],
                marginBottom: space[3],
              })}
            >
              <SectionTitle
                hint={
                  openOrders.length === 0
                    ? 'none open'
                    : `${openOrders.length} open`
                }
              >
                Open orders
              </SectionTitle>
              <a
                href="/orders"
                mix={css({
                  fontSize: font.xs,
                  color: color.brand,
                  textDecoration: 'none',
                  fontWeight: 600,
                  '&:hover': { textDecoration: 'underline' },
                })}
              >
                manage →
              </a>
            </div>
            {openOrders.length === 0 ? (
              <EmptyState
                title={messages(locale).orders.stockDetailEmpty}
                hint={
                  <>
                    Record an order on the{' '}
                    <a
                      href="/orders"
                      mix={css({
                        color: color.brand,
                        textDecoration: 'none',
                        '&:hover': { textDecoration: 'underline' },
                      })}
                    >
                      Open orders
                    </a>{' '}
                    page after placing it with your broker.
                  </>
                }
              />
            ) : (
              <OrdersTable
                locale={locale}
                orders={openOrders}
                accountMap={accountMap}
                stockMap={stockMap}
                showStockColumn={false}
                showAccountColumn={true}
              />
            )}
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

/// Read-only summary of the user's trade plans for this stock. The full
/// CRUD UI lives on /trade-plans; this card is just a glanceable view
/// from the stock detail page with a "manage →" link in the header.
function TradePlansSection() {
  return ({
    plans,
    stock,
  }: {
    plans: Array<{ plan: TradePlan; levels: TradePlanLevel[] }>
    stock: Stock
  }) => {
    if (plans.length === 0) {
      return (
        <EmptyState
          title="No trade plans for this stock yet"
          hint={
            <>
              Define entry / stop-loss / take-profit price points on the{' '}
              <a
                href="/trade-plans"
                mix={css({
                  color: color.brand,
                  textDecoration: 'none',
                  '&:hover': { textDecoration: 'underline' },
                })}
              >
                Trade plans
              </a>{' '}
              page. They'll show up here once you create them.
            </>
          }
        />
      )
    }
    return (
      <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[3] })}>
        {plans.map(({ plan, levels }) => (
          <TradePlanCard plan={plan} levels={levels} stock={stock} />
        ))}
      </div>
    )
  }
}

function TradePlanCard() {
  return ({
    plan,
    levels,
    stock,
  }: {
    plan: TradePlan
    levels: TradePlanLevel[]
    stock: Stock
  }) => {
    let activeLevels = levels.filter((l) => l.status === 'active')
    let activeLevelsSorted = [...activeLevels].sort((a, b) => {
      let ap = Number(a.price)
      let bp = Number(b.price)
      return ap - bp
    })
    return (
      <div
        mix={css({
          border: `1px solid ${color.border}`,
          borderRadius: radius.md,
          padding: space[4],
          background: color.bg,
        })}
      >
        <div
          mix={css({
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: space[3],
            flexWrap: 'wrap',
            marginBottom: space[3],
          })}
        >
          <div
            mix={css({
              display: 'flex',
              alignItems: 'center',
              gap: space[2],
              flexWrap: 'wrap',
            })}
          >
            <Badge tone={plan.status === 'active' ? 'success' : 'neutral'}>
              {plan.status}
            </Badge>
            <span mix={css({ fontSize: font.sm, color: color.textMuted })}>
              {levels.length} level{levels.length === 1 ? '' : 's'} ·{' '}
              {activeLevels.length} active
            </span>
          </div>
          <span mix={css({ fontSize: font.xs, color: color.textDim })}>
            since {plan.created_at.slice(0, 10)}
          </span>
        </div>

        {plan.rationale && (
          <p
            mix={css({
              margin: `0 0 ${space[3]}`,
              fontSize: font.sm,
              color: color.textMuted,
              fontStyle: 'italic',
              lineHeight: 1.5,
            })}
          >
            {plan.rationale}
          </p>
        )}

        {activeLevelsSorted.length === 0 ? (
          <div mix={css({ fontSize: font.sm, color: color.textDim })}>
            No active levels (all triggered or cancelled). Use{' '}
            <a
              href="/trade-plans"
              mix={css({
                color: color.brand,
                textDecoration: 'none',
                '&:hover': { textDecoration: 'underline' },
              })}
            >
              Trade plans
            </a>{' '}
            to add more.
          </div>
        ) : (
          <table
            mix={css({
              width: '100%',
              borderCollapse: 'collapse',
              fontSize: font.sm,
            })}
          >
            <tbody>
              {activeLevelsSorted.map((l) => (
                <LevelRow level={l} stock={stock} />
              ))}
            </tbody>
          </table>
        )}
      </div>
    )
  }
}

function LevelRow() {
  return ({ level, stock }: { level: TradePlanLevel; stock: Stock }) => {
    let toneMap: Record<string, BadgeTone> = {
      buy: 'brand',
      stop_loss: 'danger',
      take_profit: 'success',
      trim: 'warn',
    }
    let kindLabels: Record<string, string> = {
      buy: 'Buy',
      stop_loss: 'Stop loss',
      take_profit: 'Take profit',
      trim: 'Trim',
    }
    let tone = toneMap[level.kind] ?? 'neutral'
    let label = kindLabels[level.kind] ?? level.kind
    let sizeDisplay =
      level.quantity != null
        ? `${level.quantity} sh`
        : level.fraction_pct != null
          ? `${level.fraction_pct}%`
          : '—'
    return (
      <tr mix={css({ borderTop: `1px solid ${color.borderSoft}` })}>
        <td mix={css({ padding: `${space[2]} ${space[3]} ${space[2]} 0`, width: '110px' })}>
          <Badge tone={tone}>{label}</Badge>
        </td>
        <td
          mix={css({
            padding: `${space[2]} ${space[3]}`,
            fontFamily: font.mono,
            color: color.text,
            fontVariantNumeric: 'tabular-nums',
          })}
        >
          {fmtMoney(level.price)} {stock.currency}
        </td>
        <td
          mix={css({
            padding: `${space[2]} ${space[3]}`,
            fontFamily: font.mono,
            color: color.textMuted,
            fontVariantNumeric: 'tabular-nums',
          })}
        >
          {sizeDisplay}
        </td>
        <td
          mix={css({
            padding: `${space[2]} 0 ${space[2]} ${space[3]}`,
            color: color.textDim,
            fontSize: font.xs,
          })}
        >
          {level.notes ?? ''}
        </td>
      </tr>
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
              {n.title ?? '(untitled)'}
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
