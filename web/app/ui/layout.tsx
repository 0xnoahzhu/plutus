import { css, type RemixNode } from 'remix/ui'
import {
  ArrowLeftRight,
  BarChart3,
  Bookmark,
  ClipboardCheck,
  FileText,
  Filter,
  GitMerge,
  Globe,
  LayoutDashboard,
  LogOut,
  Newspaper,
  ScrollText,
  SearchCheck,
  ThumbsUp,
  TrendingUp,
  Wallet,
  Zap,
} from 'lucide-static'

import { routes } from '../routes.ts'

import { Document } from './document.tsx'
import { Icon } from './icon.tsx'
import { color, font, labelStyle, radius, shadow, space } from './tokens.ts'

export interface LayoutProps {
  children?: RemixNode
  title?: string
  /// Optional subtitle shown right under the title — useful for context like
  /// a date range or row count. Plain text only.
  subtitle?: string
  /// Currently-selected country (ISO alpha-2). When provided, the country
  /// chip group renders in the filter row.
  country?: string
  /// Resolved locale for this request — "en" or "zh-CN". Used both for
  /// content rendering and for the language chip group.
  locale: string
}

// ── Navigation ───────────────────────────────────────────────────────────────

/// Sidebar links only reference top-level single-method routes (the form
/// route pairs like `login` expose `{ index, action }` and aren't navigable
/// from the nav). Constraining to keys that carry `href` keeps the union
/// type aligned with the NavLink renderer.
type NavRoute = {
  [K in keyof typeof routes]: typeof routes[K] extends { href: (...args: never) => string }
    ? K
    : never
}[keyof typeof routes]

type NavEntry =
  | { kind: 'link'; route: NavRoute; label: string; icon: string }
  | { kind: 'divider'; label: string }

const link = (route: NavRoute, label: string, icon: string): NavEntry => ({
  kind: 'link',
  route,
  label,
  icon,
})
const divider = (label: string): NavEntry => ({ kind: 'divider', label })

const NAV: NavEntry[] = [
  link('home', 'Dashboard', LayoutDashboard),
  link('holdings', 'Holdings', Wallet),
  link('stocks', 'Stocks', TrendingUp),
  link('transactions', 'Transactions', ArrowLeftRight),
  link('watchlists', 'Watchlists', Bookmark),
  divider('Calendar'),
  link('news', 'News', Newspaper),
  link('briefs', 'Briefs', FileText),
  link('earnings', 'Earnings', BarChart3),
  link('macroEvents', 'Macro', Globe),
  link('catalysts', 'Catalysts', Zap),
  divider('Analysis'),
  link('screeners', 'Screeners', Filter),
  link('recommendations', 'Recommendations', ThumbsUp),
  link('portfolioReviews', 'Reviews', ClipboardCheck),
  link('correlations', 'Correlations', GitMerge),
  link('selfExams', 'Self-Exam', SearchCheck),
  divider(''),
  link('audit', 'Audit', ScrollText),
]

// ── Country / locale ─────────────────────────────────────────────────────────

// Country → list of MIC codes covered. Stocks carry market_code; we map back
// to a country to decide whether a row passes the country filter.
export const COUNTRY_TO_MARKETS: Record<string, string[]> = {
  US: ['XNAS', 'XNYS'],
  HK: ['XHKG'],
  CN: ['XSHG', 'XSHE'],
}
const ALL_COUNTRIES = Object.keys(COUNTRY_TO_MARKETS)

/// The country used when the URL has no `country` query parameter.
export const DEFAULT_COUNTRY = 'US'

/// Inverse lookup so controllers can resolve a `market_code` to its country.
export const MARKET_TO_COUNTRY: Record<string, string> = Object.fromEntries(
  Object.entries(COUNTRY_TO_MARKETS).flatMap(([country, markets]) =>
    markets.map((m) => [m, country]),
  ),
)

/// Read the `country` query parameter. Always returns one country — unknown
/// or missing values fall back to `DEFAULT_COUNTRY`.
export function parseCountry(search: URLSearchParams): string {
  let c = search.get('country')
  if (c && ALL_COUNTRIES.includes(c)) return c
  return DEFAULT_COUNTRY
}

/// Filter items by a single country. `pickMarket` returns each item's
/// `market_code`. Items whose market doesn't belong to the country (or
/// whose market_code can't be resolved) are dropped.
export function filterByCountry<T>(
  items: T[],
  country: string,
  pickMarket: (item: T) => string | undefined,
): T[] {
  return items.filter((item) => {
    let m = pickMarket(item)
    if (!m) return false
    return MARKET_TO_COUNTRY[m] === country
  })
}

/// Supported UI locales. zh-CN comes from the translations JSON map on each
/// agent-output row; en is the canonical source language.
export const LOCALES = ['en', 'zh-CN'] as const
export type Locale = (typeof LOCALES)[number]

export const LOCALE_LABELS: Record<Locale, string> = {
  en: 'English',
  'zh-CN': '中文',
}

export const DEFAULT_LOCALE: Locale = 'en'

const LOCALE_COOKIE = 'plutus_locale'

function isLocale(v: string | null): v is Locale {
  return v === 'en' || v === 'zh-CN'
}

export function parseLocale(search: URLSearchParams): Locale {
  let l = search.get('locale')
  return isLocale(l) ? l : DEFAULT_LOCALE
}

/// `?locale=` > cookie > Accept-Language (zh* → zh-CN) > "en".
export function resolveLocale(request: Request, search: URLSearchParams): Locale {
  let q = search.get('locale')
  if (isLocale(q)) return q

  let cookie = request.headers.get('cookie') ?? ''
  for (let part of cookie.split(';')) {
    let [k, v] = part.split('=').map((s) => s.trim())
    if (k === LOCALE_COOKIE && isLocale(v)) return v
  }

  let al = (request.headers.get('accept-language') ?? '').toLowerCase()
  if (al.includes('zh')) return 'zh-CN'

  return DEFAULT_LOCALE
}

export function localeCookie(locale: Locale): string {
  return `${LOCALE_COOKIE}=${locale}; Path=/; Max-Age=31536000; SameSite=Lax`
}

// ── Layout shell ─────────────────────────────────────────────────────────────

const SIDEBAR_WIDTH = '240px'

export function Layout() {
  return ({ title, subtitle, children, country, locale }: LayoutProps) => (
    <Document title={title ? `${title} · plutus` : 'plutus'} lang={locale}>
      <div
        mix={css({
          display: 'grid',
          gridTemplateColumns: `${SIDEBAR_WIDTH} 1fr`,
          minHeight: '100vh',
          background: color.bg,
        })}
      >
        <Sidebar />
        <main
          mix={css({
            padding: `${space[8]} ${space[10]}`,
            maxWidth: '1400px',
            width: '100%',
            // Tighten on narrow screens.
            '@media (max-width: 1100px)': {
              padding: `${space[6]} ${space[6]}`,
            },
          })}
        >
          {title && (
            <header
              mix={css({
                display: 'flex',
                alignItems: 'baseline',
                justifyContent: 'space-between',
                gap: space[4],
                marginBottom: subtitle ? space[1] : space[4],
                flexWrap: 'wrap',
              })}
            >
              <div>
                <h1
                  mix={css({
                    margin: 0,
                    fontSize: font.xxl,
                    fontWeight: 700,
                    color: color.text,
                    letterSpacing: '-0.01em',
                  })}
                >
                  {title}
                </h1>
                {subtitle && (
                  <p
                    mix={css({
                      margin: `${space[1]} 0 0`,
                      fontSize: font.sm,
                      color: color.textMuted,
                    })}
                  >
                    {subtitle}
                  </p>
                )}
              </div>
            </header>
          )}

          {/* Filter row — country + language chips below the title. */}
          <div
            mix={css({
              display: 'flex',
              gap: space[6],
              flexWrap: 'wrap',
              alignItems: 'center',
              marginTop: title ? space[4] : 0,
              marginBottom: space[6],
            })}
          >
            {country !== undefined && (
              <CountryChips selected={country} options={ALL_COUNTRIES} locale={locale} />
            )}
            <LocaleChips selected={locale as Locale} country={country} />
          </div>

          <div>{children}</div>
        </main>
      </div>
    </Document>
  )
}

// ── Sidebar ──────────────────────────────────────────────────────────────────

function Sidebar() {
  return () => (
    <aside
      mix={css({
        background: color.sidebar,
        borderRight: `1px solid ${color.divider}`,
        padding: `${space[6]} 0`,
        display: 'flex',
        flexDirection: 'column',
        position: 'sticky',
        top: 0,
        height: '100vh',
        overflowY: 'auto',
      })}
    >
      <Brand />
      <nav mix={css({ marginTop: space[6], flex: 1 })}>
        <ul mix={css({ listStyle: 'none', padding: 0, margin: 0 })}>
          {NAV.map((entry) =>
            entry.kind === 'divider' ? <NavDivider label={entry.label} /> : (
              <NavLink route={entry.route} label={entry.label} icon={entry.icon} />
            ),
          )}
        </ul>
      </nav>
      <LogoutLink />
    </aside>
  )
}

/// Sits at the very bottom of the sidebar. POSTs to /logout via an inline
/// form so we don't need any client JS — the form's redirect-after-submit
/// lands the user on /login with the session cookie cleared.
function LogoutLink() {
  return () => (
    <form
      method="post"
      action="/logout"
      mix={css({
        margin: 0,
        padding: `${space[2]} ${space[3]}`,
        borderTop: `1px solid ${color.divider}`,
      })}
    >
      <button
        type="submit"
        mix={css({
          width: '100%',
          display: 'flex',
          alignItems: 'center',
          gap: space[3],
          padding: `${space[2]} ${space[3]}`,
          background: 'transparent',
          border: 'none',
          borderRadius: radius.md,
          color: color.textMuted,
          fontSize: font.base,
          fontWeight: 500,
          fontFamily: font.sans,
          cursor: 'pointer',
          textAlign: 'left',
          transition: 'background 120ms ease, color 120ms ease',
          '&:hover': {
            background: color.hover,
            color: color.danger,
          },
        })}
      >
        <Icon svg={LogOut} size={18} />
        Sign out
      </button>
    </form>
  )
}

function Brand() {
  return () => (
    <div
      mix={css({
        padding: `0 ${space[5]}`,
        display: 'flex',
        alignItems: 'center',
        gap: space[2],
      })}
    >
      {/* Logo mark: filled brand-color square with a stylized P. Keeps the
          dependency footprint at zero (no extra asset to host). */}
      <div
        mix={css({
          width: '28px',
          height: '28px',
          borderRadius: radius.md,
          background: `linear-gradient(135deg, ${color.brand}, ${color.brandHover})`,
          display: 'inline-flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#fff',
          fontWeight: 700,
          fontSize: font.md,
          letterSpacing: '-0.02em',
        })}
      >
        P
      </div>
      <span
        mix={css({
          fontSize: font.lg,
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

function NavDivider() {
  return ({ label }: { label: string }) => (
    <li
      mix={css({
        ...labelStyle,
        marginTop: space[5],
        marginBottom: space[1],
        padding: `0 ${space[5]}`,
        // Empty-label dividers render as spacers — useful before "Audit".
        minHeight: label ? undefined : space[3],
      })}
    >
      {label}
    </li>
  )
}

function NavLink() {
  return ({
    route,
    label,
    icon,
  }: {
    route: NavRoute
    label: string
    icon: string
  }) => (
    <li>
      <a
        href={routes[route].href({} as never)}
        mix={css({
          display: 'flex',
          alignItems: 'center',
          gap: space[3],
          padding: `${space[2]} ${space[5]}`,
          color: color.textMuted,
          textDecoration: 'none',
          fontSize: font.base,
          fontWeight: 500,
          borderLeft: `3px solid transparent`,
          transition: 'background 120ms ease, color 120ms ease',
          '&:hover': {
            background: color.hover,
            color: color.text,
          },
        })}
      >
        <Icon svg={icon} size={18} />
        {label}
      </a>
    </li>
  )
}

// ── Country chips ────────────────────────────────────────────────────────────

interface CountryChipsProps {
  selected: string
  options: string[]
  locale?: string
}

function CountryChips() {
  return ({ selected, options, locale }: CountryChipsProps) => (
    <ChipGroup label="Country">
      {options.map((c) => (
        <ChipLink
          href={buildHref({ country: c, locale })}
          active={c === selected}
          label={c}
        />
      ))}
    </ChipGroup>
  )
}

interface LocaleChipsProps {
  selected: Locale
  country?: string
}

function LocaleChips() {
  return ({ selected, country }: LocaleChipsProps) => (
    <ChipGroup label="Language">
      {LOCALES.map((l) => (
        <ChipLink
          href={buildHref({ locale: l, country })}
          active={l === selected}
          label={LOCALE_LABELS[l]}
        />
      ))}
    </ChipGroup>
  )
}

function ChipGroup() {
  return ({ label, children }: { label: string; children: RemixNode }) => (
    <div
      mix={css({
        display: 'inline-flex',
        alignItems: 'center',
        gap: space[2],
        padding: `${space[1]} ${space[2]} ${space[1]} ${space[3]}`,
        background: color.surface,
        border: `1px solid ${color.border}`,
        borderRadius: radius.pill,
        boxShadow: shadow.card,
      })}
    >
      <span mix={css({ ...labelStyle })}>{label}</span>
      <div
        mix={css({
          display: 'inline-flex',
          gap: space[1],
          background: color.bg,
          padding: '3px',
          borderRadius: radius.pill,
        })}
      >
        {children}
      </div>
    </div>
  )
}

function ChipLink() {
  return ({ href, active, label }: { href: string; active: boolean; label: string }) => (
    <a
      href={href}
      mix={css({
        padding: `${space[1]} ${space[3]}`,
        fontSize: font.sm,
        fontWeight: 600,
        borderRadius: radius.pill,
        textDecoration: 'none',
        color: active ? color.text : color.textMuted,
        background: active ? color.surface : 'transparent',
        boxShadow: active ? shadow.card : 'none',
        transition: 'background 120ms ease, color 120ms ease',
        '&:hover': active ? undefined : { color: color.text },
      })}
    >
      {label}
    </a>
  )
}

/// Build a relative href preserving country + locale together. Switching one
/// chip keeps the other intact.
function buildHref(params: { country?: string; locale?: string }): string {
  let qs = new URLSearchParams()
  if (params.country) qs.set('country', params.country)
  if (params.locale) qs.set('locale', params.locale)
  let s = qs.toString()
  return s ? `?${s}` : '?'
}

// ── Reusable primitives ──────────────────────────────────────────────────────

export function Card() {
  return ({
    children,
    padding,
    border = true,
  }: {
    children: RemixNode
    /// Override interior padding. Defaults to `space[5]` (20px).
    padding?: string
    /// Set to false for a borderless card useful when nesting inside another
    /// card or table.
    border?: boolean
  }) => (
    <div
      mix={css({
        background: color.surface,
        borderRadius: radius.lg,
        padding: padding ?? space[5],
        border: border ? `1px solid ${color.border}` : undefined,
        boxShadow: shadow.card,
      })}
    >
      {children}
    </div>
  )
}

/// "Quick stat" tile — uppercase label + big value + optional caption.
/// Use inside a grid for dashboard-style headers.
export function Stat() {
  return ({
    label,
    value,
    caption,
    trend,
  }: {
    label: string
    value: RemixNode
    caption?: string
    /// Optional positive/negative tint applied to the value text. Pass the
    /// sign of the underlying number.
    trend?: 'up' | 'down' | 'flat'
  }) => {
    let valueColor =
      trend === 'up' ? color.success : trend === 'down' ? color.danger : color.text
    return (
      <Card>
        <div mix={css({ ...labelStyle, marginBottom: space[2] })}>{label}</div>
        <div
          mix={css({
            fontSize: font.xxl,
            fontWeight: 700,
            color: valueColor,
            lineHeight: 1.1,
            letterSpacing: '-0.02em',
          })}
        >
          {value}
        </div>
        {caption && (
          <div
            mix={css({
              marginTop: space[1],
              fontSize: font.xs,
              color: color.textMuted,
            })}
          >
            {caption}
          </div>
        )}
      </Card>
    )
  }
}

/// Section heading used inside cards / above tables. Uppercase track-wide
/// label, optional right-side hint.
export function SectionTitle() {
  return ({ children, hint }: { children: RemixNode; hint?: string }) => (
    <div
      mix={css({
        display: 'flex',
        alignItems: 'baseline',
        justifyContent: 'space-between',
        marginBottom: space[3],
      })}
    >
      <h3
        mix={css({
          margin: 0,
          ...labelStyle,
        })}
      >
        {children}
      </h3>
      {hint && (
        <span mix={css({ fontSize: font.xs, color: color.textDim })}>{hint}</span>
      )}
    </div>
  )
}

export type BadgeTone =
  | 'neutral'
  | 'brand'
  | 'success'
  | 'danger'
  | 'warn'
  | 'info'

const BADGE_TONES: Record<BadgeTone, { bg: string; fg: string }> = {
  neutral: { bg: color.borderSoft, fg: color.textMuted },
  brand: { bg: color.brandSoft, fg: color.brandSoftText },
  success: { bg: color.successSoft, fg: color.successText },
  danger: { bg: color.dangerSoft, fg: color.dangerText },
  warn: { bg: color.warnSoft, fg: color.warnText },
  info: { bg: color.infoSoft, fg: color.infoText },
}

/// Small tag/pill for status, type, sentiment, etc. Pick a tone that
/// communicates the meaning at a glance.
export function Badge() {
  return ({
    children,
    tone = 'neutral',
    title,
  }: {
    children: RemixNode
    tone?: BadgeTone
    title?: string
  }) => {
    let { bg, fg } = BADGE_TONES[tone]
    return (
      <span
        title={title}
        mix={css({
          display: 'inline-flex',
          alignItems: 'center',
          gap: space[1],
          padding: `2px ${space[2]}`,
          background: bg,
          color: fg,
          borderRadius: radius.pill,
          fontSize: font.xs,
          fontWeight: 600,
          whiteSpace: 'nowrap',
        })}
      >
        {children}
      </span>
    )
  }
}

/// Hashes a stock symbol to a deterministic pastel for the avatar.
const AVATAR_PALETTE = [
  '#fbbf24', // amber
  '#f87171', // red
  '#34d399', // emerald
  '#60a5fa', // blue
  '#a78bfa', // violet
  '#f472b6', // pink
  '#22d3ee', // cyan
  '#fb923c', // orange
] as const

/// Colored circle with stock ticker initials. Placeholder for a real logo;
/// looks much friendlier than `#23` in lists.
export function StockBadge() {
  return ({ symbol, size = 28 }: { symbol: string; size?: number }) => {
    // Deterministic hash so the same symbol always gets the same color.
    let hash = 0
    for (let i = 0; i < symbol.length; i++) hash = (hash * 31 + symbol.charCodeAt(i)) | 0
    let bg = AVATAR_PALETTE[Math.abs(hash) % AVATAR_PALETTE.length]
    let initials = symbol.slice(0, 2).toUpperCase()
    return (
      <span
        mix={css({
          width: `${size}px`,
          height: `${size}px`,
          borderRadius: radius.pill,
          background: bg,
          color: '#fff',
          display: 'inline-flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: `${Math.max(10, size * 0.4)}px`,
          fontWeight: 700,
          letterSpacing: '-0.02em',
          flexShrink: 0,
        })}
      >
        {initials}
      </span>
    )
  }
}

/// Use inside cards / pages when there's no data to show. Single source of
/// truth so the empty experience feels consistent.
export function EmptyState() {
  return ({
    title,
    hint,
  }: {
    title: string
    /// Optional explanation or call to action — accepts JSX so you can drop
    /// in `<code>` for API hints.
    hint?: RemixNode
  }) => (
    <div
      mix={css({
        padding: `${space[8]} ${space[5]}`,
        textAlign: 'center',
        color: color.textMuted,
      })}
    >
      <div mix={css({ fontSize: font.md, fontWeight: 600, color: color.text })}>
        {title}
      </div>
      {hint && (
        <div mix={css({ marginTop: space[2], fontSize: font.sm })}>{hint}</div>
      )}
    </div>
  )
}

// Re-export tokens so consumers can import a single module for everything.
export { color, font, labelStyle, radius, shadow, space } from './tokens.ts'
