import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Account, type Broker } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
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
  type Theme,
} from '../ui/layout.tsx'
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

interface Props {
  locale: string
  theme: Theme
  accounts: Account[]
  brokers: Broker[]
  brokerMap: Record<number, Broker>
  error: string | null
  flash: string | null
}

export const accounts: BuildAction<'GET', typeof routes.accounts> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let flash = url.searchParams.get('flash')
    let error = url.searchParams.get('error')
    let [accts, brks] = await Promise.all([
      api.accounts().catch(() => [] as Account[]),
      api.brokers().catch(() => [] as Broker[]),
    ])
    let brokerMap: Record<number, Broker> = {}
    for (let b of brks) brokerMap[b.id] = b
    // Order set server-side: created_at desc, id desc.
    return render(
      <AccountsPage
        locale={locale}
        theme={theme}
        accounts={accts}
        brokers={brks}
        brokerMap={brokerMap}
        error={error}
        flash={flash}
      />,
      request,
      { locale, theme },
    )
  },
}

export const accountCreate: BuildAction<'POST', typeof routes.accountCreate> = {
  async handler({ request }) {
    let form = await request.formData()
    let broker_id = Number(form.get('broker_id') ?? 0)
    let name = String(form.get('name') ?? '').trim()
    let account_number = String(form.get('account_number') ?? '').trim()
    let base_currency = String(form.get('base_currency') ?? '').trim()
    if (!broker_id || !name || !base_currency) {
      return Response.redirect(new URL('/accounts?error=missing', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.createAccountRaw(cookie, {
      broker_id,
      name,
      account_number: account_number || null,
      base_currency,
    })
    if (!upstream.ok) {
      return Response.redirect(new URL('/accounts?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/accounts?flash=created', request.url), 303)
  },
}

/// POST /accounts/:id/cash — pin the account's cash balance.
///
/// Cash is derived from the ledger, but a ledger that starts mid-life
/// has buys with no matching deposits and rolls up negative. This is the
/// correction: "my broker says I hold X as of T". Only flows after T are
/// applied on top, so past trades aren't double-counted.
///
/// Day-to-day deposits and withdrawals don't belong here — record those
/// as DEPOSIT / WITHDRAWAL transactions and this balance keeps up on its
/// own.
export const accountCash: BuildAction<'POST', typeof routes.accountCash> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/accounts?error=server', request.url), 303)
    }
    let form = await request.formData()
    let balance = String(form.get('cash_balance') ?? '').trim()
    let asOf = String(form.get('cash_as_of') ?? '').trim()
    let clear = String(form.get('clear') ?? '') === '1'

    let cookie = request.headers.get('cookie')
    if (clear) {
      // Drop the anchor: balance back to 0 and no as-of, which puts the
      // account back on pure-ledger cash.
      let res = await api.updateAccountRaw(cookie, id, {
        cash_balance: '0',
        cash_as_of: null,
      })
      return Response.redirect(
        new URL(res.ok ? '/accounts?flash=cash-cleared' : '/accounts?error=server', request.url),
        303,
      )
    }
    if (!balance || !asOf) {
      return Response.redirect(new URL('/accounts?error=cash-missing', request.url), 303)
    }
    // `datetime-local` gives `2026-08-07T14:30` with no zone; same UTC
    // approximation the orders and transactions forms make.
    let asOfIso = `${asOf}${asOf.length === 16 ? ':00' : ''}Z`
    let upstream = await api.updateAccountRaw(cookie, id, {
      cash_balance: balance,
      cash_as_of: asOfIso,
    })
    if (!upstream.ok) {
      return Response.redirect(new URL('/accounts?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/accounts?flash=cash-set', request.url), 303)
  },
}

export const accountDelete: BuildAction<'POST', typeof routes.accountDelete> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/accounts?error=server', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.deleteAccountRaw(cookie, id)
    if (!upstream.ok) {
      let code = upstream.status === 409 ? 'in-use' : 'server'
      return Response.redirect(new URL(`/accounts?error=${code}`, request.url), 303)
    }
    return Response.redirect(new URL('/accounts?flash=deleted', request.url), 303)
  },
}

/// Inline cash-anchor editor for one account row.
///
/// Reads as a value with an edit affordance rather than a permanently
/// open form: the anchor is set once from a statement and then left
/// alone, so a row of always-visible inputs would imply it needs
/// constant attention. `<details>` keeps it one click away with no JS.
function CashAnchorCell() {
  return ({ account, locale }: { account: Account; locale: string }) => {
    let p = messages(locale).accounts
    let anchored = account.cash_as_of != null
    return (
      <details mix={css({ '&[open] summary': { marginBottom: space[2] } })}>
        <summary
          mix={css({
            cursor: 'pointer',
            listStyle: 'none',
            fontFamily: font.mono,
            fontVariantNumeric: 'tabular-nums',
            color: anchored ? color.text : color.textDim,
            '&::-webkit-details-marker': { display: 'none' },
            '&:hover': { color: color.brandHover },
          })}
        >
          {anchored ? (
            <>
              {account.cash_balance} {account.base_currency}
              <span
                mix={css({
                  display: 'block',
                  fontSize: font.xs,
                  color: color.textDim,
                  fontFamily: font.sans,
                })}
              >
                <LocalTime value={account.cash_as_of!} format="date" />
              </span>
            </>
          ) : (
            <span mix={css({ fontFamily: font.sans, fontSize: font.sm })}>
              {p.cashNotSet}
            </span>
          )}
        </summary>
        <form
          method="post"
          action={`/accounts/${account.id}/cash`}
          mix={css({ display: 'flex', flexDirection: 'column', gap: space[2] })}
        >
          <label mix={css(labelWrap)}>
            <span mix={css(labelText)}>{p.cashBalanceLabel}</span>
            <input
              type="text"
              name="cash_balance"
              inputmode="decimal"
              value={anchored ? account.cash_balance : undefined}
              placeholder="0.00"
              required
              mix={css(fieldStyle)}
            />
          </label>
          <label mix={css(labelWrap)}>
            <span mix={css(labelText)}>{p.cashAsOfLabel}</span>
            <input type="datetime-local" name="cash_as_of" required mix={css(fieldStyle)} />
          </label>
          <span mix={css({ fontSize: font.xs, color: color.textDim, lineHeight: 1.5 })}>
            {p.cashHint}
          </span>
          <div mix={css({ display: 'flex', gap: space[2] })}>
            <button type="submit" mix={css(primaryButton)}>
              {p.cashSubmit}
            </button>
            {anchored && (
              <button
                type="submit"
                name="clear"
                value="1"
                formnovalidate
                title={p.cashClearConfirm}
                mix={css(secondaryButton)}
              >
                {p.cashClear}
              </button>
            )}
          </div>
        </form>
      </details>
    )
  }
}

function AccountsPage() {
  return ({ locale, theme, accounts, brokers, brokerMap, error, flash }: Props) => {
    let m = messages(locale)
    let p = m.accounts
    let pageTitle = m.pages.accounts
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
          {brokers.length === 0 ? (
            <div mix={css({ marginTop: space[3] })}>
              <EmptyState title={p.errBrokerMissing} />
            </div>
          ) : (
            <form
              method="post"
              action="/accounts/new"
              mix={css({
                display: 'grid',
                gridTemplateColumns: '1fr 1fr',
                gap: space[3],
                marginTop: space[3],
                '@media (max-width: 700px)': { gridTemplateColumns: '1fr' },
              })}
            >
              <label mix={css(labelWrap)}>
                <span mix={css(labelText)}>{p.brokerLabel}</span>
                <select name="broker_id" required mix={css(fieldStyle)}>
                  {brokers.map((b) => (
                    <option value={b.id}>
                      {b.name} ({b.code})
                    </option>
                  ))}
                </select>
              </label>
              <label mix={css(labelWrap)}>
                <span mix={css(labelText)}>{p.namePlaceholder}</span>
                <input
                  name="name"
                  type="text"
                  required
                  autoComplete="off"
                  mix={css(fieldStyle)}
                />
              </label>
              <label mix={css(labelWrap)}>
                <span mix={css(labelText)}>{p.accountNumberPlaceholder}</span>
                <input
                  name="account_number"
                  type="text"
                  autoComplete="off"
                  mix={css(fieldStyle)}
                />
              </label>
              <label mix={css(labelWrap)}>
                <span mix={css(labelText)}>{p.currencyPlaceholder}</span>
                <input
                  name="base_currency"
                  type="text"
                  required
                  autoComplete="off"
                  placeholder="USD"
                  mix={css(fieldStyle)}
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

        <div mix={css({ marginTop: space[5] })}>
          <Card>
            <SectionTitle>{p.listSection}</SectionTitle>
            {accounts.length === 0 ? (
              <div mix={css({ marginTop: space[3] })}>
                <EmptyState title={p.emptyTitle} hint={p.emptyHint} />
              </div>
            ) : (
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
                    <Th>{p.columnName}</Th>
                    <Th>{p.columnBroker}</Th>
                    <Th>{p.columnNumber}</Th>
                    <Th>{p.columnCurrency}</Th>
                    <Th>{p.columnCash}</Th>
                    <Th>{p.columnCreated}</Th>
                    <Th>{''}</Th>
                  </tr>
                </thead>
                <tbody>
                  {accounts.map((a) => {
                    let b = brokerMap[a.broker_id]
                    return (
                      <tr mix={css({ borderTop: `1px solid ${color.borderSoft}` })}>
                        <Td>{a.name}</Td>
                        <Td>{b ? `${b.name} (${b.code})` : `#${a.broker_id}`}</Td>
                        <Td>{a.account_number ?? '—'}</Td>
                        <Td>{a.base_currency}</Td>
                        <Td>
                          <CashAnchorCell account={a} locale={locale} />
                        </Td>
                        <Td>
                          <LocalTime value={a.created_at} format="date" />
                        </Td>
                        <Td>
                          <form
                            method="post"
                            action={`/accounts/${a.id}/delete`}
                            mix={css({ margin: 0 })}
                          >
                            <button
                              type="submit"
                              title={m.confirms.deleteAccount(a.name)}
                              mix={css(dangerButton)}
                            >
                              {p.deleteSubmit}
                            </button>
                          </form>
                        </Td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            )}
          </Card>
        </div>
      </Layout>
    )
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
    let p = messages(locale).accounts
    // Flat lookup rather than another ternary rung — the chain was
    // already four deep and cash added three more cases.
    let errors: Record<string, string> = {
      missing: p.errMissingCreate,
      'in-use': p.errInUse,
      'cash-missing': p.errCashMissing,
    }
    let flashes: Record<string, string> = {
      created: p.flashCreated,
      deleted: p.flashDeleted,
      'cash-set': p.flashCashSet,
      'cash-cleared': p.flashCashCleared,
    }
    let { tone, message } = error
      ? { tone: 'error' as const, message: errors[error] ?? p.errServer }
      : { tone: 'success' as const, message: flash ? (flashes[flash] ?? '') : '' }
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

function Th() {
  return ({ children }: { children: string }) => (
    <th
      mix={css({
        textAlign: 'left',
        padding: `${space[3]} ${space[4]}`,
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
  return ({ children }: { children: import('remix/ui').RemixNode }) => (
    <td
      mix={css({
        padding: `${space[3]} ${space[4]}`,
        color: color.text,
        fontVariantNumeric: 'tabular-nums',
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

const secondaryButton = {
  padding: `${space[2]} ${space[3]}`,
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
