import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import type { routes } from '../routes.ts'
import { Document } from '../ui/document.tsx'
import { color, font, radius, resolveLocale, space } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

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

/// Standalone auth page — bypasses [[Layout]] so the sidebar and chip row
/// don't appear before the user is signed in. Just centers the brand mark
/// + a single card on a full-bleed background.
function LoginPage() {
  return ({ locale, next, error }: LoginProps) => (
    <Document title="Sign in · plutus" lang={locale}>
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
          <BrandMark />

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
              Sign in
            </h1>

            {error && (
              <div mix={css({ marginTop: space[4] })}>
                <ErrorBanner code={error} />
              </div>
            )}

            <form
              method="post"
              action="/login"
              mix={css({ marginTop: space[5] })}
            >
              <input type="hidden" name="next" value={next} />
              <input
                id="password"
                name="password"
                type="password"
                placeholder="Master password"
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
                  '&::placeholder': { color: color.textDim },
                })}
              />
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
                Sign in
              </button>
            </form>
          </div>
        </div>
      </div>
    </Document>
  )
}

function BrandMark() {
  return () => (
    <div
      mix={css({
        display: 'flex',
        alignItems: 'center',
        gap: space[2],
      })}
    >
      <div
        mix={css({
          width: '36px',
          height: '36px',
          borderRadius: radius.md,
          background: `linear-gradient(135deg, ${color.brand}, ${color.brandHover})`,
          display: 'inline-flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#fff',
          fontWeight: 700,
          fontSize: font.lg,
          letterSpacing: '-0.02em',
        })}
      >
        P
      </div>
      <span
        mix={css({
          fontSize: font.xl,
          fontWeight: 700,
          color: color.text,
          letterSpacing: '-0.02em',
        })}
      >
        plutus
      </span>
    </div>
  )
}

function ErrorBanner() {
  return ({ code }: { code: string }) => {
    let message =
      code === 'bad-password'
        ? 'Wrong password.'
        : code === 'missing'
          ? 'Enter the password.'
          : 'Login failed.'
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
