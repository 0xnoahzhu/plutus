import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import { Document } from '../ui/document.tsx'
import { BrandMark, color, font, radius, resolveLocale, resolveTheme, space, type Theme } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

const showForm: BuildAction<'GET', typeof routes.login.index> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let next = url.searchParams.get('next') ?? '/'
    let error = url.searchParams.get('error')
    return render(
      <LoginPage locale={locale} theme={theme} next={next} error={error} />,
      request,
      { locale, theme },
    )
  },
}

const submitForm: BuildAction<'POST', typeof routes.login.action> = {
  async handler({ request }) {
    let form = await request.formData()
    let username = String(form.get('username') ?? '').trim()
    let password = String(form.get('password') ?? '')
    let next = String(form.get('next') ?? '/')

    if (!username || !password) {
      return Response.redirect(new URL('/login?error=missing', request.url), 303)
    }

    let upstream = await api.loginRaw(username, password)
    if (!upstream.ok) {
      let code = upstream.status === 401 ? 'bad-credentials' : 'server'
      return Response.redirect(new URL(`/login?error=${code}`, request.url), 303)
    }

    // Forced password change: the server signals via the response body.
    // Sit the user on /change-password until they update their password,
    // and only then send them on to `next`.
    let body: { password_reset_required?: boolean; is_admin?: boolean } = {}
    try {
      body = await upstream.json()
    } catch {
      // Empty body on success is unexpected but harmless — fall through.
    }
    let location = body.password_reset_required
      ? `/change-password?next=${encodeURIComponent(next)}`
      : body.is_admin
        ? '/admin'
        : next

    let headers = new Headers({ Location: location })
    let setCookie = upstream.headers.get('set-cookie')
    if (setCookie) headers.set('Set-Cookie', setCookie)
    return new Response(null, { status: 303, headers })
  },
}

export const login = { index: showForm, action: submitForm }

interface LoginProps {
  locale: string
  theme: Theme
  next: string
  error: string | null
}

/// Standalone auth page — bypasses [[Layout]] so the sidebar and chip row
/// don't appear before the user is signed in. Just centers the brand mark
/// + a single card on a full-bleed background.
function LoginPage() {
  return ({ locale, theme, next, error }: LoginProps) => {
    let m = messages(locale).auth.login
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
        })}
      >
        <div
          mix={css({
            width: '100%',
            maxWidth: '380px',
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

            {error && (
              <div mix={css({ marginTop: space[4] })}>
                <ErrorBanner code={error} locale={locale} />
              </div>
            )}

            <form
              method="post"
              action="/login"
              mix={css({ marginTop: space[5] })}
            >
              <input type="hidden" name="next" value={next} />
              <input
                id="username"
                name="username"
                type="text"
                placeholder={m.username}
                autoFocus
                autoComplete="username"
                mix={css(fieldStyle)}
              />
              <div mix={css({ marginTop: space[3] })}>
                <input
                  id="password"
                  name="password"
                  type="password"
                  placeholder={m.password}
                  autoComplete="current-password"
                  mix={css(fieldStyle)}
                />
              </div>
              <button
                type="submit"
                mix={css({
                  marginTop: space[3],
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
  '&:focus': {
    borderColor: color.brand,
    background: color.surface,
  },
  '&::placeholder': { color: color.textDim },
}

function ErrorBanner() {
  return ({ code, locale }: { code: string; locale: string }) => {
    let m = messages(locale).auth.login
    let message =
      code === 'bad-credentials'
        ? m.errBadCredentials
        : code === 'missing'
          ? m.errMissing
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
