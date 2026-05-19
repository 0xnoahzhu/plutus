import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Recommendation, type Stock } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const recommendations: BuildAction<'GET', typeof routes.recommendations> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let [recs, stocks] = await Promise.all([
      api.recommendations({ locale }).catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    let open = recs.filter((r) => r.status === 'open')
    let closed = recs.filter((r) => r.status !== 'open')

    return render(
      <RecommendationsPage
        open={open}
        closed={closed}
        stocks={stockMap}
        locale={locale}
      />,
      request,
      { locale },
    )
  },
}

interface RecommendationsProps {
  open: Recommendation[]
  closed: Recommendation[]
  stocks: Map<number, Stock>
  locale: string
}

function RecommendationsPage() {
  return ({ open, closed, stocks, locale }: RecommendationsProps) => (
    <Layout title="Recommendations" locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Standalone buy/sell/reduce/hold calls made by the agent. Each is tracked
        from <code>issued_at</code> until it's closed with an outcome and PnL.
        Agent writes via <code>POST /api/v1/recommendations</code> and closes
        them via <code>POST /api/v1/recommendations/:id/close</code>.
      </p>

      <SectionHeader label="Open" sub={`${open.length}`} />
      {open.length === 0 ? (
        <Empty>No open recommendations.</Empty>
      ) : (
        <RecsTable recs={open} stocks={stocks} kind="open" />
      )}

      <div mix={css({ marginTop: '24px' })}>
        <SectionHeader label="Closed" sub={`${closed.length}`} />
      </div>
      {closed.length === 0 ? (
        <Empty>No closed recommendations recorded yet.</Empty>
      ) : (
        <RecsTable recs={closed} stocks={stocks} kind="closed" />
      )}
    </Layout>
  )
}

function SectionHeader() {
  return ({ label, sub }: { label: string; sub: string }) => (
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
          fontSize: '12px',
          fontWeight: 700,
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          color: '#0f172a',
        })}
      >
        {label}
      </h3>
      <span mix={css({ fontSize: '11px', color: '#94a3b8' })}>{sub}</span>
    </div>
  )
}

function Empty() {
  return ({ children }: { children: string }) => (
    <p mix={css({ color: '#94a3b8', fontStyle: 'italic', fontSize: '13px', margin: 0 })}>
      {children}
    </p>
  )
}

function RecsTable() {
  return ({
    recs,
    stocks,
    kind,
  }: {
    recs: Recommendation[]
    stocks: Map<number, Stock>
    kind: 'open' | 'closed'
  }) => (
    <div mix={css({ display: 'flex', flexDirection: 'column', gap: '8px' })}>
      {recs.map((r) => (
        <RecRow rec={r} stock={r.stock_id != null ? stocks.get(r.stock_id) : undefined} kind={kind} />
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
    <div
      mix={css({
        background: '#fff',
        border: '1px solid #e2e8f0',
        borderLeft: `3px solid ${actionAccent(rec.action)}`,
        borderRadius: '8px',
        padding: '12px 16px',
      })}
    >
      <div
        mix={css({
          display: 'flex',
          alignItems: 'baseline',
          gap: '8px',
          marginBottom: '6px',
          flexWrap: 'wrap',
        })}
      >
        <TargetChip rec={rec} stock={stock} />
        <ActionPill action={rec.action} />
        {rec.confidence && (
          <span
            mix={css({
              fontSize: '11px',
              fontWeight: 600,
              color: '#0f172a',
            })}
          >
            conf {rec.confidence}
          </span>
        )}
        <span mix={css({ fontSize: '11px', color: '#64748b' })}>
          horizon: {rec.target_horizon}
        </span>
        {rec.target_price && (
          <span mix={css({ fontSize: '11px', color: '#0f172a' })}>
            target {rec.target_price}
            {rec.target_currency ? ` ${rec.target_currency}` : ''}
          </span>
        )}
        {kind === 'closed' && <StatusPill status={rec.status} />}
        {rec.pnl_pct && <PnlPill pnl={rec.pnl_pct} />}
        <span mix={css({ marginLeft: 'auto', fontSize: '11px', color: '#94a3b8' })}>
          issued {rec.issued_at.slice(0, 10)} · {rec.source}
        </span>
      </div>
      <pre
        mix={css({
          margin: 0,
          padding: '8px 10px',
          background: '#f8fafc',
          border: '1px solid #e2e8f0',
          borderRadius: '4px',
          fontSize: '12px',
          lineHeight: 1.55,
          color: '#1f2937',
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-word',
          fontFamily: 'inherit',
        })}
      >
        {rec.rationale_md}
      </pre>
      {rec.outcome_md && (
        <pre
          mix={css({
            marginTop: '8px',
            padding: '8px 10px',
            background: '#fef9c3',
            border: '1px solid #fde68a',
            borderRadius: '4px',
            fontSize: '12px',
            lineHeight: 1.55,
            color: '#713f12',
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-word',
            fontFamily: 'inherit',
          })}
        >
          {rec.outcome_md}
        </pre>
      )}
    </div>
  )
}

function actionAccent(action: string): string {
  return action === 'buy'
    ? '#166534'
    : action === 'sell'
      ? '#991b1b'
      : action === 'reduce'
        ? '#d97706'
        : action === 'add'
          ? '#15803d'
          : '#475569'
}

function TargetChip() {
  return ({ rec, stock }: { rec: Recommendation; stock: Stock | undefined }) => {
    if (stock) {
      return (
        <a
          href={`/stocks/${stock.id}`}
          mix={css({
            fontFamily: 'ui-monospace, monospace',
            fontWeight: 600,
            color: '#1d4ed8',
            textDecoration: 'none',
            fontSize: '13px',
            '&:hover': { textDecoration: 'underline' },
          })}
        >
          {stock.symbol}
        </a>
      )
    }
    if (rec.sector_code) {
      return (
        <span
          mix={css({
            fontFamily: 'ui-monospace, monospace',
            fontSize: '11px',
            color: '#7c3aed',
            fontWeight: 600,
          })}
        >
          sector:{rec.sector_code}
        </span>
      )
    }
    return <span mix={css({ fontSize: '11px', color: '#94a3b8' })}>(no target)</span>
  }
}

function ActionPill() {
  return ({ action }: { action: string }) => {
    let bg = actionAccent(action)
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          background: bg,
          color: '#fff',
          fontSize: '10px',
          fontWeight: 700,
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
        })}
      >
        {action}
      </span>
    )
  }
}

function StatusPill() {
  return ({ status }: { status: string }) => {
    let palette: Record<string, [string, string]> = {
      closed_correct: ['#dcfce7', '#166534'],
      closed_wrong: ['#fee2e2', '#991b1b'],
      closed_neutral: ['#e2e8f0', '#475569'],
      expired: ['#fef3c7', '#92400e'],
      open: ['#dbeafe', '#1e40af'],
    }
    let [bg, fg] = palette[status] ?? ['#e2e8f0', '#475569']
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '999px',
          background: bg,
          color: fg,
          fontSize: '11px',
          fontWeight: 600,
        })}
      >
        {status.replace('closed_', '')}
      </span>
    )
  }
}

function PnlPill() {
  return ({ pnl }: { pnl: string }) => {
    let n = parseFloat(pnl)
    let pos = n > 0
    let color = pos ? '#166534' : n < 0 ? '#991b1b' : '#475569'
    let bg = pos ? '#dcfce7' : n < 0 ? '#fee2e2' : '#e2e8f0'
    let sign = pos ? '+' : ''
    return (
      <span
        mix={css({
          padding: '1px 8px',
          borderRadius: '4px',
          background: bg,
          color,
          fontSize: '11px',
          fontWeight: 700,
        })}
        title="PnL percentage from entry"
      >
        {sign}{pnl}%
      </span>
    )
  }
}
