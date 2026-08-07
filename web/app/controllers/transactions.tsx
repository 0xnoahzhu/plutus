//! The trade ledger. Two surfaces: the top-level `/transactions` page
//! with a record form, search, country filter and pagination, and a
//! read-only panel on the stock-detail page (which imports
//! [[TransactionsTable]] from here).
//!
//! `?stock_id=` scopes the page to one ticker — that's the target
//! holdings rows and the stock-detail "full history →" link point at, so
//! a position and the transactions that produced it are always one click
//! apart.

import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import { api, type Account, type Stock, type Transaction } from '../api.ts'
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
  SectionTitle,
  space,
  StockBadge,
  type Theme,
} from '../ui/layout.tsx'
import { fmtMoney } from '../ui/format.ts'
import { LocalTime } from '../ui/local-time.tsx'
import { Pagination, SearchBar } from '../ui/pagination.tsx'
import { render } from '../utils/render.tsx'

const PER_PAGE = 15

/// Canonical kinds, in the order the picker offers them. Values match
/// `plutus_core::transaction::TransactionKind::as_str()`; the server
/// rejects anything else with a 400, so this list is also the client-side
/// guard.
const KINDS = [
  'BUY',
  'SELL',
  'DIVIDEND',
  'FEE',
  'INTEREST',
  'DEPOSIT',
  'WITHDRAWAL',
  'FX',
  'CORPORATE_ACTION',
] as const

const KIND_VALUES = new Set<string>(KINDS)

/// Kind → badge tone. Money in is success, money out danger, the rest
/// stay informational so a long ledger doesn't read as all alarm.
const KIND_TONES: Record<string, BadgeTone> = {
  BUY: 'success',
  SELL: 'danger',
  DIVIDEND: 'warn',
  FEE: 'danger',
  INTEREST: 'warn',
  DEPOSIT: 'info',
  WITHDRAWAL: 'neutral',
  FX: 'brand',
  CORPORATE_ACTION: 'info',
}

function kindLabel(kind: string, locale: string): string {
  let p = messages(locale).pages.transactions
  let table: Record<string, string> = {
    BUY: p.kindBuy,
    SELL: p.kindSell,
    DIVIDEND: p.kindDividend,
    FEE: p.kindFee,
    INTEREST: p.kindInterest,
    DEPOSIT: p.kindDeposit,
    WITHDRAWAL: p.kindWithdrawal,
    FX: p.kindFx,
    CORPORATE_ACTION: p.kindCorporateAction,
  }
  return table[kind] ?? kind
}

export const transactions: BuildAction<'GET', typeof routes.transactions> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let country = parseCountry(url.searchParams)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let q = (url.searchParams.get('q') ?? '').trim()
    let flash = url.searchParams.get('flash')
    let error = url.searchParams.get('error')
    let pageParam = Number(url.searchParams.get('page') ?? '1')
    let page = Number.isFinite(pageParam) && pageParam > 0 ? Math.floor(pageParam) : 1
    let stockIdParam = Number(url.searchParams.get('stock_id') ?? '')
    let stockId = Number.isFinite(stockIdParam) && stockIdParam > 0 ? stockIdParam : null

    // Backend handles stock_id + country + q + pagination; we only need
    // to resolve stock symbols for the page slice so the table can
    // render market_code / symbol.
    let [result, accounts, dropdownStocks] = await Promise.all([
      api
        .transactionsPage({
          country: country || undefined,
          page,
          perPage: PER_PAGE,
          q: q || undefined,
          stock_id: stockId ?? undefined,
        })
        .catch(() => ({
          items: [] as Transaction[],
          total: 0,
          page,
          perPage: PER_PAGE,
        })),
      api.accounts().catch(() => [] as Account[]),
      // Suggestion pool for the ticker input. Capped by the backend, so
      // tickers past the cap are typed by hand and resolved server-side
      // via `?symbol=` on submit.
      api.stocks(locale).catch(() => [] as Stock[]),
    ])

    // Resolve symbols for the rows on this page by id — a stock past the
    // dropdown cap still has to render. Include the filtered stock so the
    // header can name it even on an empty page.
    let stockIds = result.items
      .map((t) => t.stock_id)
      .filter((id): id is number => id != null)
    if (stockId != null) stockIds.push(stockId)
    let rowStocks = await api.stocksByIds(stockIds, locale).catch(() => [] as Stock[])
    let stockMap = new Map<number, Stock>()
    for (let s of dropdownStocks) stockMap.set(s.id, s)
    for (let s of rowStocks) stockMap.set(s.id, s)
    let accountMap = new Map<number, Account>(accounts.map((a) => [a.id, a]))

    return render(
      <TransactionsPage
        rows={result.items}
        total={result.total}
        page={page}
        perPage={PER_PAGE}
        query={q}
        stocks={stockMap}
        accounts={accounts}
        accountMap={accountMap}
        dropdownStocks={dropdownStocks}
        stockId={stockId}
        country={country}
        locale={locale}
        theme={theme}
        error={error}
        flash={flash}
      />,
      request,
      { locale, theme },
    )
  },
}

/// POST /transactions/new — record a trade by hand.
///
/// Until now the ledger could only be written through the API, which
/// made the web UI useless for the one-off trade you did on your phone.
export const transactionCreate: BuildAction<
  'POST',
  typeof routes.transactionCreate
> = {
  async handler({ request }) {
    let form = await request.formData()
    let back = redirectTarget(form, request)
    let account_id = Number(form.get('account_id') ?? 0)
    let kind = String(form.get('kind') ?? '').trim().toUpperCase()
    // Blank ticker is legitimate — deposits, withdrawals and account-level
    // fees have no stock.
    let stock_symbol = String(form.get('stock_symbol') ?? '').trim()
    let resolvedStock = stock_symbol
      ? await api.stockBySymbol(stock_symbol).catch(() => null)
      : null
    if (stock_symbol && !resolvedStock) {
      return redirectWith(back, 'error', 'bad-symbol')
    }

    let executed_at = String(form.get('executed_at') ?? '').trim()
    let quantity = String(form.get('quantity') ?? '').trim()
    let price = String(form.get('price') ?? '').trim()
    // Default the currency to the stock's own so the common case is one
    // less field to fill in.
    let trade_currency =
      String(form.get('trade_currency') ?? '').trim().toUpperCase() ||
      resolvedStock?.currency ||
      ''
    let commission = String(form.get('commission') ?? '').trim() || '0'
    let commission_currency =
      String(form.get('commission_currency') ?? '').trim().toUpperCase() ||
      trade_currency
    let tax = String(form.get('tax') ?? '').trim() || '0'
    let tax_currency =
      String(form.get('tax_currency') ?? '').trim().toUpperCase() || trade_currency
    let fx_rate_to_base = String(form.get('fx_rate_to_base') ?? '').trim() || '1'
    let external_ref = String(form.get('external_ref') ?? '').trim() || null
    let notes = String(form.get('notes') ?? '').trim() || null

    if (!account_id || !kind || !executed_at || !quantity || !price || !trade_currency) {
      return redirectWith(back, 'error', 'missing')
    }
    if (!KIND_VALUES.has(kind)) {
      return redirectWith(back, 'error', 'bad-kind')
    }
    // `datetime-local` hands us `2026-05-20T14:30` with no zone. Same
    // pragmatic approximation the orders form makes: treat it as UTC.
    let executed_at_iso = `${executed_at}${executed_at.length === 16 ? ':00' : ''}Z`

    let cookie = request.headers.get('cookie')
    let upstream = await api.createTransactionRaw(cookie, {
      account_id,
      stock_id: resolvedStock ? resolvedStock.id : null,
      kind,
      executed_at: executed_at_iso,
      quantity,
      price,
      trade_currency,
      commission,
      commission_currency,
      tax,
      tax_currency,
      fx_rate_to_base,
      external_ref,
      notes,
      source: 'manual',
    })
    if (!upstream.ok) {
      return redirectWith(back, 'error', 'server')
    }
    return redirectWith(back, 'flash', 'created')
  },
}

/// POST /transactions/:id/delete — drop a row from the ledger.
export const transactionDelete: BuildAction<
  'POST',
  typeof routes.transactionDelete
> = {
  async handler({ request, params }) {
    let form = await request.formData()
    let back = redirectTarget(form, request)
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return redirectWith(back, 'error', 'bad-id')
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.deleteTransactionRaw(cookie, id)
    if (!upstream.ok) {
      return redirectWith(back, 'error', 'server')
    }
    return redirectWith(back, 'flash', 'deleted')
  },
}

/// Where to send the browser after a write. Forms carry the filtered
/// view they were submitted from (`/transactions?stock_id=7`) so the
/// user lands back where they were instead of on the unfiltered list.
///
/// Untrusted input — it arrives in the request body, so a crafted form
/// could point it anywhere. Resolve it against the current request
/// first, then check the *resolved* origin and path: checking the raw
/// string instead would wave through `/transactions/../admin`, which
/// `new URL()` later normalizes to `/admin`. Anything that doesn't land
/// on this origin under `/transactions` falls back to the plain page.
function redirectTarget(form: FormData, request: Request): URL {
  let here = new URL(request.url)
  let fallback = new URL('/transactions', here)
  let raw = String(form.get('return_to') ?? '')
  if (!raw.startsWith('/') || raw.startsWith('//')) return fallback
  let resolved: URL
  try {
    resolved = new URL(raw, here)
  } catch {
    return fallback
  }
  if (resolved.origin !== here.origin) return fallback
  if (
    resolved.pathname !== '/transactions' &&
    !resolved.pathname.startsWith('/transactions/')
  ) {
    return fallback
  }
  return resolved
}

function redirectWith(target: URL, key: 'flash' | 'error', value: string): Response {
  let url = new URL(target)
  // Drop any stale banner from the URL we came in on before adding ours.
  url.searchParams.delete('flash')
  url.searchParams.delete('error')
  url.searchParams.set(key, value)
  return Response.redirect(url, 303)
}

// ── Page ─────────────────────────────────────────────────────────────────

interface TxnProps {
  rows: Transaction[]
  total: number
  page: number
  perPage: number
  query: string
  stocks: Map<number, Stock>
  accounts: Account[]
  accountMap: Map<number, Account>
  dropdownStocks: Stock[]
  stockId: number | null
  country: string
  locale: string
  theme: Theme
  error: string | null
  flash: string | null
}

function TransactionsPage() {
  return ({
    rows,
    total,
    page,
    perPage,
    query,
    stocks,
    accounts,
    accountMap,
    dropdownStocks,
    stockId,
    country,
    locale,
    theme,
    error,
    flash,
  }: TxnProps) => {
    let p = messages(locale).pages.transactions
    let totalPages = Math.max(1, Math.ceil(total / perPage))
    let scoped = stockId != null ? (stocks.get(stockId) ?? null) : null
    let scopedLabel = scoped?.symbol ?? (stockId != null ? `#${stockId}` : '')
    // Writes bounce back to the filtered view when there is one.
    let returnTo =
      stockId != null ? `/transactions?stock_id=${stockId}` : '/transactions'
    return (
      <Layout
        title={p.title}
        subtitle={
          stockId != null
            ? p.subtitleStock(scopedLabel, total)
            : p.subtitle(total, country)
        }
        country={country}
        locale={locale}
        theme={theme}
      >
        {(error || flash) && (
          <div mix={css({ marginBottom: space[4] })}>
            <Banner error={error} flash={flash} locale={locale} />
          </div>
        )}

        {stockId != null && (
          <div mix={css({ marginBottom: space[4] })}>
            <StockFilterChip
              stock={scoped}
              stockId={stockId}
              label={scopedLabel}
              locale={locale}
            />
          </div>
        )}

        <Card>
          <SectionTitle>{p.createSection}</SectionTitle>
          {accounts.length === 0 ? (
            <EmptyState title={p.accountMissing} />
          ) : (
            <CreateForm
              locale={locale}
              accounts={accounts}
              stocks={dropdownStocks}
              presetSymbol={scoped?.symbol ?? null}
              presetCurrency={scoped?.currency ?? null}
              returnTo={returnTo}
            />
          )}
        </Card>

        {stockId == null && (
          <div mix={css({ marginTop: space[4] })}>
            <Card>
              <SearchBar
                action="/transactions"
                locale={locale}
                query={query}
                placeholder={p.searchPlaceholder}
                extraParams={{ country }}
              />
            </Card>
          </div>
        )}

        <div mix={css({ marginTop: space[4] })}>
          {rows.length === 0 ? (
            <Card>
              <EmptyState
                title={stockId != null ? p.emptyStockTitle : p.emptyTitle}
                hint={stockId != null ? p.emptyStockHint : p.emptyHint}
              />
            </Card>
          ) : (
            <TransactionsTable
              locale={locale}
              rows={rows}
              stocks={stocks}
              accounts={accountMap}
              showStockColumn={stockId == null}
              showAccountColumn
              showActions
              returnTo={returnTo}
            />
          )}
        </div>

        {totalPages > 1 && (
          <Pagination
            action="/transactions"
            locale={locale}
            page={page}
            totalPages={totalPages}
            total={total}
            perPage={perPage}
            query={query}
            extraParams={
              stockId != null
                ? { country, stock_id: String(stockId) }
                : { country }
            }
          />
        )}
      </Layout>
    )
  }
}

/// Header strip for the `?stock_id=` view: names the ticker, links to
/// its detail page, and offers a way back to the unfiltered list.
function StockFilterChip() {
  return ({
    stock,
    stockId,
    label,
    locale,
  }: {
    stock: Stock | null
    stockId: number
    label: string
    locale: string
  }) => {
    let p = messages(locale).pages.transactions
    return (
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: space[3],
          flexWrap: 'wrap',
          padding: `${space[2]} ${space[3]}`,
          background: color.hover,
          borderRadius: radius.md,
        })}
      >
        <a
          href={`/stocks/${stockId}`}
          mix={css({
            display: 'inline-flex',
            alignItems: 'center',
            gap: space[2],
            textDecoration: 'none',
            color: color.text,
            '&:hover': { color: color.brandHover },
          })}
        >
          <StockBadge symbol={label} size={22} />
          <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>{label}</span>
          {stock && <Badge tone="neutral">{stock.market_code}</Badge>}
        </a>
        <a
          href="/transactions"
          mix={css({
            fontSize: font.xs,
            color: color.brand,
            textDecoration: 'none',
            fontWeight: 600,
            '&:hover': { textDecoration: 'underline' },
          })}
        >
          {p.clearStockFilter}
        </a>
      </div>
    )
  }
}

// ── Create form ──────────────────────────────────────────────────────────

function CreateForm() {
  return ({
    locale,
    accounts,
    stocks,
    presetSymbol,
    presetCurrency,
    returnTo,
  }: {
    locale: string
    accounts: Account[]
    stocks: Stock[]
    /// Prefilled when the page is scoped to one ticker — recording
    /// another trade on the stock you're looking at shouldn't mean
    /// retyping it.
    presetSymbol: string | null
    presetCurrency: string | null
    returnTo: string
  }) => {
    let p = messages(locale).pages.transactions
    return (
      <form
        method="post"
        action="/transactions/new"
        mix={css({
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
          gap: space[3],
          marginTop: space[3],
          alignItems: 'start',
        })}
      >
        <input type="hidden" name="return_to" value={returnTo} />

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
          <span mix={css(labelText)}>{p.kindLabel}</span>
          <select name="kind" required mix={css(fieldStyle)}>
            {KINDS.map((k) => (
              <option value={k}>{kindLabel(k, locale)}</option>
            ))}
          </select>
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.stockLabel}</span>
          {/* Optional: cash movements (deposit / withdrawal / account
              fees) carry no stock. Same symbol + datalist approach the
              orders form uses — beats a <select> past a few hundred
              tickers and lets people just type what they know. */}
          <input
            type="text"
            name="stock_symbol"
            list="transactions-stock-symbols"
            placeholder={p.stockPlaceholder}
            value={presetSymbol ?? undefined}
            autocomplete="off"
            spellcheck={false}
            mix={css({ ...fieldStyle, textTransform: 'uppercase' })}
          />
          <datalist id="transactions-stock-symbols">
            {stocks.map((s) => (
              <option value={s.symbol} label={s.name ?? ''} />
            ))}
          </datalist>
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.executedAtLabel}</span>
          <input type="datetime-local" name="executed_at" required mix={css(fieldStyle)} />
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
          <span mix={css(labelText)}>{p.priceLabel}</span>
          <input
            type="text"
            name="price"
            inputmode="decimal"
            required
            placeholder="0.00"
            mix={css(fieldStyle)}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.tradeCurrencyLabel}</span>
          {/* Left blank the server falls back to the resolved stock's
              own currency, so this only needs filling for cash rows. */}
          <input
            type="text"
            name="trade_currency"
            placeholder="USD"
            value={presetCurrency ?? undefined}
            autocomplete="off"
            mix={css({ ...fieldStyle, textTransform: 'uppercase' })}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.commissionLabel}</span>
          <input
            type="text"
            name="commission"
            inputmode="decimal"
            placeholder="0.00"
            mix={css(fieldStyle)}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.commissionCurrencyLabel}</span>
          <input
            type="text"
            name="commission_currency"
            placeholder={presetCurrency ?? 'USD'}
            autocomplete="off"
            mix={css({ ...fieldStyle, textTransform: 'uppercase' })}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.taxLabel}</span>
          <input
            type="text"
            name="tax"
            inputmode="decimal"
            placeholder="0.00"
            mix={css(fieldStyle)}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.taxCurrencyLabel}</span>
          <input
            type="text"
            name="tax_currency"
            placeholder={presetCurrency ?? 'USD'}
            autocomplete="off"
            mix={css({ ...fieldStyle, textTransform: 'uppercase' })}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.fxRateLabel}</span>
          <input
            type="text"
            name="fx_rate_to_base"
            inputmode="decimal"
            placeholder="1"
            title={p.fxRateHint}
            mix={css(fieldStyle)}
          />
        </label>

        <label mix={css(labelWrap)}>
          <span mix={css(labelText)}>{p.externalRefLabel}</span>
          <input
            type="text"
            name="external_ref"
            placeholder={p.externalRefPlaceholder}
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

/// The ledger as a table. Reused on stock-detail with
/// `showStockColumn={false}` (the page already names the stock) and
/// `showActions={false}` (that panel is a read-only glance; the full
/// CRUD lives on `/transactions`).
export function TransactionsTable() {
  return ({
    locale,
    rows,
    stocks,
    accounts,
    showStockColumn,
    showAccountColumn,
    showActions,
    returnTo,
  }: {
    locale: string
    rows: Transaction[]
    stocks: Map<number, Stock>
    accounts: Map<number, Account>
    showStockColumn: boolean
    showAccountColumn: boolean
    showActions: boolean
    /// Where per-row writes should return to. Ignored when
    /// `showActions` is false.
    returnTo?: string
  }) => {
    let p = messages(locale).pages.transactions
    return (
      <Card padding="0">
        <table
          mix={css({
            width: '100%',
            borderCollapse: 'collapse',
            fontSize: font.base,
          })}
        >
          <thead>
            <tr>
              <Th>{p.columnDate}</Th>
              <Th>{p.columnKind}</Th>
              {showStockColumn && <Th>{p.columnSymbol}</Th>}
              {showStockColumn && <Th>{p.columnMarket}</Th>}
              {showAccountColumn && <Th>{p.columnAccount}</Th>}
              <Th align="right">{p.columnQty}</Th>
              <Th align="right">{p.columnPrice}</Th>
              <Th>{p.columnCurrency}</Th>
              <Th align="right">{p.columnCommission}</Th>
              <Th align="right">{p.columnTax}</Th>
              <Th>{p.columnSource}</Th>
              {showActions && <Th>{''}</Th>}
            </tr>
          </thead>
          <tbody>
            {rows.map((t) => (
              <TransactionRow
                txn={t}
                stock={t.stock_id != null ? (stocks.get(t.stock_id) ?? null) : null}
                account={accounts.get(t.account_id) ?? null}
                locale={locale}
                showStockColumn={showStockColumn}
                showAccountColumn={showAccountColumn}
                showActions={showActions}
                returnTo={returnTo ?? '/transactions'}
              />
            ))}
          </tbody>
        </table>
      </Card>
    )
  }
}

function TransactionRow() {
  return ({
    txn,
    stock,
    account,
    locale,
    showStockColumn,
    showAccountColumn,
    showActions,
    returnTo,
  }: {
    txn: Transaction
    stock: Stock | null
    account: Account | null
    locale: string
    showStockColumn: boolean
    showAccountColumn: boolean
    showActions: boolean
    returnTo: string
  }) => {
    let all = messages(locale)
    let p = all.pages.transactions
    let symbol = stock?.symbol ?? (txn.stock_id != null ? `#${txn.stock_id}` : '—')
    return (
      <tr
        // Row click opens the stock; the action buttons below stop the
        // event themselves, so a delete never navigates away first.
        data-row-href={txn.stock_id != null ? `/stocks/${txn.stock_id}` : undefined}
        mix={css({
          borderTop: `1px solid ${color.borderSoft}`,
          cursor: txn.stock_id != null ? 'pointer' : 'default',
          '&:hover td': { background: color.hover },
        })}
      >
        <Td>
          <span mix={css({ color: color.textMuted, fontFamily: font.mono })}>
            <LocalTime value={txn.executed_at} format="datetime" />
          </span>
        </Td>
        <Td>
          <Badge tone={KIND_TONES[txn.kind] ?? 'neutral'}>
            {kindLabel(txn.kind, locale)}
          </Badge>
        </Td>
        {showStockColumn && (
          <Td>
            {stock ? (
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
                <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>
                  {stock.symbol}
                </span>
              </a>
            ) : (
              <span mix={css({ color: color.textMuted })}>{symbol}</span>
            )}
          </Td>
        )}
        {showStockColumn && (
          <Td>{stock ? <Badge tone="neutral">{stock.market_code}</Badge> : '—'}</Td>
        )}
        {showAccountColumn && (
          <Td>
            <span mix={css({ fontSize: font.sm, color: color.textMuted })}>
              {account?.name ?? `#${txn.account_id}`}
            </span>
          </Td>
        )}
        <Td align="right" mono>
          {txn.quantity}
        </Td>
        <Td align="right" mono>
          {fmtMoney(txn.price)}
        </Td>
        <Td>{txn.trade_currency}</Td>
        <Td align="right" mono>
          {fmtMoney(txn.commission)} {txn.commission_currency}
        </Td>
        <Td align="right" mono>
          {fmtMoney(txn.tax)} {txn.tax_currency}
        </Td>
        <Td>
          <span
            mix={css({
              fontSize: font.xs,
              color: txn.source === 'agent' ? color.brandHover : color.textMuted,
              fontWeight: txn.source === 'agent' ? 600 : 400,
            })}
          >
            {txn.source}
          </span>
        </Td>
        {showActions && (
          <Td>
            <form
              method="post"
              action={`/transactions/${txn.id}/delete`}
              mix={css({ margin: 0 })}
            >
              <input type="hidden" name="return_to" value={returnTo} />
              <button
                type="submit"
                title={all.confirms.deleteTransaction(
                  kindLabel(txn.kind, locale),
                  symbol,
                  txn.quantity,
                )}
                mix={css(dangerButton)}
              >
                {p.deleteSubmit}
              </button>
            </form>
          </Td>
        )}
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
  let p = messages(locale).pages.transactions
  if (error) {
    let table: Record<string, string> = {
      missing: p.errMissingCreate,
      'bad-symbol': p.errBadSymbol,
      'bad-kind': p.errBadKind,
      'bad-id': p.errBadId,
      server: p.errServer,
    }
    return { tone: 'error' as const, message: table[error] ?? p.errServer }
  }
  if (flash) {
    let table: Record<string, string> = {
      created: p.flashCreated,
      deleted: p.flashDeleted,
    }
    return { tone: 'success' as const, message: table[flash] ?? '' }
  }
  return { tone: 'success' as const, message: '' }
}

function Th() {
  return ({
    children,
    align = 'left',
  }: {
    children: RemixNode
    align?: 'left' | 'right'
  }) => (
    <th
      mix={css({
        textAlign: align,
        padding: `${space[3]} ${space[4]}`,
        fontSize: font.xs,
        textTransform: 'uppercase',
        letterSpacing: '0.08em',
        color: color.textMuted,
        fontWeight: 600,
        background: color.hover,
        borderBottom: `1px solid ${color.border}`,
      })}
    >
      {children}
    </th>
  )
}

function Td() {
  return ({
    children,
    align = 'left',
    mono,
  }: {
    children: RemixNode
    align?: 'left' | 'right'
    mono?: boolean
  }) => (
    <td
      mix={css({
        padding: `${space[3]} ${space[4]}`,
        textAlign: align,
        fontVariantNumeric: 'tabular-nums',
        fontFamily: mono ? font.mono : 'inherit',
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
  color: color.textOnBrand,
  border: 'none',
  borderRadius: radius.md,
  fontSize: font.base,
  fontWeight: 600,
  cursor: 'pointer',
  '&:hover': { background: color.brandHover },
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
