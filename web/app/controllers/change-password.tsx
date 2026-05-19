import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Document } from '../ui/document.tsx'
import {
  BrandMark,
  color,
  font,
  radius,
  resolveLocale,
  resolveTheme,
  space,
  type Theme,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

const showForm: BuildAction<'GET', typeof routes.changePassword.index> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let next = url.searchParams.get('next') ?? '/'
    let error = url.searchParams.get('error')
    let forced = url.searchParams.get('forced') === '1'
    return render(
      <ChangePasswordPage
        locale={locale}
        theme={theme}
        next={next}
        error={error}
        forced={forced}
      />,
      request,
      { locale, theme },
    )
  },
}

const submitForm: BuildAction<'POST', typeof routes.changePassword.action> = {
  async handler({ request }) {
    let form = await request.formData()
    let current = String(form.get('current_password') ?? '')
    let next_password = String(form.get('new_password') ?? '')
    let confirm = String(form.get('new_password_confirm') ?? '')
    let next = String(form.get('next') ?? '/')

    if (!next_password || !confirm) {
      return Response.redirect(
        new URL(`/change-password?error=missing&next=${encodeURIComponent(next)}`, request.url),
        303,
      )
    }
    if (next_password !== confirm) {
      return Response.redirect(
        new URL(`/change-password?error=mismatch&next=${encodeURIComponent(next)}`, request.url),
        303,
      )
    }

    let cookie = request.headers.get('cookie')
    let upstream = await api.changePasswordRaw(cookie, {
      current_password: current,
      new_password: next_password,
      new_password_confirm: confirm,
    })
    if (!upstream.ok) {
      let code = upstream.status === 401 ? 'wrong-current' : upstream.status === 403 ? 'forbidden' : 'server'
      return Response.redirect(
        new URL(`/change-password?error=${code}&next=${encodeURIComponent(next)}`, request.url),
        303,
      )
    }
    return Response.redirect(new URL(next, request.url), 303)
  },
}

export const changePassword = { index: showForm, action: submitForm }

interface Props {
  locale: string
  theme: Theme
  next: string
  error: string | null
  forced: boolean
}

/// Same standalone shell as the login page — no sidebar, single centered
/// card. We use it both for the admin-reset forced flow (where the user
/// just logged in with a temp password) and for opt-in self-service
/// changes.
function ChangePasswordPage() {
  return ({ locale, theme, next, error, forced }: Props) => {
    let m = messages(locale).auth.changePassword
    return (
    <Document title={`${m.title} · Plutus`} lang={locale} theme={theme}>
      <div
        mix={css({
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: space[6],
          background: color.bg,
          position: 'relative',
        })}
      >
        <div
          mix={css({
            position: 'absolute',
            top: space[5],
            right: space[5],
          })}
        >
          <LocaleToggle locale={locale} next={next} />
        </div>
        <div
          mix={css({
            width: '100%',
            maxWidth: '420px',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
          })}
        >
          <BrandMark size={40} />

          <div
            mix={css({
              marginTop: space[6],
              width: '100%',
              background: color.surface,
              border: `1px solid ${color.border}`,
              borderRadius: radius.lg,
              padding: `${space[6]} ${space[6]}`,
            })}
          >
            <h1
              mix={css({
                margin: 0,
                fontSize: font.xl,
                fontWeight: 700,
                color: color.text,
                letterSpacing: '-0.01em',
                textAlign: 'center',
              })}
            >
              {m.title}
            </h1>

            <p
              mix={css({
                margin: `${space[3]} 0 0`,
                fontSize: font.sm,
                color: color.textMuted,
                textAlign: 'center',
                lineHeight: 1.5,
              })}
            >
              {forced ? m.hintForced : m.hintOptional}
            </p>

            {error && (
              <div mix={css({ marginTop: space[4] })}>
                <ErrorBanner code={error} locale={locale} />
              </div>
            )}

            <form
              method="post"
              action="/change-password"
              mix={css({ marginTop: space[5] })}
            >
              <input type="hidden" name="next" value={next} />
              <input
                name="current_password"
                type="password"
                placeholder={m.current}
                autoFocus
                autoComplete="current-password"
                mix={css(fieldStyle)}
              />
              <div mix={css({ marginTop: space[3] })}>
                <input
                  name="new_password"
                  type="password"
                  placeholder={m.next}
                  autoComplete="new-password"
                  mix={css(fieldStyle)}
                />
              </div>
              <div mix={css({ marginTop: space[3] })}>
                <input
                  name="new_password_confirm"
                  type="password"
                  placeholder={m.confirm}
                  autoComplete="new-password"
                  mix={css(fieldStyle)}
                />
              </div>
              <button
                type="submit"
                mix={css({
                  marginTop: space[4],
                  width: '100%',
                  padding: `${space[3]} ${space[4]}`,
                  background: color.brand,
                  color: '#fff',
                  border: 'none',
                  borderRadius: radius.md,
                  fontSize: font.base,
                  fontWeight: 600,
                  cursor: 'pointer',
                  transition: 'background 120ms ease',
                  '&:hover': { background: color.brandHover },
                })}
              >
                {m.submit}
              </button>
            </form>
          </div>
        </div>
      </div>
    </Document>
    )
  }
}

/// Compact en / zh-CN toggle — same shape as the one on /login. Inline
/// here (rather than imported) so each unauthenticated shell stays
/// self-contained; cookie persistence comes from `render()` which
/// always emits a Set-Cookie for the resolved locale.
function LocaleToggle() {
  return ({ locale, next }: { locale: string; next?: string }) => {
    let qs = (l: string) => {
      let p = new URLSearchParams()
      p.set('locale', l)
      if (next) p.set('next', next)
      return `?${p.toString()}`
    }
    return (
      <div
        mix={css({
          display: 'inline-flex',
          gap: space[1],
          padding: '3px',
          background: color.surface,
          border: `1px solid ${color.border}`,
          borderRadius: radius.pill,
        })}
      >
        {(['en', 'zh-CN'] as const).map((l) => {
          let active = l === locale
          return (
            <a
              href={qs(l)}
              mix={css({
                display: 'inline-flex',
                alignItems: 'center',
                padding: `${space[1]} ${space[3]}`,
                fontSize: font.sm,
                fontWeight: 600,
                borderRadius: radius.pill,
                textDecoration: 'none',
                color: active ? color.text : color.textMuted,
                background: active ? color.bg : 'transparent',
                transition: 'background 120ms ease, color 120ms ease',
                '&:hover': active ? undefined : { color: color.text },
              })}
            >
              {l === 'en' ? 'EN' : '中文'}
            </a>
          )
        })}
      </div>
    )
  }
}

const fieldStyle = {
  width: '100%',
  padding: `${space[3]} ${space[3]}`,
  background: color.bg,
  border: `1px solid ${color.border}`,
  borderRadius: radius.md,
  fontSize: font.base,
  color: color.text,
  fontFamily: font.sans,
  outline: 'none',
  '&:focus': { borderColor: color.brand, background: color.surface },
  '&::placeholder': { color: color.textDim },
}

function ErrorBanner() {
  return ({ code, locale }: { code: string; locale: string }) => {
    let m = messages(locale).auth.changePassword
    let message =
      code === 'wrong-current'
        ? m.errWrongCurrent
        : code === 'mismatch'
          ? m.errMismatch
          : code === 'missing'
            ? m.errMissing
            : code === 'forbidden'
              ? m.errForbidden
              : m.errServer
    return (
      <div
        mix={css({
          padding: `${space[2]} ${space[3]}`,
          background: color.dangerSoft,
          color: color.dangerText,
          borderRadius: radius.md,
          fontSize: font.sm,
          textAlign: 'center',
        })}
      >
        {message}
      </div>
    )
  }
}
