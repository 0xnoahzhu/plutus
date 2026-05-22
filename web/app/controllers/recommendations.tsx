import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Recommendation, type Stock } from '../api.ts'
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
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { fmtMoney } from '../ui/format.ts'
import { LocalTime } from '../ui/local-time.tsx'
import { MarkdownToggle } from '../ui/markdown.tsx'
import { render } from '../utils/render.tsx'

export const recommendations: BuildAction<'GET', typeof routes.recommendations> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let recs = await api.recommendations({ locale }).catch(() => [])
    // Resolve symbols only for stocks the rec list touches — the
    // default /stocks endpoint is capped at 200, which silently drops
    // recommendations on tickers past that cap.
    let stocks = await api
      .stocksByIds(
        recs.map((r) => r.stock_id).filter((id): id is number => id != null),
        locale,
      )
      .catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    let open = recs.filter((r) => r.status === 'open')
    let closed = recs.filter((r) => r.status !== 'open')

    return render(
      <RecommendationsPage
        open={open}
        closed={closed}
        stocks={stockMap}
        locale={locale}
        theme={theme}
      />,
      request,
      { locale, theme },
    )
  },
}

interface RecommendationsProps {
  open: Recommendation[]
  closed: Recommendation[]
  stocks: Map<number, Stock>
  locale: string
  theme: Theme
}

function RecommendationsPage() {
  return ({ open, closed, stocks, locale, theme }: RecommendationsProps) => {
    let p = messages(locale).pages.recommendations
    return (
    <Layout
      title={p.title}
      subtitle="Standalone buy / sell / reduce / hold calls the agent tracks from issue until close-out with PnL."
      locale={locale}
      theme={theme}
    >
      <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[6] })}>
        <div>
          <SectionTitle hint={`${open.length}`}>Open</SectionTitle>
          {open.length === 0 ? (
            <Card>
              <EmptyState
                title="No open recommendations"
                hint={
                  <>
                    Agent writes via <code>POST /api/v1/recommendations</code>.
                  </>
                }
              />
            </Card>
          ) : (
            <RecsList recs={open} stocks={stocks} kind="open" />
          )}
        </div>

        <div>
          <SectionTitle hint={`${closed.length}`}>Closed</SectionTitle>
          {closed.length === 0 ? (
            <Card>
              <EmptyState title="No closed recommendations recorded yet" />
            </Card>
          ) : (
            <RecsList recs={closed} stocks={stocks} kind="closed" />
          )}
        </div>
      </div>
    </Layout>
    )
  }
}

function RecsList() {
  return ({
    recs,
    stocks,
    kind,
  }: {
    recs: Recommendation[]
    stocks: Map<number, Stock>
    kind: 'open' | 'closed'
  }) => (
    <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[3] })}>
      {recs.map((r) => (
        <RecRow
          rec={r}
          stock={r.stock_id != null ? stocks.get(r.stock_id) : undefined}
          kind={kind}
        />
      ))}
    </div>
  )
}

function RecRow() {
  return ({
    rec,
    stock,
    kind,
  }: {
    rec: Recommendation
    stock: Stock | undefined
    kind: 'open' | 'closed'
  }) => (
    <Card padding="0">
      <div
        mix={css({
          borderLeft: `3px solid ${actionAccent(rec.action)}`,
          padding: `${space[4]} ${space[5]}`,
          borderRadius: radius.lg,
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
          <Target rec={rec} stock={stock} />
          <Badge tone={actionTone(rec.action)}>{rec.action}</Badge>
          {rec.confidence && (
            <span
              mix={css({
                fontSize: font.xs,
                fontWeight: 600,
                color: color.text,
              })}
            >
              conf {rec.confidence}
            </span>
          )}
          <span mix={css({ fontSize: font.xs, color: color.textMuted })}>
            horizon: {rec.target_horizon}
          </span>
          {rec.target_price && (
            <span
              mix={css({
                fontSize: font.xs,
                color: color.text,
                fontFamily: font.mono,
              })}
            >
              target {fmtMoney(rec.target_price)}
              {rec.target_currency ? ` ${rec.target_currency}` : ''}
            </span>
          )}
          {kind === 'closed' && <Badge tone={statusTone(rec.status)}>{statusLabel(rec.status)}</Badge>}
          {rec.pnl_pct && <PnlPill pnl={rec.pnl_pct} />}
          <span
            mix={css({
              marginLeft: 'auto',
              fontSize: font.xs,
              color: color.textDim,
            })}
          >
            issued <LocalTime value={rec.issued_at} format="date" /> · {rec.source}
          </span>
        </div>

        {rec.rationale_md && <MarkdownToggle source={rec.rationale_md} />}

        {rec.outcome_md && (
          <div mix={css({ marginTop: space[2] })}>
            <MarkdownToggle source={rec.outcome_md} />
          </div>
        )}
      </div>
    </Card>
  )
}

function Target() {
  return ({ rec, stock }: { rec: Recommendation; stock: Stock | undefined }) => {
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
          <span
            mix={css({
              fontFamily: font.mono,
              fontWeight: 600,
              fontSize: font.base,
            })}
          >
            {stock.symbol}
          </span>
        </a>
      )
    }
    if (rec.sector_code) {
      return <Badge tone="brand">sector:{rec.sector_code}</Badge>
    }
    return (
      <span mix={css({ fontSize: font.xs, color: color.textDim })}>(no target)</span>
    )
  }
}

function PnlPill() {
  return ({ pnl }: { pnl: string }) => {
    let n = Number.parseFloat(pnl)
    let tone: BadgeTone = n > 0 ? 'success' : n < 0 ? 'danger' : 'neutral'
    let sign = n > 0 ? '+' : ''
    return (
      <Badge tone={tone} title="PnL percentage from entry">
        {`${sign}${pnl}%`}
      </Badge>
    )
  }
}

/// Left-border accent. Mirrors the action tone so the card communicates
/// direction at a glance, even when the badge is scrolled out of view.
function actionAccent(action: string): string {
  switch (action) {
    case 'buy':
    case 'add':
      return color.success
    case 'sell':
      return color.danger
    case 'reduce':
      return color.warn
    default:
      return color.border
  }
}

function actionTone(action: string): BadgeTone {
  switch (action) {
    case 'buy':
    case 'add':
      return 'success'
    case 'sell':
      return 'danger'
    case 'reduce':
      return 'warn'
    default:
      return 'neutral'
  }
}

function statusTone(status: string): BadgeTone {
  switch (status) {
    case 'closed_correct':
      return 'success'
    case 'closed_wrong':
      return 'danger'
    case 'closed_neutral':
    case 'expired':
      return 'neutral'
    case 'open':
      return 'info'
    default:
      return 'neutral'
  }
}

function statusLabel(status: string): string {
  return status.replace('closed_', '')
}
