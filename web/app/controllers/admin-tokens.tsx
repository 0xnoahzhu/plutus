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
import { LocalTime } from '../ui/local-time.tsx'
import { render } from '../utils/render.tsx'

import { AdminTabs } from './admin.tsx'

interface TokenRow {
  id: number
  label: string
  /// Plaintext token. `null` for legacy rows (minted before the column
  /// existed) — they render with "—" and no copy affordance.
  token_plain: string | null
  created_at: string
  last_used_at: string | null
}

/// `head:4...tail:4` mask for the list view. Full value still goes
/// into `data-copy` so the copy button can pull it intact.
function maskToken(token: string): string {
  if (token.length <= 12) return token
  return `${token.slice(0, 4)}...${token.slice(-4)}`
}

interface Props {
  locale: string
  theme: Theme
  tokens: TokenRow[]
  error: string | null
  flash: string | null
}

/// GET /admin/tokens — list every admin-grade token. The plaintext lives
/// on each row alongside its hash (see `models/api_token.rs`), so the
/// list itself is the canonical place to grab a token; no separate
/// "show once" banner needed.
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
      />,
      request,
      { locale, theme },
    )
  },
}

/// POST /admin/tokens/new — mint a fresh admin-grade token. Plain 303
/// back to the list with a `?flash=created` success banner; the new
/// token shows up on its list row (masked + Copy button), so no
/// separate "show once" banner is needed.
export const adminTokenCreate: BuildAction<'POST', typeof routes.adminTokenCreate> = {
  async handler({ request }) {
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
    return Response.redirect(
      new URL('/admin/tokens?flash=created', request.url),
      303,
    )
  },
}

export const adminTokenDelete: BuildAction<'POST', typeof routes.adminTokenDelete> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/admin/tokens?error=server', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminDeleteTokenRaw(cookie, id)
    if (!upstream.ok) {
      return Response.redirect(new URL('/admin/tokens?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/admin/tokens?flash=deleted', request.url), 303)
  },
}

function AdminTokensPage() {
  return ({ locale, theme, tokens, error, flash }: Props) => {
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

            {(error || flash) && (
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
                        <Th>{p.columnKey}</Th>
                        <Th>{p.columnCreated}</Th>
                        <Th>{p.columnLastUsed}</Th>
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
    let all = messages(locale)
    let p = all.adminTokens
    let confirms = all.confirms
    return (
      <tr mix={css({ borderTop: `1px solid ${color.borderSoft}` })}>
        <Td>{token.label}</Td>
        <Td>
          {token.token_plain ? (
            <div
              mix={css({
                display: 'inline-flex',
                alignItems: 'center',
                gap: space[2],
              })}
            >
              <code
                mix={css({
                  fontFamily: font.mono,
                  fontSize: font.sm,
                  color: color.textMuted,
                })}
              >
                {maskToken(token.token_plain)}
              </code>
              {/* Full plaintext lives only in `data-copy`; the rendered
                  text is the head/tail mask. The global click handler in
                  document.tsx grabs the attribute value into the
                  clipboard. */}
              <button
                type="button"
                data-copy={token.token_plain}
                data-copy-done={p.copied}
                mix={css(copyButton)}
              >
                {p.copy}
              </button>
            </div>
          ) : (
            <span mix={css({ color: color.textDim })}>—</span>
          )}
        </Td>
        <Td>
          <LocalTime value={token.created_at} format="date" />
        </Td>
        <Td>
          {token.last_used_at ? (
            <LocalTime value={token.last_used_at} format="date" />
          ) : (
            p.neverUsed
          )}
        </Td>
        <Td>
          <form
            method="post"
            action={`/admin/tokens/${token.id}/delete`}
            mix={css({ margin: 0 })}
          >
            <button
              type="submit"
              title={confirms.deleteAdminToken(token.label)}
              mix={css(dangerButton)}
            >
              {p.deleteSubmit}
            </button>
          </form>
        </Td>
      </tr>
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
          : flash === 'created'
            ? { tone: 'success' as const, message: p.flashCreated }
            : flash === 'deleted'
              ? { tone: 'success' as const, message: p.flashDeleted }
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

const copyButton = {
  padding: `2px ${space[2]}`,
  background: 'transparent',
  border: `1px solid ${color.border}`,
  borderRadius: radius.sm,
  color: color.text,
  fontSize: font.xs,
  fontWeight: 500,
  fontFamily: 'inherit',
  cursor: 'pointer',
  '&:hover': { background: color.hover },
}
