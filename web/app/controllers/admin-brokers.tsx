import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Broker } from '../api.ts'
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

interface Props {
  locale: string
  theme: Theme
  brokers: Broker[]
  error: string | null
  flash: string | null
}

/// GET /admin/brokers — list every registered broker. Like the admin
/// users page, anonymous / non-admin sessions get bounced to /login
/// because the upstream call returns 403 and we treat that as
/// not-authorized.
export const adminBrokers: BuildAction<'GET', typeof routes.adminBrokers> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let error = url.searchParams.get('error')
    let flash = url.searchParams.get('flash')

    let cookie = request.headers.get('cookie')
    let brokers: Broker[] = []
    try {
      brokers = await api.adminListBrokers(cookie)
    } catch {
      return Response.redirect(new URL('/login', request.url), 303)
    }
    brokers.sort((a, b) => a.code.localeCompare(b.code))

    return render(
      <AdminBrokersPage
        locale={locale}
        theme={theme}
        brokers={brokers}
        error={error}
        flash={flash}
      />,
      request,
      { locale, theme },
    )
  },
}

export const adminBrokerCreate: BuildAction<'POST', typeof routes.adminBrokerCreate> = {
  async handler({ request }) {
    let form = await request.formData()
    let code = String(form.get('code') ?? '').trim()
    let name = String(form.get('name') ?? '').trim()
    if (!code || !name) {
      return Response.redirect(new URL('/admin/brokers?error=missing-create', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminCreateBrokerRaw(cookie, { code, name })
    if (!upstream.ok) {
      let err =
        upstream.status === 409
          ? 'taken'
          : upstream.status === 403
            ? 'forbidden'
            : 'server'
      return Response.redirect(new URL(`/admin/brokers?error=${err}`, request.url), 303)
    }
    return Response.redirect(new URL('/admin/brokers?flash=created', request.url), 303)
  },
}

export const adminBrokerRename: BuildAction<'POST', typeof routes.adminBrokerRename> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/admin/brokers?error=bad-id', request.url), 303)
    }
    let form = await request.formData()
    let name = String(form.get('name') ?? '').trim()
    if (!name) {
      return Response.redirect(new URL('/admin/brokers?error=missing-rename', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminUpdateBrokerRaw(cookie, id, name)
    if (!upstream.ok) {
      return Response.redirect(new URL('/admin/brokers?error=server', request.url), 303)
    }
    return Response.redirect(new URL('/admin/brokers?flash=renamed', request.url), 303)
  },
}

export const adminBrokerDelete: BuildAction<'POST', typeof routes.adminBrokerDelete> = {
  async handler({ request, params }) {
    let id = Number(params.id)
    if (!Number.isFinite(id)) {
      return Response.redirect(new URL('/admin/brokers?error=bad-id', request.url), 303)
    }
    let cookie = request.headers.get('cookie')
    let upstream = await api.adminDeleteBrokerRaw(cookie, id)
    if (!upstream.ok) {
      let err = upstream.status === 409 ? 'in-use' : 'server'
      return Response.redirect(new URL(`/admin/brokers?error=${err}`, request.url), 303)
    }
    return Response.redirect(new URL('/admin/brokers?flash=deleted', request.url), 303)
  },
}

function AdminBrokersPage() {
  return ({ locale, theme, brokers, error, flash }: Props) => {
    let m = messages(locale)
    let p = m.adminBrokers
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
              })}
            >
              {p.subtitle}
            </p>

            <AdminTabs locale={locale} active="brokers" />

            {(error || flash) && (
              <div mix={css({ marginBottom: space[4] })}>
                <Banner error={error} flash={flash} locale={locale} />
              </div>
            )}

            <Card>
              <SectionTitle>{p.createSection}</SectionTitle>
              <form
                method="post"
                action="/admin/brokers/new"
                mix={css({
                  display: 'flex',
                  gap: space[3],
                  flexWrap: 'wrap',
                  alignItems: 'center',
                  marginTop: space[3],
                })}
              >
                <input
                  name="code"
                  type="text"
                  placeholder={p.codePlaceholder}
                  required
                  autoComplete="off"
                  mix={css({ ...fieldStyle, flex: '1 1 180px' })}
                />
                <input
                  name="name"
                  type="text"
                  placeholder={p.namePlaceholder}
                  required
                  autoComplete="off"
                  mix={css({ ...fieldStyle, flex: '1 1 220px' })}
                />
                <button type="submit" mix={css(primaryButton)}>
                  {p.createSubmit}
                </button>
              </form>
            </Card>

            <div mix={css({ marginTop: space[5] })}>
              <Card>
                <SectionTitle>{p.listSection}</SectionTitle>
                {brokers.length === 0 ? (
                  <div mix={css({ marginTop: space[3] })}>
                    <EmptyState title={p.emptyTitle} hint={p.emptyHint} />
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
                    {brokers.map((b) => (
                      <BrokerRow broker={b} locale={locale} />
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

function BrokerRow() {
  return ({ broker, locale }: { broker: Broker; locale: string }) => {
    let p = messages(locale).adminBrokers
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
            <div mix={css({ fontSize: font.base, fontWeight: 600, color: color.text })}>
              {broker.name}
            </div>
            <div
              mix={css({
                fontFamily: font.mono,
                fontSize: font.xs,
                color: color.textMuted,
                marginTop: space[1],
              })}
            >
              {broker.code} · #{broker.id}
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
            action={`/admin/brokers/${broker.id}/rename`}
            mix={css({ display: 'flex', gap: space[2], flex: '1 1 280px', margin: 0 })}
          >
            <input
              name="name"
              type="text"
              placeholder={p.renamePlaceholder}
              required
              autoComplete="off"
              mix={css({ ...fieldStyle, flex: '1 1 auto' })}
            />
            <button type="submit" mix={css(secondaryButton)}>
              {p.renameSubmit}
            </button>
          </form>
          <form
            method="post"
            action={`/admin/brokers/${broker.id}/delete`}
            mix={css({ margin: 0 })}
          >
            <button type="submit" mix={css(dangerButton)}>
              {p.deleteSubmit}
            </button>
          </form>
        </div>
      </li>
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
    let p = messages(locale).adminBrokers
    let { tone, message } =
      error === 'missing-create'
        ? { tone: 'error' as const, message: p.errMissingCreate }
        : error === 'missing-rename'
          ? { tone: 'error' as const, message: p.errMissingRename }
          : error === 'bad-id'
            ? { tone: 'error' as const, message: p.errBadId }
            : error === 'taken'
              ? { tone: 'error' as const, message: p.errTaken }
              : error === 'in-use'
                ? { tone: 'error' as const, message: p.errInUse }
                : error
                  ? { tone: 'error' as const, message: p.errServer }
                  : flash === 'created'
                    ? { tone: 'success' as const, message: p.flashCreated }
                    : flash === 'renamed'
                      ? { tone: 'success' as const, message: p.flashRenamed }
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
  padding: `${space[3]} ${space[4]}`,
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
  padding: `${space[2]} ${space[3]}`,
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
