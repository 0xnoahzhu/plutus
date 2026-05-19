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

import { AdminTabs } from './admin.tsx'

interface TokenRow {
  id: number
  label: string
  created_at: string
  last_used_at: string | null
  revoked_at: string | null
}

interface Props {
  locale: string
  theme: Theme
  tokens: TokenRow[]
  error: string | null
  flash: string | null
  /// Freshly minted plaintext token, shown only on the render directly
  /// following the POST that created it. Putting it through a redirect
  /// URL or cookie would leak it into browser history and access logs;
  /// rendering once and forcing the admin to navigate away is safer.
  freshToken: string | null
  freshLabel: string | null
}

/// GET /admin/tokens — list every admin-grade token.
export const adminTokens: BuildAction<'GET', typeof routes.adminTokens> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let cookie = request.headers.get('cookie')
    let error = url.searchParams.get('error')
    let flash = url.searchParams.get('flash')

    let tokens: TokenRow[] = []
    try {
      tokens = await api.adminListTokens(cookie)
    } catch {
      return Response.redirect(new URL('/login', request.url), 303)
    }
    tokens.sort((a, b) => b.created_at.localeCompare(a.created_at))

    return render(
      <AdminTokensPage
        locale={locale}
        theme={theme}
        tokens={tokens}
        error={error}
        flash={flash}
        freshToken={null}
        freshLabel={null}
      />,
      request,
      { locale, theme },
    )
  },
}

/// POST /admin/tokens/new — mint a fresh admin-grade token and render
/// the page directly so the plain token surfaces in a copy-this-now
/// banner. No redirect because the secret must not travel through a
/// `Location` header.
export const adminTokenCreate: BuildAction<'POST', typeof routes.adminTokenCreate> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let form = await request.formData()
    let label = String(form.get('label') ?? '').trim()
    let cookie = request.headers.get('cookie')

    if (!label) {
      return Response.redirect(
        new URL('/admin/tokens?error=missing', request.url),
        303,
      )
    }
    let upstream = await api.adminCreateTokenRaw(cookie, label)
    if (!upstream.ok) {
      let code = upstream.status === 403 ? 'forbidden' : 'server'
      return Response.redirect(
        new URL(`/admin/tokens?error=${code}`, request.url),
        303,
      )
    }
    let body = (await upstream.json()) as { token: string; label: string }
    let tokens = await api.adminListTokens(cookie).catch(() => [] as TokenRow[])
    tokens.sort((a, b) => b.created_at.localeCompare(a.created_at))
    return render(
      <AdminTokensPage
        locale={locale}
        theme={theme}
        tokens={tokens}
        error={null}
        flash={null}
        freshToken={body.token}
        freshLabel={body.label}
      />,
      request,
      { locale, theme },
    )
  },
}

export const adminTokenRevoke: BuildAction<'POST', typeof routes.adminTokenRevoke> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/admin/tokens?error=server', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminRevokeTokenRaw(cookie, id)
    if (!upstream.ok) {
      return Response.redirect(new URL('/admin/tokens?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/admin/tokens?flash=revoked', request.url), 303)
  },
}

function AdminTokensPage() {
  return ({ locale, theme, tokens, error, flash, freshToken, freshLabel }: Props) => {
    let m = messages(locale)
    let p = m.adminTokens
    return (
      <Document title={`${p.title} · Plutus`} lang={locale} theme={theme}>
        <div
          mix={css({
            minHeight: '100vh',
            background: color.bg,
            padding: `${space[6]} ${space[6]}`,
          })}
        >
          <div mix={css({ maxWidth: '780px', margin: '0 auto' })}>
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
                    '&:hover': { background: color.hover, color: color.danger },
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
                lineHeight: 1.5,
              })}
            >
              {p.subtitle}
            </p>

            <AdminTabs locale={locale} active="tokens" />

            {freshToken && (
              <FreshTokenBanner
                locale={locale}
                token={freshToken}
                label={freshLabel ?? ''}
              />
            )}
            {(error || flash) && !freshToken && (
              <div mix={css({ marginBottom: space[4] })}>
                <Banner error={error} flash={flash} locale={locale} />
              </div>
            )}

            <Card>
              <SectionTitle>{p.createSection}</SectionTitle>
              <form
                method="post"
                action="/admin/tokens/new"
                mix={css({
                  display: 'flex',
                  gap: space[3],
                  flexWrap: 'wrap',
                  alignItems: 'center',
                  marginTop: space[3],
                })}
              >
                <input
                  name="label"
                  type="text"
                  placeholder={p.labelPlaceholder}
                  required
                  autoComplete="off"
                  mix={css({ ...fieldStyle, flex: '1 1 280px' })}
                />
                <button type="submit" mix={css(primaryButton)}>
                  {p.createSubmit}
                </button>
              </form>
            </Card>

            <div mix={css({ marginTop: space[5] })}>
              <Card>
                <SectionTitle>{p.listSection}</SectionTitle>
                {tokens.length === 0 ? (
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
                        <Th>{p.columnLabel}</Th>
                        <Th>{p.columnCreated}</Th>
                        <Th>{p.columnLastUsed}</Th>
                        <Th>{p.columnStatus}</Th>
                        <Th>{''}</Th>
                      </tr>
                    </thead>
                    <tbody>
                      {tokens.map((t) => (
                        <TokenRowView token={t} locale={locale} />
                      ))}
                    </tbody>
                  </table>
                )}
              </Card>
            </div>
          </div>
        </div>
      </Document>
    )
  }
}

function TokenRowView() {
  return ({ token, locale }: { token: TokenRow; locale: string }) => {
    let p = messages(locale).adminTokens
    let revoked = token.revoked_at != null
    return (
      <tr mix={css({ borderTop: `1px solid ${color.borderSoft}` })}>
        <Td>{token.label}</Td>
        <Td>{token.created_at.slice(0, 10)}</Td>
        <Td>{token.last_used_at ? token.last_used_at.slice(0, 10) : p.neverUsed}</Td>
        <Td>
          <span
            mix={css({
              padding: `2px ${space[2]}`,
              borderRadius: radius.sm,
              fontSize: font.xs,
              fontWeight: 600,
              background: revoked ? color.dangerSoft : color.successSoft,
              color: revoked ? color.dangerText : color.successText,
            })}
          >
            {revoked ? p.statusRevoked : p.statusActive}
          </span>
        </Td>
        <Td>
          {!revoked && (
            <form
              method="post"
              action={`/admin/tokens/${token.id}/revoke`}
              mix={css({ margin: 0 })}
            >
              <button type="submit" mix={css(dangerButton)}>
                {p.revokeSubmit}
              </button>
            </form>
          )}
        </Td>
      </tr>
    )
  }
}

function FreshTokenBanner() {
  return ({
    locale,
    token,
    label,
  }: {
    locale: string
    token: string
    label: string
  }) => {
    let p = messages(locale).adminTokens
    return (
      <div
        mix={css({
          marginBottom: space[5],
          padding: `${space[4]} ${space[5]}`,
          background: color.successSoft,
          color: color.successText,
          border: `1px solid ${color.success}`,
          borderRadius: radius.lg,
        })}
      >
        <div mix={css({ fontWeight: 700, fontSize: font.md, marginBottom: space[2] })}>
          {p.flashCreatedTitle}
        </div>
        <div mix={css({ fontSize: font.sm, marginBottom: space[3], opacity: 0.85 })}>
          {label} — {p.flashCreatedHint}
        </div>
        <code
          mix={css({
            display: 'block',
            padding: `${space[2]} ${space[3]}`,
            background: color.surface,
            color: color.text,
            borderRadius: radius.md,
            fontFamily: font.mono,
            fontSize: font.sm,
            wordBreak: 'break-all',
            userSelect: 'all',
          })}
        >
          {token}
        </code>
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
    let p = messages(locale).adminTokens
    let { tone, message } =
      error === 'missing'
        ? { tone: 'error' as const, message: p.errMissingLabel }
        : error
          ? { tone: 'error' as const, message: p.errServer }
          : flash === 'revoked'
            ? { tone: 'success' as const, message: p.flashRevoked }
            : { tone: 'success' as const, message: '' }
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
