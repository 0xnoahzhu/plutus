import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Card,
  color,
  EmptyState,
  font,
  Layout,
  radius,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  type Theme,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

interface TokenRow {
  id: number
  label: string
  /// Plaintext token. `null` for legacy rows minted before the column
  /// existed — those render as "—" with no copy button.
  token_plain: string | null
  created_at: string
  last_used_at: string | null
}

/// Mask a token for the list view: `chD3...4UUU`. Keep 4 chars at each
/// end so users can identify which token it is without exposing the
/// whole secret on-screen. The full value still lives in the DOM via
/// `data-copy` so the copy button can grab it.
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

/// GET /api-keys — list the caller's tokens. The plaintext lives on
/// each row alongside its hash (see `models/api_token.rs`), so the
/// list itself is the canonical place to grab a token; no separate
/// "show once" banner needed.
export const apiKeys: BuildAction<'GET', typeof routes.apiKeys> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let flash = url.searchParams.get('flash')
    let error = url.searchParams.get('error')
    let tokens = (await api.tokens().catch(() => [])) as TokenRow[]
    tokens.sort((a, b) => b.created_at.localeCompare(a.created_at))
    return render(
      <ApiKeysPage
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

/// POST /api-keys/new — create. Plain 303 back to the list with a
/// `?flash=created` success banner. The new token is visible on its
/// list row (masked + Copy button); refreshes don't re-show anything
/// secret because the plaintext is rendered the same way every time.
export const apiKeyCreate: BuildAction<'POST', typeof routes.apiKeyCreate> = {
  async handler({ request }) {
    let form = await request.formData()
    let label = String(form.get('label') ?? '').trim()
    let cookie = request.headers.get('cookie')

    if (!label) {
      return Response.redirect(new URL('/api-keys?error=missing', request.url), 303)
    }
    let upstream = await api.createTokenRaw(cookie, label)
    if (!upstream.ok) {
      return Response.redirect(new URL('/api-keys?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/api-keys?flash=created', request.url), 303)
  },
}

/// POST /api-keys/:id/delete — hard delete the row.
export const apiKeyDelete: BuildAction<'POST', typeof routes.apiKeyDelete> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/api-keys?error=server', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.deleteTokenRaw(cookie, id)
    if (!upstream.ok) {
      return Response.redirect(new URL('/api-keys?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/api-keys?flash=deleted', request.url), 303)
  },
}

function ApiKeysPage() {
  return ({ locale, theme, tokens, error, flash }: Props) => {
    let m = messages(locale)
    let p = m.apiKeys
    let pageTitle = m.pages.apiKeys
    return (
      <Layout
        title={pageTitle.title}
        subtitle={pageTitle.subtitle}
        locale={locale}
        theme={theme}
      >
        {(error || flash) && (
          <div mix={css({ marginBottom: space[4] })}>
            <Banner error={error} flash={flash} locale={locale} />
          </div>
        )}

        <Card>
          <SectionTitle>{p.createSection}</SectionTitle>
          <p
            mix={css({
              margin: `${space[2]} 0 ${space[4]}`,
              fontSize: font.sm,
              color: color.textMuted,
              lineHeight: 1.5,
            })}
          >
            {p.description}
          </p>
          <form
            method="post"
            action="/api-keys/new"
            mix={css({
              display: 'flex',
              gap: space[3],
              flexWrap: 'wrap',
              alignItems: 'center',
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
                    <TokenRowView key={t.id} token={t} locale={locale} />
                  ))}
                </tbody>
              </table>
            )}
          </Card>
        </div>
      </Layout>
    )
  }
}

function TokenRowView() {
  return ({ token, locale }: { token: TokenRow; locale: string }) => {
    let all = messages(locale)
    let p = all.apiKeys
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
              {/* The full plaintext lives in `data-copy` only; the rendered
                  text is the mask. The global handler in document.tsx
                  copies the attribute value to the clipboard on click. */}
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
        <Td>{token.created_at.slice(0, 10)}</Td>
        <Td>{token.last_used_at ? token.last_used_at.slice(0, 10) : p.neverUsed}</Td>
        <Td>
          <form
            method="post"
            action={`/api-keys/${token.id}/delete`}
            mix={css({ margin: 0 })}
          >
            <button
              type="submit"
              title={confirms.deleteApiKey(token.label)}
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
    let p = messages(locale).apiKeys
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
