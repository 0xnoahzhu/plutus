//! Pending limit orders the user has placed with their broker. Two
//! surfaces: a top-level `/orders` list page with a create form +
//! per-row lifecycle actions, and a smaller "Open orders" card slot on
//! the stock-detail page (see stock-detail.tsx, which imports the inner
//! list component from here).
//!
//! Mirrors trade-plans.tsx: small focused BuildActions for each lifecycle
//! transition (fill / cancel / reopen / delete), all redirecting back to
//! `/orders` with a flash query string.

import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import {
  api,
  type Account,
  type PendingOrder,
  type Stock,
} from '../api.ts'
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
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

const SIDE_VALUES = new Set(['buy', 'sell'])
const TYPE_VALUES = new Set(['limit', 'stop', 'stop_limit'])

/// GET /orders — list every pending order the user has recorded.
export const orders: BuildAction<'GET', typeof routes.orders> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let flash = url.searchParams.get('flash')
    let error = url.searchParams.get('error')

    let [list, accounts, stocks] = await Promise.all([
      api.pendingOrders().catch(() => [] as PendingOrder[]),
      api.accounts().catch(() => [] as Account[]),
      api.stocks(locale).catch(() => [] as Stock[]),
    ])

    let accountMap = new Map<number, Account>(accounts.map((a) => [a.id, a]))
    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))

    return render(
      <OrdersPage
        locale={locale}
        theme={theme}
        orders={list}
        accounts={accounts}
        stocks={stocks}
        accountMap={accountMap}
        stockMap={stockMap}
        error={error}
        flash={flash}
      />,
      request,
      { locale, theme },
    )
  },
}

/// POST /orders/new — record a fresh pending order.
export const orderCreate: BuildAction<'POST', typeof routes.orderCreate> = {
  async handler({ request }) {
    let form = await request.formData()
    let account_id = Number(form.get('account_id') ?? 0)
    let stock_id = Number(form.get('stock_id') ?? 0)
    let side = String(form.get('side') ?? '').trim()
    let order_type = String(form.get('order_type') ?? '').trim()
    // Empty strings become null so the Rust side doesn't try to parse "".
    let limit_price = String(form.get('limit_price') ?? '').trim() || null
    let stop_price = String(form.get('stop_price') ?? '').trim() || null
    let quantity = String(form.get('quantity') ?? '').trim()
    let tif = String(form.get('time_in_force') ?? 'gtc').trim() || 'gtc'
    let expires_at = String(form.get('expires_at') ?? '').trim() || null
    let broker_ref = String(form.get('broker_order_ref') ?? '').trim() || null
    let notes = String(form.get('notes') ?? '').trim() || null
    let trade_plan_level_id_raw = String(form.get('trade_plan_level_id') ?? '').trim()
    let trade_plan_level_id = trade_plan_level_id_raw
      ? Number(trade_plan_level_id_raw)
      : null

    if (!account_id || !stock_id || !side || !order_type || !quantity) {
      return Response.redirect(new URL('/orders?error=missing', request.url), 303)
    }
    if (!SIDE_VALUES.has(side)) {
      return Response.redirect(new URL('/orders?error=bad-side', request.url), 303)
    }
    if (!TYPE_VALUES.has(order_type)) {
      return Response.redirect(new URL('/orders?error=bad-type', request.url), 303)
    }
    // GTD: turn a datetime-local string into an RFC3339-ish UTC stamp.
    // The browser hands us `2026-05-20T14:30` (local, no zone); appending
    // `:00Z` is a pragmatic approximation — exact zone handling can come
    // later if users complain.
    let expires_at_iso = expires_at ? `${expires_at}:00Z` : null

    let cookie = request.headers.get('cookie')
    let upstream = await api.createPendingOrderRaw(cookie, {
      account_id,
      stock_id,
      trade_plan_level_id,
      side,
      order_type,
      limit_price,
      stop_price,
      quantity,
      time_in_force: tif,
      expires_at: expires_at_iso,
      broker_order_ref: broker_ref,
      notes,
    })
    if (!upstream.ok) {
      return Response.redirect(new URL('/orders?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/orders?flash=created', request.url), 303)
  },
}

/// POST /orders/:id/fill — flip to status=filled (stamps filled_at server-side).
export const orderFill: BuildAction<'POST', typeof routes.orderFill> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/orders?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.updatePendingOrderRaw(cookie, id, { status: 'filled' })
    if (!upstream.ok) {
      return Response.redirect(new URL('/orders?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/orders?flash=filled', request.url), 303)
  },
}

/// POST /orders/:id/cancel — flip to status=cancelled.
export const orderCancel: BuildAction<'POST', typeof routes.orderCancel> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/orders?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.updatePendingOrderRaw(cookie, id, { status: 'cancelled' })
    if (!upstream.ok) {
      return Response.redirect(new URL('/orders?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/orders?flash=cancelled', request.url), 303)
  },
}

/// POST /orders/:id/reopen — flip back to status=open (clears stamps).
export const orderReopen: BuildAction<'POST', typeof routes.orderReopen> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/orders?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.updatePendingOrderRaw(cookie, id, { status: 'open' })
    if (!upstream.ok) {
      return Response.redirect(new URL('/orders?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/orders?flash=reopened', request.url), 303)
  },
}

/// POST /orders/:id/delete — hard delete the row.
export const orderDelete: BuildAction<'POST', typeof routes.orderDelete> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/orders?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.deletePendingOrderRaw(cookie, id)
    if (!upstream.ok) {
      return Response.redirect(new URL('/orders?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/orders?flash=deleted', request.url), 303)
  },
}

// ── Page ─────────────────────────────────────────────────────────────────

interface PageProps {
  locale: string
  theme: Theme
  orders: PendingOrder[]
  accounts: Account[]
  stocks: Stock[]
  accountMap: Map<number, Account>
  stockMap: Map<number, Stock>
  error: string | null
  flash: string | null
}

function OrdersPage() {
  return ({
    locale,
    theme,
    orders,
    accounts,
    stocks,
    accountMap,
    stockMap,
    error,
    flash,
  }: PageProps) => {
    let m = messages(locale)
    let p = m.orders
    let title = m.pages.orders
    let canCreate = accounts.length > 0 && stocks.length > 0

    return (
      <Layout
        title={title.title}
        subtitle={title.subtitle}
        locale={locale}
        theme={theme}
      >
        {(error || flash) && (
          <div mix={css({ marginBottom: space[4] })}>
            <Banner error={error} flash={flash} locale={locale} />
          </div>
        )}

        <Card>
          <SectionTitle>{p.createSection}</SectionTitle>
          {!canCreate ? (
            <div mix={css({ marginTop: space[3] })}>
              <EmptyState
                title={accounts.length === 0 ? p.accountMissing : p.stockMissing}
              />
            </div>
          ) : (
            <CreateForm locale={locale} accounts={accounts} stocks={stocks} />
          )}
        </Card>

        <div mix={css({ marginTop: space[5] })}>
          <OrdersTable
            locale={locale}
            orders={orders}
            accountMap={accountMap}
            stockMap={stockMap}
            showStockColumn
            showAccountColumn
          />
        </div>
      </Layout>
    )
  }
}

// ── Create form ──────────────────────────────────────────────────────────

function CreateForm() {
  return ({
    locale,
    accounts,
    stocks,
  }: {
    locale: string
    accounts: Account[]
    stocks: Stock[]
  }) => {
    let p = messages(locale).orders
    return (
      <form
        method="post"
        action="/orders/new"
        mix={css({
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
          gap: space[3],
          marginTop: space[3],
          alignItems: 'start',
        })}
      >
        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.accountLabel}</span>
          <select name="account_id" required mix={css(fieldStyle)}>
            <option value="">{p.accountPlaceholder}</option>
            {accounts.map((a) => (
              <option value={a.id}>
                {a.name} — {a.base_currency}
              </option>
            ))}
          </select>
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.stockLabel}</span>
          <select name="stock_id" required mix={css(fieldStyle)}>
            <option value="">{p.stockPlaceholder}</option>
            {stocks.map((s) => (
              <option value={s.id}>
                {s.symbol}
                {s.name ? ` — ${s.name}` : ''}
              </option>
            ))}
          </select>
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.sideLabel}</span>
          <select name="side" required mix={css(fieldStyle)}>
            <option value="buy">{p.sideBuy}</option>
            <option value="sell">{p.sideSell}</option>
          </select>
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.orderTypeLabel}</span>
          <select name="order_type" required mix={css(fieldStyle)}>
            <option value="limit">{p.orderTypeLimit}</option>
            <option value="stop">{p.orderTypeStop}</option>
            <option value="stop_limit">{p.orderTypeStopLimit}</option>
          </select>
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.limitPriceLabel}</span>
          <input
            type="text"
            name="limit_price"
            inputmode="decimal"
            placeholder="0.00"
            mix={css(fieldStyle)}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.stopPriceLabel}</span>
          <input
            type="text"
            name="stop_price"
            inputmode="decimal"
            placeholder="0.00"
            mix={css(fieldStyle)}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.quantityLabel}</span>
          <input
            type="text"
            name="quantity"
            inputmode="decimal"
            required
            placeholder="0"
            mix={css(fieldStyle)}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.tifLabel}</span>
          <select name="time_in_force" mix={css(fieldStyle)}>
            <option value="gtc">{p.tifGtc}</option>
            <option value="day">{p.tifDay}</option>
            <option value="gtd">{p.tifGtd}</option>
          </select>
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.expiresAtLabel}</span>
          <input type="datetime-local" name="expires_at" mix={css(fieldStyle)} />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.brokerRefLabel}</span>
          <input
            type="text"
            name="broker_order_ref"
            placeholder={p.brokerRefPlaceholder}
            mix={css(fieldStyle)}
          />
        </label>

        <label mix={css({ ...labelWrap, gridColumn: '1 / -1' })}>
          <span mix={css(labelText)}>{p.notesLabel}</span>
          <input type="text" name="notes" mix={css(fieldStyle)} />
        </label>

        <div mix={css({ gridColumn: '1 / -1' })}>
          <button type="submit" mix={css(primaryButton)}>
            {p.createSubmit}
          </button>
        </div>
      </form>
    )
  }
}

// ── List / table (also exported for use in stock-detail) ─────────────────

/// Renders the order list as a table with per-row lifecycle buttons.
/// Reused on stock-detail.tsx via `showStockColumn={false}` so the
/// stock column doesn't repeat the parent page's heading.
export function OrdersTable() {
  return ({
    locale,
    orders,
    accountMap,
    stockMap,
    showStockColumn,
    showAccountColumn,
  }: {
    locale: string
    orders: PendingOrder[]
    accountMap: Map<number, Account>
    stockMap: Map<number, Stock>
    showStockColumn: boolean
    showAccountColumn: boolean
  }) => {
    let m = messages(locale)
    let p = m.orders

    if (orders.length === 0) {
      return (
        <Card>
          <EmptyState title={p.emptyTitle} hint={p.emptyHint} />
        </Card>
      )
    }

    return (
      <Card>
        <SectionTitle>{m.nav.orders}</SectionTitle>
        <table
          mix={css({
            width: '100%',
            borderCollapse: 'collapse',
            fontSize: font.base,
            marginTop: space[3],
          })}
        >
          <thead>
            <tr>
              {showStockColumn && <Th>{p.columnStock}</Th>}
              {showAccountColumn && <Th>{p.columnAccount}</Th>}
              <Th>{p.columnSide}</Th>
              <Th>{p.columnType}</Th>
              <Th>{p.columnPrice}</Th>
              <Th>{p.columnQuantity}</Th>
              <Th>{p.columnTif}</Th>
              <Th>{p.columnStatus}</Th>
              <Th>{p.columnPlaced}</Th>
              <Th>{''}</Th>
            </tr>
          </thead>
          <tbody>
            {orders.map((o) => (
              <OrderRow
                order={o}
                stock={stockMap.get(o.stock_id) ?? null}
                account={accountMap.get(o.account_id) ?? null}
                locale={locale}
                showStockColumn={showStockColumn}
                showAccountColumn={showAccountColumn}
              />
            ))}
          </tbody>
        </table>
      </Card>
    )
  }
}

function OrderRow() {
  return ({
    order,
    stock,
    account,
    locale,
    showStockColumn,
    showAccountColumn,
  }: {
    order: PendingOrder
    stock: Stock | null
    account: Account | null
    locale: string
    showStockColumn: boolean
    showAccountColumn: boolean
  }) => {
    let all = messages(locale)
    let p = all.orders
    let confirms = all.confirms

    let symbol = stock?.symbol ?? `#${order.stock_id}`
    let priceText =
      order.order_type === 'stop'
        ? order.stop_price ?? '—'
        : order.order_type === 'stop_limit'
          ? `${order.stop_price ?? '—'} / ${order.limit_price ?? '—'}`
          : order.limit_price ?? '—'
    let sideLabel = order.side === 'buy' ? p.sideBuy : p.sideSell
    let sideTone: BadgeTone = order.side === 'buy' ? 'success' : 'danger'
    let typeLabel =
      order.order_type === 'limit'
        ? p.orderTypeLimit
        : order.order_type === 'stop'
          ? p.orderTypeStop
          : p.orderTypeStopLimit
    let tifLabel =
      order.time_in_force === 'gtc'
        ? p.tifGtc
        : order.time_in_force === 'day'
          ? p.tifDay
          : p.tifGtd
    let statusLabel =
      order.status === 'open'
        ? p.statusOpen
        : order.status === 'filled'
          ? p.statusFilled
          : order.status === 'cancelled'
            ? p.statusCancelled
            : p.statusExpired
    let statusTone: BadgeTone =
      order.status === 'open'
        ? 'info'
        : order.status === 'filled'
          ? 'success'
          : order.status === 'cancelled'
            ? 'neutral'
            : 'warn'
    let isOpen = order.status === 'open'

    return (
      <tr mix={css({ borderTop: `1px solid ${color.borderSoft}` })}>
        {showStockColumn && (
          <Td>
            <div mix={css({ display: 'inline-flex', alignItems: 'center', gap: space[2] })}>
              <StockBadge symbol={symbol} />
              <span mix={css({ fontFamily: font.mono })}>{symbol}</span>
            </div>
          </Td>
        )}
        {showAccountColumn && <Td>{account?.name ?? `#${order.account_id}`}</Td>}
        <Td>
          <Badge tone={sideTone}>{sideLabel}</Badge>
        </Td>
        <Td>{typeLabel}</Td>
        <Td>
          <span mix={css({ fontVariantNumeric: 'tabular-nums' })}>{priceText}</span>
        </Td>
        <Td>
          <span mix={css({ fontVariantNumeric: 'tabular-nums' })}>{order.quantity}</span>
        </Td>
        <Td>{tifLabel}</Td>
        <Td>
          <Badge tone={statusTone}>{statusLabel}</Badge>
        </Td>
        <Td>
          <span mix={css({ fontSize: font.sm, color: color.textMuted })}>
            <LocalTime value={order.placed_at} format="date" />
          </span>
        </Td>
        <Td>
          <div mix={css({ display: 'inline-flex', gap: space[2], flexWrap: 'wrap' })}>
            {isOpen ? (
              <>
                <form
                  method="post"
                  action={`/orders/${order.id}/fill`}
                  mix={css({ margin: 0 })}
                >
                  <button type="submit" mix={css(secondaryButton)}>
                    {p.fillSubmit}
                  </button>
                </form>
                <form
                  method="post"
                  action={`/orders/${order.id}/cancel`}
                  mix={css({ margin: 0 })}
                >
                  <button
                    type="submit"
                    title={confirms.cancelOrder(sideLabel, symbol, priceText)}
                    mix={css(secondaryButton)}
                  >
                    {p.cancelSubmit}
                  </button>
                </form>
              </>
            ) : (
              <form
                method="post"
                action={`/orders/${order.id}/reopen`}
                mix={css({ margin: 0 })}
              >
                <button type="submit" mix={css(secondaryButton)}>
                  {p.reopenSubmit}
                </button>
              </form>
            )}
            <form
              method="post"
              action={`/orders/${order.id}/delete`}
              mix={css({ margin: 0 })}
            >
              <button
                type="submit"
                title={confirms.deleteOrder(sideLabel, symbol, priceText)}
                mix={css(dangerButton)}
              >
                {p.deleteSubmit}
              </button>
            </form>
          </div>
        </Td>
      </tr>
    )
  }
}

// ── Helpers ──────────────────────────────────────────────────────────────

function Banner() {
  return ({
    error,
    flash,
    locale,
  }: {
    error: string | null
    flash: string | null
    locale: string
  }) => {
    let { tone, message } = describe(error, flash, locale)
    if (!message) return null
    let bg = tone === 'error' ? color.dangerSoft : color.successSoft
    let fg = tone === 'error' ? color.dangerText : color.successText
    return (
      <div
        mix={css({
          padding: `${space[2]} ${space[3]}`,
          background: bg,
          color: fg,
          borderRadius: radius.md,
          fontSize: font.sm,
        })}
      >
        {message}
      </div>
    )
  }
}

function describe(error: string | null, flash: string | null, locale: string) {
  let p = messages(locale).orders
  if (error) {
    let table: Record<string, string> = {
      missing: p.errMissingCreate,
      'bad-side': p.errBadSide,
      'bad-type': p.errBadType,
      'bad-id': p.errBadId,
      server: p.errServer,
    }
    return { tone: 'error' as const, message: table[error] ?? p.errServer }
  }
  if (flash) {
    let table: Record<string, string> = {
      created: p.flashCreated,
      filled: p.flashFilled,
      cancelled: p.flashCancelled,
      reopened: p.flashReopened,
      deleted: p.flashDeleted,
    }
    return { tone: 'success' as const, message: table[flash] ?? '' }
  }
  return { tone: 'success' as const, message: '' }
}

function Th() {
  return ({ children }: { children: RemixNode }) => (
    <th
      mix={css({
        textAlign: 'left',
        padding: `${space[2]} ${space[3]}`,
        fontSize: font.xs,
        fontWeight: 600,
        textTransform: 'uppercase',
        letterSpacing: '0.06em',
        color: color.textMuted,
        borderBottom: `1px solid ${color.border}`,
      })}
    >
      {children}
    </th>
  )
}

function Td() {
  return ({ children }: { children: RemixNode }) => (
    <td
      mix={css({
        padding: `${space[2]} ${space[3]}`,
        color: color.text,
        verticalAlign: 'middle',
      })}
    >
      {children}
    </td>
  )
}

const fieldStyle = {
  width: '100%',
  padding: `${space[2]} ${space[3]}`,
  background: color.surface,
  border: `1px solid ${color.border}`,
  borderRadius: radius.md,
  fontSize: font.base,
  color: color.text,
  fontFamily: font.sans,
  outline: 'none',
  '&:focus': { borderColor: color.brand },
  '&::placeholder': { color: color.textDim },
}

const labelWrap = {
  display: 'flex',
  flexDirection: 'column' as const,
  gap: space[1],
}

const labelText = {
  fontSize: font.xs,
  fontWeight: 600,
  color: color.textMuted,
  textTransform: 'uppercase' as const,
  letterSpacing: '0.06em',
}

const primaryButton = {
  padding: `${space[2]} ${space[4]}`,
  background: color.brand,
  color: '#fff',
  border: 'none',
  borderRadius: radius.md,
  fontSize: font.base,
  fontWeight: 600,
  cursor: 'pointer',
  '&:hover': { background: color.brandHover },
}

const secondaryButton = {
  padding: `${space[1]} ${space[3]}`,
  background: 'transparent',
  border: `1px solid ${color.border}`,
  borderRadius: radius.md,
  color: color.text,
  fontSize: font.sm,
  fontWeight: 500,
  fontFamily: 'inherit',
  cursor: 'pointer',
  '&:hover': { background: color.hover },
}

const dangerButton = {
  padding: `${space[1]} ${space[3]}`,
  background: 'transparent',
  border: `1px solid ${color.border}`,
  borderRadius: radius.md,
  color: color.danger,
  fontSize: font.sm,
  fontWeight: 500,
  fontFamily: 'inherit',
  cursor: 'pointer',
  '&:hover': { background: color.dangerSoft, borderColor: color.danger },
}
