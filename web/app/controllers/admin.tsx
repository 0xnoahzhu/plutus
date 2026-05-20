import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Document } from '../ui/document.tsx'
import {
  BrandMark,
  Card,
  color,
  EmptyState,
  font,
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  type Theme,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

interface UserRow {
  id: number
  username: string
  password_reset_required: boolean
  allowed_countries: string[]
  created_at: string
  updated_at: string
}

/// Country codes the API accepts. Kept in sync with
/// `SUPPORTED_COUNTRIES` on the Rust side and `COUNTRY_TO_MARKETS` in
/// `ui/layout.tsx`. The order here is the order the checkboxes render
/// in.
const SUPPORTED_COUNTRIES = ['US', 'HK', 'CN'] as const
type CountryCode = (typeof SUPPORTED_COUNTRIES)[number]

/// Parse the multi-checkbox `country` field out of a submit. Returns
/// only known codes — anything else is silently dropped (the API would
/// 400 anyway).
function parseCountriesFromForm(form: FormData): string[] {
  let raw = form.getAll('country').map((v) => String(v).trim().toUpperCase())
  return SUPPORTED_COUNTRIES.filter((c) => raw.includes(c))
}

/// GET /admin — list all users. Reachable only to the env-configured admin.
/// Regular sessions hit a 403 from the API and we punt them to /.
export const admin: BuildAction<'GET', typeof routes.admin> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let cookie = request.headers.get('cookie')
    let error = url.searchParams.get('error')
    let flash = url.searchParams.get('flash')

    let users: UserRow[] = []
    try {
      users = await api.adminListUsers(cookie)
    } catch {
      // Not authenticated as admin → send away. Bare cookie / regular user.
      return Response.redirect(new URL('/login', request.url), 303)
    }

    return render(
      <AdminPage
        locale={locale}
        theme={theme}
        users={users}
        error={error}
        flash={flash}
      />,
      request,
      { locale, theme },
    )
  },
}

/// POST /admin/users/new — create a new user account.
export const adminUserCreate: BuildAction<'POST', typeof routes.adminUserCreate> = {
  async handler({ request }) {
    let form = await request.formData()
    let username = String(form.get('username') ?? '').trim()
    let password = String(form.get('password') ?? '')
    if (!username || !password) {
      return Response.redirect(new URL('/admin?error=missing-create', request.url), 303)
    }
    let countries = parseCountriesFromForm(form)
    if (countries.length === 0) {
      return Response.redirect(
        new URL('/admin?error=missing-countries', request.url),
        303,
      )
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminCreateUserRaw(cookie, {
      username,
      password,
      allowed_countries: countries,
    })
    if (!upstream.ok) {
      let code = upstream.status === 409 ? 'taken' : upstream.status === 403 ? 'forbidden' : 'server'
      return Response.redirect(new URL(`/admin?error=${code}`, request.url), 303)
    }
    return Response.redirect(
      new URL(`/admin?flash=created&user=${encodeURIComponent(username)}`, request.url),
      303,
    )
  },
}

/// POST /admin/users/:id/countries — replace a user's country allowlist.
/// Empty selection is rejected here so the user always has at least one
/// tab to land on.
export const adminUserCountries: BuildAction<
  'POST',
  typeof routes.adminUserCountries
> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/admin?error=bad-id', request.url), 303)
    }
    let form = await request.formData()
    let countries = parseCountriesFromForm(form)
    if (countries.length === 0) {
      return Response.redirect(
        new URL('/admin?error=missing-countries', request.url),
        303,
      )
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminUpdateUserCountriesRaw(cookie, id, countries)
    if (!upstream.ok) {
      let code =
        upstream.status === 403
          ? 'forbidden'
          : upstream.status === 404
            ? 'not-found'
            : 'server'
      return Response.redirect(new URL(`/admin?error=${code}`, request.url), 303)
    }
    return Response.redirect(new URL(`/admin?flash=countries-updated`, request.url), 303)
  },
}

/// POST /admin/users/:id/reset — admin sets a fresh temp password and flips
/// `password_reset_required=true` on the user row.
export const adminUserReset: BuildAction<'POST', typeof routes.adminUserReset> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/admin?error=bad-id', request.url), 303)
    }
    let form = await request.formData()
    let password = String(form.get('password') ?? '')
    if (!password) {
      return Response.redirect(new URL('/admin?error=missing-reset', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminResetUserPasswordRaw(cookie, id, password)
    if (!upstream.ok) {
      let code = upstream.status === 403 ? 'forbidden' : upstream.status === 404 ? 'not-found' : 'server'
      return Response.redirect(new URL(`/admin?error=${code}`, request.url), 303)
    }
    return Response.redirect(new URL(`/admin?flash=reset`, request.url), 303)
  },
}

/// POST /admin/users/:id/delete — remove a user account. Hard delete; their
/// rows in per-user tables become orphans (the unique indexes on
/// (user_id, …) keep them isolated).
export const adminUserDelete: BuildAction<'POST', typeof routes.adminUserDelete> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/admin?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminDeleteUserRaw(cookie, id)
    if (!upstream.ok) {
      let code = upstream.status === 403 ? 'forbidden' : upstream.status === 404 ? 'not-found' : 'server'
      return Response.redirect(new URL(`/admin?error=${code}`, request.url), 303)
    }
    return Response.redirect(new URL(`/admin?flash=deleted`, request.url), 303)
  },
}

interface Props {
  locale: string
  theme: Theme
  users: UserRow[]
  error: string | null
  flash: string | null
}

/// Standalone admin shell — no regular sidebar. Admin doesn't have per-user
/// data of its own, so the data nav would just produce 403s. Keep the page
/// simple: brand + a single "Users" section.
function AdminPage() {
  return ({ locale, theme, users, error, flash }: Props) => {
    let m = messages(locale)
    return (
    <Document title={`${m.admin.title} · Plutus`} lang={locale} theme={theme}>
      <div
        mix={css({
          minHeight: '100vh',
          background: color.bg,
          padding: `${space[6]} ${space[6]}`,
        })}
      >
        <div
          mix={css({
            maxWidth: '780px',
            margin: '0 auto',
          })}
        >
          <header
            mix={css({
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: space[6],
              flexWrap: 'wrap',
              gap: space[3],
            })}
          >
            <BrandMark size={32} />
            <form method="post" action="/logout" mix={css({ margin: 0 })}>
              <button
                type="submit"
                mix={css({
                  padding: `${space[2]} ${space[3]}`,
                  background: 'transparent',
                  border: `1px solid ${color.border}`,
                  borderRadius: radius.md,
                  color: color.textMuted,
                  fontSize: font.sm,
                  fontWeight: 500,
                  fontFamily: font.sans,
                  cursor: 'pointer',
                  '&:hover': {
                    background: color.hover,
                    color: color.danger,
                  },
                })}
              >
                {m.nav.signOut}
              </button>
            </form>
          </header>

          <h1
            mix={css({
              margin: `0 0 ${space[1]}`,
              fontSize: font.xxl,
              fontWeight: 700,
              color: color.text,
              letterSpacing: '-0.01em',
            })}
          >
            {m.admin.title}
          </h1>
          <p
            mix={css({
              margin: `0 0 ${space[4]}`,
              fontSize: font.sm,
              color: color.textMuted,
            })}
          >
            {m.admin.subtitle}
          </p>

          <AdminTabs locale={locale} active="users" />

          {(error || flash) && (
            <div mix={css({ marginBottom: space[4] })}>
              <Banner error={error} flash={flash} locale={locale} />
            </div>
          )}

          <Card>
            <SectionTitle>{m.admin.createSection}</SectionTitle>
            <form
              method="post"
              action="/admin/users/new"
              mix={css({
                display: 'flex',
                gap: space[3],
                flexWrap: 'wrap',
                alignItems: 'center',
                marginTop: space[3],
              })}
            >
              <input
                name="username"
                type="text"
                placeholder={m.admin.createUsername}
                required
                autoComplete="off"
                mix={css({ ...fieldStyle, flex: '1 1 180px' })}
              />
              <input
                name="password"
                type="text"
                placeholder={m.admin.createPassword}
                required
                autoComplete="off"
                mix={css({ ...fieldStyle, flex: '1 1 220px' })}
              />
              <CountryCheckboxes
                locale={locale}
                selected={SUPPORTED_COUNTRIES}
                label={m.admin.countriesLabel}
              />
              <button
                type="submit"
                mix={css({
                  padding: `${space[3]} ${space[4]}`,
                  background: color.brand,
                  color: '#fff',
                  border: 'none',
                  borderRadius: radius.md,
                  fontSize: font.base,
                  fontWeight: 600,
                  cursor: 'pointer',
                  '&:hover': { background: color.brandHover },
                })}
              >
                {m.admin.createSubmit}
              </button>
            </form>
          </Card>

          <div mix={css({ marginTop: space[5] })}>
            <Card>
              <SectionTitle>{m.admin.usersSection}</SectionTitle>
              {users.length === 0 ? (
                <div mix={css({ marginTop: space[3] })}>
                  <EmptyState
                    title={m.admin.emptyTitle}
                    hint={m.admin.emptyHint}
                  />
                </div>
              ) : (
                <ul
                  mix={css({
                    listStyle: 'none',
                    padding: 0,
                    margin: `${space[3]} 0 0`,
                    display: 'flex',
                    flexDirection: 'column',
                    gap: space[3],
                  })}
                >
                  {users.map((u) => (
                    <UserRow user={u} locale={locale} />
                  ))}
                </ul>
              )}
            </Card>
          </div>
        </div>
      </div>
    </Document>
    )
  }
}

function UserRow() {
  return ({ user, locale }: { user: UserRow; locale: string }) => {
    let all = messages(locale)
    let m = all.admin
    let confirms = all.confirms
    return (
    <li
      mix={css({
        border: `1px solid ${color.border}`,
        borderRadius: radius.md,
        padding: space[3],
        background: color.bg,
      })}
    >
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: space[3],
        })}
      >
        <div>
          <div
            mix={css({
              fontSize: font.base,
              fontWeight: 600,
              color: color.text,
            })}
          >
            {user.username}
          </div>
          <div
            mix={css({
              fontSize: font.xs,
              color: color.textMuted,
              marginTop: space[1],
              display: 'flex',
              alignItems: 'center',
              gap: space[2],
              flexWrap: 'wrap',
            })}
          >
            <span>
              #{user.id} · created {user.created_at.slice(0, 10)}
            </span>
            {user.password_reset_required && (
              <span
                mix={css({
                  padding: `2px ${space[2]}`,
                  background: color.dangerSoft,
                  color: color.dangerText,
                  borderRadius: radius.sm,
                  fontWeight: 600,
                })}
              >
                {m.resetBadge}
              </span>
            )}
            {user.allowed_countries.map((c) => (
              <span
                mix={css({
                  padding: `2px ${space[2]}`,
                  background: color.brandSoft,
                  color: color.brand,
                  borderRadius: radius.sm,
                  fontWeight: 600,
                  letterSpacing: '0.04em',
                })}
              >
                {c}
              </span>
            ))}
          </div>
        </div>
      </div>
      <div
        mix={css({
          marginTop: space[3],
          display: 'flex',
          gap: space[3],
          flexWrap: 'wrap',
        })}
      >
        <form
          method="post"
          action={`/admin/users/${user.id}/reset`}
          mix={css({
            display: 'flex',
            gap: space[2],
            flex: '1 1 280px',
            margin: 0,
          })}
        >
          <input
            name="password"
            type="text"
            placeholder={m.resetPlaceholder}
            required
            autoComplete="off"
            mix={css({ ...fieldStyle, flex: '1 1 auto' })}
          />
          <button
            type="submit"
            title={confirms.resetUserPassword(user.username)}
            mix={css(secondaryButtonStyle)}
          >
            {m.resetSubmit}
          </button>
        </form>
        <form
          method="post"
          action={`/admin/users/${user.id}/delete`}
          mix={css({ margin: 0 })}
        >
          <button
            type="submit"
            title={confirms.deleteUser(user.username)}
            mix={css(dangerButtonStyle)}
          >
            {m.deleteSubmit}
          </button>
        </form>
      </div>

      {/* Country scope. Self-contained form: checkboxes + Save. Submitting
          with no boxes checked is rejected at the controller boundary so
          the row always renders some country label. */}
      <form
        method="post"
        action={`/admin/users/${user.id}/countries`}
        mix={css({
          marginTop: space[3],
          display: 'flex',
          gap: space[3],
          alignItems: 'center',
          flexWrap: 'wrap',
          margin: `${space[3]} 0 0`,
        })}
      >
        <CountryCheckboxes
          locale={locale}
          selected={user.allowed_countries}
          label={m.countriesLabel}
        />
        <button
          type="submit"
          title={confirms.updateUserCountries(user.username)}
          mix={css(secondaryButtonStyle)}
        >
          {m.updateCountriesSubmit}
        </button>
      </form>
    </li>
    )
  }
}

/// Three-checkbox group bound to `name="country"` so the multi-select
/// arrives as a single multi-valued form field. Used in both the create
/// form and the per-user edit form so the layout is identical.
function CountryCheckboxes() {
  return ({
    locale,
    selected,
    label,
  }: {
    locale: string
    selected: readonly string[]
    label: string
  }) => {
    let m = messages(locale).admin
    let labelFor = (c: CountryCode) =>
      c === 'US' ? m.countryUS : c === 'HK' ? m.countryHK : m.countryCN
    return (
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          gap: space[3],
          flexWrap: 'wrap',
        })}
      >
        <span
          mix={css({
            fontSize: font.xs,
            fontWeight: 600,
            color: color.textMuted,
            textTransform: 'uppercase',
            letterSpacing: '0.06em',
          })}
        >
          {label}
        </span>
        {SUPPORTED_COUNTRIES.map((c) => (
          <label
            mix={css({
              display: 'inline-flex',
              alignItems: 'center',
              gap: space[1],
              fontSize: font.sm,
              color: color.text,
              cursor: 'pointer',
            })}
          >
            <input
              type="checkbox"
              name="country"
              value={c}
              checked={selected.includes(c)}
            />
            {labelFor(c)}
          </label>
        ))}
      </div>
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
  let m = messages(locale).admin
  if (error) {
    let table: Record<string, string> = {
      'missing-create': m.errMissingCreate,
      'missing-reset': m.errMissingReset,
      'missing-countries': m.errMissingCountries,
      'bad-id': m.errBadId,
      taken: m.errTaken,
      forbidden: m.errForbidden,
      'not-found': m.errNotFound,
      server: m.errServer,
    }
    return { tone: 'error' as const, message: table[error] ?? m.errServer }
  }
  if (flash) {
    let table: Record<string, string> = {
      created: m.flashCreated,
      reset: m.flashReset,
      deleted: m.flashDeleted,
      'countries-updated': m.flashCountriesUpdated,
    }
    return { tone: 'success' as const, message: table[flash] ?? '' }
  }
  return { tone: 'success' as const, message: '' }
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

const secondaryButtonStyle = {
  padding: `${space[2]} ${space[3]}`,
  background: 'transparent',
  border: `1px solid ${color.border}`,
  borderRadius: radius.md,
  color: color.text,
  fontSize: font.sm,
  fontWeight: 500,
  fontFamily: font.sans,
  cursor: 'pointer',
  '&:hover': { background: color.hover },
}

const dangerButtonStyle = {
  padding: `${space[2]} ${space[3]}`,
  background: 'transparent',
  border: `1px solid ${color.border}`,
  borderRadius: radius.md,
  color: color.danger,
  fontSize: font.sm,
  fontWeight: 500,
  fontFamily: font.sans,
  cursor: 'pointer',
  '&:hover': { background: color.dangerSoft, borderColor: color.danger },
}

/// Tab nav for the admin shell. Reused by `/admin` (Users) and
/// `/admin/brokers`. Style copies the soft "white pill on inset gray
/// track" pattern that the rest of the chrome uses.
export function AdminTabs() {
  return ({
    locale,
    active,
  }: {
    locale: string
    active: 'users' | 'brokers' | 'tokens'
  }) => {
    let m = messages(locale).admin
    return (
      <div
        mix={css({
          display: 'inline-flex',
          gap: space[1],
          padding: '3px',
          background: color.bg,
          border: `1px solid ${color.border}`,
          borderRadius: radius.pill,
          marginBottom: space[5],
        })}
      >
        <AdminTab href="/admin" label={m.tabUsers} active={active === 'users'} />
        <AdminTab href="/admin/brokers" label={m.tabBrokers} active={active === 'brokers'} />
        <AdminTab href="/admin/tokens" label={m.tabTokens} active={active === 'tokens'} />
      </div>
    )
  }
}

function AdminTab() {
  return ({ href, label, active }: { href: string; label: string; active: boolean }) => (
    <a
      href={href}
      mix={css({
        display: 'inline-flex',
        alignItems: 'center',
        padding: `${space[1]} ${space[3]}`,
        fontSize: font.sm,
        fontWeight: 600,
        borderRadius: radius.pill,
        textDecoration: 'none',
        color: active ? color.text : color.textMuted,
        background: active ? color.surface : 'transparent',
        transition: 'background 120ms ease, color 120ms ease',
        '&:hover': active ? undefined : { color: color.text },
      })}
    >
      {label}
    </a>
  )
}
