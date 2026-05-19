import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import type { routes } from '../routes.ts'
import {
  color,
  font,
  Layout,
  radius,
  resolveLocale,
  space,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

/// `routes.login` is a `form()` pair — `routes.login.index` is the GET that
/// shows the form, `routes.login.action` is the POST that handles submit.
/// We propagate the upstream `Set-Cookie` from the API response so the
/// browser keeps the plutus_session cookie.
const showForm: BuildAction<'GET', typeof routes.login.index> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let next = url.searchParams.get('next') ?? '/'
    let error = url.searchParams.get('error')
    return render(
      <LoginPage locale={locale} next={next} error={error} />,
      request,
      { locale },
    )
  },
}

const submitForm: BuildAction<'POST', typeof routes.login.action> = {
  async handler({ request }) {
    let form = await request.formData()
    let password = String(form.get('password') ?? '')
    let next = String(form.get('next') ?? '/')

    if (!password) {
      return Response.redirect(new URL('/login?error=missing', request.url), 303)
    }

    let upstream = await api.loginRaw(password)
    if (!upstream.ok) {
      let code = upstream.status === 401 ? 'bad-password' : 'server'
      return Response.redirect(new URL(`/login?error=${code}`, request.url), 303)
    }

    // Forward the session cookie back to the browser. The API sets it as
    // HttpOnly + SameSite=Lax + Path=/ — passing the value through verbatim
    // keeps those attributes intact.
    let headers = new Headers({ Location: next })
    let setCookie = upstream.headers.get('set-cookie')
    if (setCookie) headers.set('Set-Cookie', setCookie)
    return new Response(null, { status: 303, headers })
  },
}

export const login = { index: showForm, action: submitForm }

interface LoginProps {
  locale: string
  next: string
  error: string | null
}

function LoginPage() {
  return ({ locale, next, error }: LoginProps) => (
    <Layout title="Sign in" locale={locale}>
      <div
        mix={css({
          display: 'flex',
          justifyContent: 'center',
          paddingTop: space[10],
        })}
      >
        <div
          mix={css({
            width: '100%',
            maxWidth: '420px',
            background: color.surface,
            border: `1px solid ${color.border}`,
            borderRadius: radius.lg,
            padding: `${space[6]} ${space[6]}`,
          })}
        >
          <h2
            mix={css({
              margin: 0,
              fontSize: font.xl,
              fontWeight: 700,
              color: color.text,
              letterSpacing: '-0.01em',
            })}
          >
            Sign in
          </h2>
          <p
            mix={css({
              margin: `${space[2]} 0 ${space[5]}`,
              fontSize: font.sm,
              color: color.textMuted,
            })}
          >
            Single-user mode. Enter the master password configured via{' '}
            <code>PLUTUS_MASTER_PASSWORD_HASH</code>.
          </p>

          {error && <ErrorBanner code={error} />}

          <form method="post" action="/login">
            <input type="hidden" name="next" value={next} />
            <Label htmlFor="password">Master password</Label>
            <input
              id="password"
              name="password"
              type="password"
              autoFocus
              autoComplete="current-password"
              mix={css({
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
              })}
            />
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
              Sign in
            </button>
          </form>
        </div>
      </div>
    </Layout>
  )
}

function Label() {
  return ({ htmlFor, children }: { htmlFor: string; children: string }) => (
    <label
      htmlFor={htmlFor}
      mix={css({
        display: 'block',
        marginBottom: space[2],
        fontSize: font.xs,
        fontWeight: 600,
        color: color.textMuted,
        textTransform: 'uppercase',
        letterSpacing: '0.08em',
      })}
    >
      {children}
    </label>
  )
}

function ErrorBanner() {
  return ({ code }: { code: string }) => {
    let message =
      code === 'bad-password'
        ? 'Wrong password. Try again.'
        : code === 'missing'
          ? 'Enter the password.'
          : 'Login failed — check the server logs.'
    return (
      <div
        mix={css({
          marginBottom: space[4],
          padding: `${space[2]} ${space[3]}`,
          background: color.dangerSoft,
          color: color.dangerText,
          borderRadius: radius.md,
          fontSize: font.sm,
        })}
      >
        {message}
      </div>
    )
  }
}
