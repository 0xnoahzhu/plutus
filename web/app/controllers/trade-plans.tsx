import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import {
  api,
  type Stock,
  type TradePlan,
  type TradePlanLevel,
} from '../api.ts'
import { messages, type Messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  type BadgeTone,
  Card,
  color,
  EmptyState,
  font,
  Layout,
  PageIntro,
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
import { render } from '../utils/render.tsx'

/// GET /trade-plans — list the caller's trade plans with nested level cards.
export const tradePlans: BuildAction<'GET', typeof routes.tradePlans> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let flash = url.searchParams.get('flash')
    let error = url.searchParams.get('error')

    let [plans, dropdownStocks] = await Promise.all([
      api.tradePlans().catch(() => [] as TradePlan[]),
      // The "new plan" form dropdown still uses the default listing
      // (capped at 200 by the backend). When the catalog grows past
      // 200 the user only sees the first 200 tickers in the picker;
      // a future change should swap this for a search/combobox.
      api.stocks(locale).catch(() => [] as Stock[]),
    ])
    // Plans -> per-plan levels in parallel. Empty fallback so the page
    // still renders if any single sub-request 500s.
    let levelLists = await Promise.all(
      plans.map((p) =>
        api.tradePlanLevels(p.id).catch(() => [] as TradePlanLevel[]),
      ),
    )
    let levelsByPlan = new Map<number, TradePlanLevel[]>()
    plans.forEach((p, i) => levelsByPlan.set(p.id, levelLists[i]))

    // Resolve symbols for the EXISTING plans by id so a plan whose
    // stock_id is past the 200-row cap still renders correctly.
    let extraStocks = await api
      .stocksByIds(plans.map((p) => p.stock_id), locale)
      .catch(() => [] as Stock[])
    let allStocks = dropdownStocks
    let stockMap = new Map<number, Stock>()
    for (let s of dropdownStocks) stockMap.set(s.id, s)
    for (let s of extraStocks) stockMap.set(s.id, s)
    // Plans come pre-sorted from the API: status asc (active before
    // closed alphabetically), then created_at desc.
    return render(
      <TradePlansPage
        locale={locale}
        theme={theme}
        plans={plans}
        levelsByPlan={levelsByPlan}
        stocks={stockMap}
        allStocks={allStocks}
        error={error}
        flash={flash}
      />,
      request,
      { locale, theme },
    )
  },
}

/// POST /trade-plans/new — create a fresh plan for the picked stock.
export const tradePlanCreate: BuildAction<'POST', typeof routes.tradePlanCreate> = {
  async handler({ request }) {
    let form = await request.formData()
    // Form posts a typed ticker symbol; resolve to stock_id server-side
    // so the downstream API contract is unchanged.
    let stock_symbol = String(form.get('stock_symbol') ?? '').trim()
    let rationale = String(form.get('rationale') ?? '').trim() || null
    if (!stock_symbol) {
      return Response.redirect(new URL('/trade-plans?error=missing', request.url), 303)
    }
    let resolved = await api.stockBySymbol(stock_symbol).catch(() => null)
    if (!resolved) {
      return Response.redirect(
        new URL('/trade-plans?error=bad-symbol', request.url),
        303,
      )
    }
    let stock_id = resolved.id
    let cookie = request.headers.get('cookie')
    let upstream = await api.createTradePlanRaw(cookie, { stock_id, rationale })
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/trade-plans?flash=created', request.url), 303)
  },
}

/// POST /trade-plans/:id/close — PATCH the plan to status=closed.
export const tradePlanClose: BuildAction<'POST', typeof routes.tradePlanClose> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/trade-plans?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.updateTradePlanRaw(cookie, id, { status: 'closed' })
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/trade-plans?flash=closed', request.url), 303)
  },
}

/// POST /trade-plans/:id/reopen — PATCH the plan back to status=active.
export const tradePlanReopen: BuildAction<'POST', typeof routes.tradePlanReopen> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/trade-plans?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.updateTradePlanRaw(cookie, id, { status: 'active' })
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/trade-plans?flash=reopened', request.url), 303)
  },
}

/// POST /trade-plans/:id/delete — hard delete plan and its levels.
export const tradePlanDelete: BuildAction<'POST', typeof routes.tradePlanDelete> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/trade-plans?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.deleteTradePlanRaw(cookie, id)
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/trade-plans?flash=deleted', request.url), 303)
  },
}

const KIND_VALUES = new Set(['buy', 'stop_loss', 'take_profit', 'trim'])

/// POST /trade-plans/:id/levels/new — add a new level to an existing plan.
export const tradePlanLevelCreate: BuildAction<
  'POST',
  typeof routes.tradePlanLevelCreate
> = {
  async handler({ request, params }) {
    let planId = Number(params.id)
    if (!Number.isFinite(planId)) {
      return Response.redirect(new URL('/trade-plans?error=bad-id', request.url), 303)
    }
    let form = await request.formData()
    let kind = String(form.get('kind') ?? '').trim()
    let price = String(form.get('price') ?? '').trim()
    // Empty fields go to the API as `null`, never the empty string — the
    // Rust side would otherwise try to parse "" as a Decimal and 400.
    let quantity = String(form.get('quantity') ?? '').trim() || null
    let fraction_pct = String(form.get('fraction_pct') ?? '').trim() || null
    let notes = String(form.get('notes') ?? '').trim() || null
    if (!kind || !price) {
      return Response.redirect(
        new URL('/trade-plans?error=missing-level', request.url),
        303,
      )
    }
    if (!KIND_VALUES.has(kind)) {
      return Response.redirect(
        new URL('/trade-plans?error=bad-kind', request.url),
        303,
      )
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.addTradePlanLevelRaw(cookie, planId, {
      kind,
      price,
      quantity,
      fraction_pct,
      notes,
    })
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(
      new URL('/trade-plans?flash=level-added', request.url),
      303,
    )
  },
}

/// POST /trade-plans/levels/:id/trigger — PATCH status=triggered.
export const tradePlanLevelTrigger: BuildAction<
  'POST',
  typeof routes.tradePlanLevelTrigger
> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/trade-plans?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.updateTradePlanLevelRaw(cookie, id, {
      status: 'triggered',
    })
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(
      new URL('/trade-plans?flash=level-triggered', request.url),
      303,
    )
  },
}

/// POST /trade-plans/levels/:id/cancel — PATCH status=cancelled.
export const tradePlanLevelCancel: BuildAction<
  'POST',
  typeof routes.tradePlanLevelCancel
> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/trade-plans?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.updateTradePlanLevelRaw(cookie, id, {
      status: 'cancelled',
    })
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(
      new URL('/trade-plans?flash=level-cancelled', request.url),
      303,
    )
  },
}

/// POST /trade-plans/levels/:id/reset — flip a triggered/cancelled level
/// back to status=active. Useful as an "undo" for a mis-click.
export const tradePlanLevelReset: BuildAction<
  'POST',
  typeof routes.tradePlanLevelReset
> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/trade-plans?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.updateTradePlanLevelRaw(cookie, id, {
      status: 'active',
    })
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(
      new URL('/trade-plans?flash=level-reset', request.url),
      303,
    )
  },
}

/// POST /trade-plans/levels/:id/delete — hard delete a level.
export const tradePlanLevelDelete: BuildAction<
  'POST',
  typeof routes.tradePlanLevelDelete
> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/trade-plans?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.deleteTradePlanLevelRaw(cookie, id)
    if (!upstream.ok) {
      return Response.redirect(new URL('/trade-plans?error=server', request.url), 303)
    }
    return Response.redirect(
      new URL('/trade-plans?flash=level-deleted', request.url),
      303,
    )
  },
}

interface Props {
  locale: string
  theme: Theme
  plans: TradePlan[]
  levelsByPlan: Map<number, TradePlanLevel[]>
  stocks: Map<number, Stock>
  /// Same data as `stocks` but in insertion order — used to populate the
  /// "Pick a stock" dropdown in the create form.
  allStocks: Stock[]
  error: string | null
  flash: string | null
}

function TradePlansPage() {
  return ({
    locale,
    theme,
    plans,
    levelsByPlan,
    stocks,
    allStocks,
    error,
    flash,
  }: Props) => {
    let m = messages(locale)
    let p = m.tradePlans
    let pageTitle = m.pages.tradePlans
    return (
      <Layout title={pageTitle.title} locale={locale} theme={theme}>
        <PageIntro>{pageTitle.subtitle}</PageIntro>
        {(error || flash) && (
          <div mix={css({ marginBottom: space[4] })}>
            <Banner error={error} flash={flash} locale={locale} />
          </div>
        )}

        <Card>
          <SectionTitle>{p.createSection}</SectionTitle>
          {allStocks.length === 0 ? (
            <div mix={css({ marginTop: space[3] })}>
              <EmptyState title={p.brokerMissing} />
            </div>
          ) : (
            <form
              method="post"
              action="/trade-plans/new"
              mix={css({
                display: 'grid',
                gridTemplateColumns: '240px 1fr',
                gap: space[3],
                marginTop: space[3],
                alignItems: 'start',
                '@media (max-width: 700px)': { gridTemplateColumns: '1fr' },
              })}
            >
              <label mix={css(labelWrap)}>
                <span mix={css(labelText)}>{p.stockLabel}</span>
                {/* Symbol input + datalist. Beats a <select> when
                    the catalog is in the thousands — the browser's
                    native autocomplete narrows as the user types,
                    and any ticker (even past the 500-row suggestion
                    pool) can be typed manually; the server resolves
                    via `?symbol=`. */}
                <input
                  type="text"
                  name="stock_symbol"
                  list="trade-plans-stock-symbols"
                  placeholder={p.stockPlaceholder}
                  autocomplete="off"
                  spellcheck={false}
                  required
                  mix={css({ ...fieldStyle, textTransform: 'uppercase' })}
                />
                <datalist id="trade-plans-stock-symbols">
                  {allStocks.map((s) => (
                    <option value={s.symbol} label={s.name ?? ''} />
                  ))}
                </datalist>
              </label>
              <label mix={css(labelWrap)}>
                <span mix={css(labelText)}>{p.rationaleLabel}</span>
                <textarea
                  name="rationale"
                  rows={2}
                  placeholder={p.rationalePlaceholder}
                  mix={css({ ...fieldStyle, resize: 'vertical' })}
                />
              </label>
              <div mix={css({ gridColumn: '1 / -1' })}>
                <button type="submit" mix={css(primaryButton)}>
                  {p.createSubmit}
                </button>
              </div>
            </form>
          )}
        </Card>

        {plans.length === 0 ? (
          <div mix={css({ marginTop: space[5] })}>
            <Card>
              <EmptyState title={p.emptyTitle} hint={p.emptyHint} />
            </Card>
          </div>
        ) : (
          <div
            mix={css({
              marginTop: space[5],
              display: 'flex',
              flexDirection: 'column',
              gap: space[4],
            })}
          >
            {plans.map((plan) => (
              <PlanCard
                plan={plan}
                levels={levelsByPlan.get(plan.id) ?? []}
                stock={stocks.get(plan.stock_id) ?? null}
                locale={locale}
              />
            ))}
          </div>
        )}
      </Layout>
    )
  }
}

function PlanCard() {
  return ({
    plan,
    levels,
    stock,
    locale,
  }: {
    plan: TradePlan
    levels: TradePlanLevel[]
    stock: Stock | null
    locale: string
  }) => {
    let all = messages(locale)
    let p = all.tradePlans
    let confirms = all.confirms
    let symbol = stock?.symbol ?? `#${plan.stock_id}`
    let displayName = stock?.name ? `${symbol} — ${stock.name}` : symbol
    let isActive = plan.status === 'active'
    let statusLabel = isActive ? p.statusActive : p.statusClosed
    let statusTone: BadgeTone = isActive ? 'success' : 'neutral'

    // Levels come pre-sorted from the API (sort_order asc, nulls last,
    // then price asc) — no client-side re-sort.
    let sortedLevels = levels

    return (
      <Card>
        <header
          mix={css({
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: space[3],
            flexWrap: 'wrap',
          })}
        >
          <div
            mix={css({
              display: 'inline-flex',
              alignItems: 'center',
              gap: space[3],
            })}
          >
            <StockBadge symbol={symbol} />
            <div>
              <div
                mix={css({
                  fontSize: font.md,
                  fontWeight: 700,
                  color: color.text,
                  fontFamily: font.mono,
                  letterSpacing: '-0.01em',
                })}
              >
                {displayName}
              </div>
              <div
                mix={css({
                  marginTop: space[1],
                  fontSize: font.xs,
                  color: color.textMuted,
                })}
              >
                #{plan.id} · <LocalTime value={plan.created_at} format="date" />
              </div>
            </div>
          </div>
          <div
            mix={css({
              display: 'inline-flex',
              alignItems: 'center',
              gap: space[2],
              flexWrap: 'wrap',
            })}
          >
            <Badge tone={statusTone}>{statusLabel}</Badge>
            {isActive ? (
              <form
                method="post"
                action={`/trade-plans/${plan.id}/close`}
                mix={css({ margin: 0 })}
              >
                <button
                  type="submit"
                  title={confirms.closeTradePlan(symbol)}
                  mix={css(secondaryButton)}
                >
                  {p.closePlanSubmit}
                </button>
              </form>
            ) : (
              <form
                method="post"
                action={`/trade-plans/${plan.id}/reopen`}
                mix={css({ margin: 0 })}
              >
                <button type="submit" mix={css(secondaryButton)}>
                  {p.reopenPlanSubmit}
                </button>
              </form>
            )}
            <form
              method="post"
              action={`/trade-plans/${plan.id}/delete`}
              mix={css({ margin: 0 })}
            >
              <button
                type="submit"
                title={confirms.deleteTradePlan(symbol)}
                mix={css(dangerButton)}
              >
                {p.deletePlanSubmit}
              </button>
            </form>
          </div>
        </header>

        {plan.rationale && (
          <p
            mix={css({
              margin: `${space[3]} 0 0`,
              fontSize: font.sm,
              color: color.textMuted,
              fontStyle: 'italic',
              lineHeight: 1.5,
            })}
          >
            {plan.rationale}
          </p>
        )}

        <div mix={css({ marginTop: space[4] })}>
          <SectionTitle>{p.levelHeadingFor(symbol)}</SectionTitle>
          {sortedLevels.length === 0 ? null : (
            <table
              mix={css({
                width: '100%',
                borderCollapse: 'collapse',
                fontSize: font.base,
                marginTop: space[2],
              })}
            >
              <thead>
                <tr>
                  <Th>{p.columnKind}</Th>
                  <Th>{p.columnPrice}</Th>
                  <Th>{p.columnSize}</Th>
                  <Th>{p.columnStatus}</Th>
                  <Th>{p.columnNotes}</Th>
                  <Th>{''}</Th>
                </tr>
              </thead>
              <tbody>
                {sortedLevels.map((lv) => (
                  <LevelRow level={lv} locale={locale} />
                ))}
              </tbody>
            </table>
          )}
        </div>

        <div mix={css({ marginTop: space[4] })}>
          <SectionTitle>{p.addLevelSection}</SectionTitle>
          <form
            method="post"
            action={`/trade-plans/${plan.id}/levels/new`}
            mix={css({
              display: 'grid',
              gridTemplateColumns:
                '160px 140px 140px 140px 1fr auto',
              gap: space[3],
              alignItems: 'end',
              '@media (max-width: 1100px)': {
                gridTemplateColumns: '1fr 1fr',
              },
              '@media (max-width: 600px)': {
                gridTemplateColumns: '1fr',
              },
            })}
          >
            <label mix={css(labelWrap)}>
              <span mix={css(labelText)}>{p.kindLabel}</span>
              <select name="kind" required mix={css(fieldStyle)}>
                <option value="buy">{p.kindBuy}</option>
                <option value="stop_loss">{p.kindStopLoss}</option>
                <option value="take_profit">{p.kindTakeProfit}</option>
                <option value="trim">{p.kindTrim}</option>
              </select>
            </label>
            <label mix={css(labelWrap)}>
              <span mix={css(labelText)}>{p.priceLabel}</span>
              <input
                name="price"
                type="text"
                inputMode="decimal"
                required
                autoComplete="off"
                mix={css({ ...fieldStyle, fontVariantNumeric: 'tabular-nums' })}
              />
            </label>
            <label mix={css(labelWrap)}>
              <span mix={css(labelText)}>{p.quantityLabel}</span>
              <input
                name="quantity"
                type="text"
                inputMode="decimal"
                autoComplete="off"
                mix={css({ ...fieldStyle, fontVariantNumeric: 'tabular-nums' })}
              />
            </label>
            <label mix={css(labelWrap)}>
              <span mix={css(labelText)}>{p.fractionPctLabel}</span>
              <input
                name="fraction_pct"
                type="text"
                inputMode="decimal"
                autoComplete="off"
                mix={css({ ...fieldStyle, fontVariantNumeric: 'tabular-nums' })}
              />
            </label>
            <label mix={css(labelWrap)}>
              <span mix={css(labelText)}>{p.notesLabel}</span>
              <input
                name="notes"
                type="text"
                autoComplete="off"
                mix={css(fieldStyle)}
              />
            </label>
            <button type="submit" mix={css(primaryButton)}>
              {p.addLevelSubmit}
            </button>
          </form>
        </div>
      </Card>
    )
  }
}

function LevelRow() {
  return ({ level, locale }: { level: TradePlanLevel; locale: string }) => {
    let all = messages(locale)
    let p = all.tradePlans
    let confirms = all.confirms
    let kindLabel = kindLabelFor(level.kind, p)
    let kindTone = kindToneFor(level.kind)
    let statusTone = statusToneFor(level.status)
    let statusLabel = statusLabelFor(level.status, p)

    let sizeCell: RemixNode = (
      <span mix={css({ color: color.textDim })}>—</span>
    )
    if (level.quantity) sizeCell = p.sizeQuantity(level.quantity)
    else if (level.fraction_pct) sizeCell = p.sizePct(level.fraction_pct)

    let isActive = level.status === 'active'

    return (
      <tr mix={css({ borderTop: `1px solid ${color.borderSoft}` })}>
        <Td>
          <Badge tone={kindTone}>{kindLabel}</Badge>
        </Td>
        <Td>
          <span mix={css({ fontFamily: font.mono, fontWeight: 600 })}>
            {fmtMoney(level.price)}
          </span>
        </Td>
        <Td>{sizeCell}</Td>
        <Td>
          <Badge tone={statusTone}>{statusLabel}</Badge>
        </Td>
        <Td>
          {level.notes ? (
            level.notes
          ) : (
            <span mix={css({ color: color.textDim })}>—</span>
          )}
        </Td>
        <Td>
          <div
            mix={css({
              display: 'inline-flex',
              gap: space[2],
              flexWrap: 'wrap',
            })}
          >
            {isActive ? (
              <>
                <form
                  method="post"
                  action={`/trade-plans/levels/${level.id}/trigger`}
                  mix={css({ margin: 0 })}
                >
                  <button type="submit" mix={css(secondaryButton)}>
                    {p.triggerSubmit}
                  </button>
                </form>
                <form
                  method="post"
                  action={`/trade-plans/levels/${level.id}/cancel`}
                  mix={css({ margin: 0 })}
                >
                  <button type="submit" mix={css(secondaryButton)}>
                    {p.cancelSubmit}
                  </button>
                </form>
              </>
            ) : (
              <form
                method="post"
                action={`/trade-plans/levels/${level.id}/reset`}
                mix={css({ margin: 0 })}
              >
                <button type="submit" mix={css(secondaryButton)}>
                  {p.resetSubmit}
                </button>
              </form>
            )}
            <form
              method="post"
              action={`/trade-plans/levels/${level.id}/delete`}
              mix={css({ margin: 0 })}
            >
              <button
                type="submit"
                title={confirms.deleteTradePlanLevel(kindLabel, fmtMoney(level.price))}
                mix={css(dangerButton)}
              >
                {p.deleteLevelSubmit}
              </button>
            </form>
          </div>
        </Td>
      </tr>
    )
  }
}

function kindLabelFor(kind: string, p: Messages['tradePlans']): string {
  switch (kind) {
    case 'buy':
      return p.kindBuy
    case 'stop_loss':
      return p.kindStopLoss
    case 'take_profit':
      return p.kindTakeProfit
    case 'trim':
      return p.kindTrim
    default:
      return kind
  }
}

function kindToneFor(kind: string): BadgeTone {
  switch (kind) {
    case 'buy':
      return 'brand'
    case 'stop_loss':
      return 'danger'
    case 'take_profit':
      return 'success'
    case 'trim':
      return 'warn'
    default:
      return 'neutral'
  }
}

function statusLabelFor(status: string, p: Messages['tradePlans']): string {
  switch (status) {
    case 'active':
      return p.levelStatusActive
    case 'triggered':
      return p.levelStatusTriggered
    case 'cancelled':
      return p.levelStatusCancelled
    default:
      return status
  }
}

function statusToneFor(status: string): BadgeTone {
  switch (status) {
    case 'active':
      return 'info'
    case 'triggered':
      return 'success'
    case 'cancelled':
      return 'neutral'
    default:
      return 'neutral'
  }
}

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
  let p = messages(locale).tradePlans
  if (error) {
    let table: Record<string, string> = {
      missing: p.errMissingCreate,
      'missing-level': p.errMissingLevel,
      'bad-kind': p.errBadKind,
      'bad-id': p.errBadId,
      server: p.errServer,
    }
    return { tone: 'error' as const, message: table[error] ?? p.errServer }
  }
  if (flash) {
    let table: Record<string, string> = {
      created: p.flashCreated,
      closed: p.flashClosed,
      reopened: p.flashReopened,
      deleted: p.flashDeleted,
      'level-added': p.flashLevelAdded,
      'level-triggered': p.flashLevelTriggered,
      'level-cancelled': p.flashLevelCancelled,
      'level-reset': p.flashLevelReset,
      'level-deleted': p.flashLevelDeleted,
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
