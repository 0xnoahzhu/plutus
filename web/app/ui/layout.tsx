import { css, type RemixNode } from 'remix/ui'

import {
  ambientAllowedCountries,
  ambientUnreadCounts,
  type EntityKind,
} from '../api.ts'
import {
  ArrowLeftRight,
  BarChart3,
  Bookmark,
  Building2,
  ChartCandlestick,
  ClipboardCheck,
  FileText,
  Filter,
  GitMerge,
  Globe,
  Key,
  LayoutDashboard,
  LogOut,
  Moon,
  Monitor,
  Newspaper,
  Receipt,
  ScrollText,
  SearchCheck,
  Settings,
  Sun,
  Target,
  ThumbsUp,
  TrendingUp,
  Wallet,
  Zap,
} from 'lucide-static'

import { messages, type Messages } from '../i18n/messages.ts'
import { routes } from '../routes.ts'

import { Document } from './document.tsx'
import { Icon } from './icon.tsx'
import { color, font, labelStyle, radius, shadow, space } from './tokens.ts'

export interface LayoutProps {
  children?: RemixNode
  title?: string
  /// Optional subtitle shown right under the title — useful for context like
  /// a date range or row count. Plain text only. Keep it short — this lives
  /// in the sticky header. For longer page-intro copy, render a `<p>` at
  /// the top of `children` (it'll scroll with the content, where it belongs).
  subtitle?: string
  /// Optional page-level navigation that should pin alongside the title —
  /// the right home for tabs ("Items / Daily / Weekly" on /watchlists) and
  /// any other top-of-page switcher you want visible while scrolling. Lives
  /// inside the sticky header below the country chip row.
  nav?: RemixNode
  /// Currently-selected country (ISO alpha-2). When provided, the country
  /// chip group renders in the filter row.
  country?: string
  /// Resolved locale for this request — "en" or "zh-CN". Used both for
  /// content rendering and for the language chip group.
  locale: string
  /// Resolved color-scheme — "system" | "dark" | "light". Drives the theme
  /// chip group and the Document's `data-theme` attribute.
  theme?: Theme
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
  | {
      kind: 'link'
      route: NavRoute
      label: string
      icon: string
      /// Entity kind whose unread count drives the badge on this row.
      /// `undefined` means the row never shows a count (e.g. holdings,
      /// settings).
      unreadKind?: EntityKind
    }
  | { kind: 'divider'; label: string }

const link = (
  route: NavRoute,
  label: string,
  icon: string,
  unreadKind?: EntityKind,
): NavEntry => ({
  kind: 'link',
  route,
  label,
  icon,
  unreadKind,
})
const divider = (label: string): NavEntry => ({ kind: 'divider', label })

/// Build the localized sidebar nav for one render. Labels come from the
/// i18n messages table; icons + route refs stay constant across locales.
function buildNav(m: Messages): NavEntry[] {
  return [
    link('home', m.nav.dashboard, LayoutDashboard),
    link('holdings', m.nav.holdings, Wallet),
    link('stocks', m.nav.stocks, TrendingUp),
    link('transactions', m.nav.transactions, ArrowLeftRight),
    link('watchlists', m.nav.watchlist, Bookmark),
    divider(m.nav.sectionCalendar),
    link('news', m.nav.news, Newspaper, 'news'),
    link('briefs', m.nav.briefs, FileText, 'market_brief'),
    link('earnings', m.nav.earnings, BarChart3, 'earnings_event'),
    link('macroEvents', m.nav.macro, Globe, 'macro_event'),
    link('catalysts', m.nav.catalysts, Zap, 'catalyst'),
    divider(m.nav.sectionAnalysis),
    link('screeners', m.nav.screeners, Filter, 'screener_run'),
    link('recommendations', m.nav.recommendations, ThumbsUp, 'recommendation'),
    link('portfolioReviews', m.nav.reviews, ClipboardCheck, 'portfolio_review'),
    link('correlations', m.nav.correlations, GitMerge, 'correlation_run'),
    link('selfExams', m.nav.selfExam, SearchCheck, 'self_exam'),
    divider(m.nav.sectionAccount),
    link('tradePlans', m.nav.tradePlans, Target),
    link('orders', m.nav.orders, Receipt),
    link('apiKeys', m.nav.apiKeys, Key),
    link('accounts', m.nav.accounts, Building2),
    divider(''),
    link('audit', m.nav.audit, ScrollText),
    link('settings', m.nav.settings, Settings),
  ]
}

// ── Country / locale ─────────────────────────────────────────────────────────

// Country → list of `stocks.market_code` values covered. Stocks carry a
// `market_code` and we map back to a country here to decide whether a
// row passes the country filter.
//
// Two conventions coexist in the data: the canonical MIC codes
// (`XNAS`, `XNYS`, `XHKG`, `XSHG`, `XSHE`) — present in the `markets`
// reference table — and the lowercase pseudo-codes the `stocks` table
// actually stores (`us`, `us_etf`, `us_adr`, `hk`, `hk_etf`, `cn_a`,
// `cn_etf`). Listing both means the filter works regardless of which
// convention the row was written with. New rows should prefer the
// lowercase pseudo-codes — the MIC entries are kept for back-compat.
export const COUNTRY_TO_MARKETS: Record<string, string[]> = {
  US: ['XNAS', 'XNYS', 'us', 'us_etf', 'us_adr'],
  HK: ['XHKG', 'hk', 'hk_etf'],
  CN: ['XSHG', 'XSHE', 'cn_a', 'cn_etf'],
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

/// Read the `country` query parameter. Always returns one country —
/// unknown or out-of-scope values fall back to the first country the
/// caller is allowed to see.
///
/// The allowed list is read from the per-request AsyncLocalStorage
/// populated by `withAuth` (from `/auth/me`). Callers can override
/// explicitly by passing `allowed`; admin / preboot code paths can
/// pass `ALL_COUNTRIES` to get the global behavior.
export function parseCountry(
  search: URLSearchParams,
  allowed?: readonly string[],
): string {
  let effective = allowed ?? ambientAllowedCountries() ?? ALL_COUNTRIES
  // Empty allowed list means "no scope" (admin actor) — treat as ALL.
  let pool = effective.length > 0 ? effective : ALL_COUNTRIES
  let c = search.get('country')
  if (c && pool.includes(c)) return c
  // Fall back to the user's first allowed country (or DEFAULT_COUNTRY
  // if it's in their list, for stability across upgrades).
  return pool.includes(DEFAULT_COUNTRY) ? DEFAULT_COUNTRY : pool[0]
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

// ── Theme ────────────────────────────────────────────────────────────────────

/// Color-scheme choices. `system` follows the browser's
/// `prefers-color-scheme`; `dark`/`light` pin the palette.
export const THEMES = ['system', 'dark', 'light'] as const
export type Theme = (typeof THEMES)[number]

export const THEME_LABELS: Record<Theme, string> = {
  system: 'System',
  dark: 'Dark',
  light: 'Light',
}

export const DEFAULT_THEME: Theme = 'system'

const THEME_COOKIE = 'plutus_theme'

function isTheme(v: string | null): v is Theme {
  return v === 'system' || v === 'dark' || v === 'light'
}

export function parseTheme(search: URLSearchParams): Theme {
  let t = search.get('theme')
  return isTheme(t) ? t : DEFAULT_THEME
}

/// `?theme=` > cookie > `system`.
export function resolveTheme(request: Request, search: URLSearchParams): Theme {
  let q = search.get('theme')
  if (isTheme(q)) return q

  let cookie = request.headers.get('cookie') ?? ''
  for (let part of cookie.split(';')) {
    let [k, v] = part.split('=').map((s) => s.trim())
    if (k === THEME_COOKIE && isTheme(v)) return v
  }

  return DEFAULT_THEME
}

export function themeCookie(theme: Theme): string {
  return `${THEME_COOKIE}=${theme}; Path=/; Max-Age=31536000; SameSite=Lax`
}

// ── Layout shell ─────────────────────────────────────────────────────────────

const SIDEBAR_WIDTH = '240px'

export function Layout() {
  return ({
    title,
    subtitle,
    nav,
    children,
    country,
    locale,
    theme = DEFAULT_THEME,
  }: LayoutProps) => (
    <Document
      title={title ? `${title} · Plutus` : 'Plutus'}
      lang={locale}
      theme={theme}
    >
      <div
        mix={css({
          display: 'grid',
          gridTemplateColumns: `${SIDEBAR_WIDTH} 1fr`,
          minHeight: '100vh',
          background: color.bg,
        })}
      >
        <Sidebar locale={locale} />
        <main
          mix={css({
            // Horizontal padding only — vertical padding lives on the
            // sticky header (top) and the children wrapper (bottom) so
            // the header's background can reach the full viewport edge
            // when pinned.
            //
            // No `maxWidth`: previously capped at 1400/1800px, which
            // left an empty strip on the right of wide monitors. Main
            // now fills the full space to the right of the sidebar.
            // Long-form text inside still reads fine because detail
            // pages use a 2:1 grid that pulls the markdown column to
            // ~⅔ of the available width.
            padding: `0 ${space[10]}`,
            width: '100%',
            minWidth: 0,
            '@media (max-width: 1100px)': {
              padding: `0 ${space[6]}`,
            },
          })}
        >
          {(title !== undefined || country !== undefined || nav !== undefined) && (
            <div
              mix={css({
                // Pin the title block + country filter (+ optional nav)
                // to the viewport top as the page scrolls. The wrapper
                // owns the top padding (previously on <main>) so at
                // scroll-top the visual rhythm is unchanged; once stuck,
                // the same padding becomes the breathing room between
                // viewport edge and the title.
                position: 'sticky',
                top: 0,
                zIndex: 10,
                background: color.bg,
                paddingTop: space[8],
                paddingBottom: space[4],
                marginBottom: space[6],
                // Hairline separator so content scrolling underneath
                // doesn't visually merge with the pinned header. Same
                // divider used elsewhere, so it feels native.
                borderBottom: `1px solid ${color.divider}`,
                '@media (max-width: 1100px)': {
                  paddingTop: space[6],
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
                    // Gap to whatever follows (country / nav). When neither
                    // is present the wrapper's paddingBottom owns the gap.
                    marginBottom:
                      country !== undefined || nav !== undefined ? space[4] : 0,
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

              {/* Page-level filter row — only the country chip lives here.
                  Language + theme moved to /settings. */}
              {country !== undefined && (
                <div
                  mix={css({
                    display: 'flex',
                    gap: space[6],
                    flexWrap: 'wrap',
                    alignItems: 'center',
                    // Gap to nav below when both present.
                    marginBottom: nav !== undefined ? space[4] : 0,
                  })}
                >
                  <CountryChips
                    selected={country}
                    options={(() => {
                      let amb = ambientAllowedCountries()
                      return amb && amb.length > 0 ? amb : ALL_COUNTRIES
                    })()}
                    locale={locale}
                  />
                </div>
              )}

              {/* Page-level nav slot — tabs and similar switchers go here so
                  they stay visible while the user scrolls through content. */}
              {nav}
            </div>
          )}

          <div
            mix={css({
              // Top padding only matters when no sticky header exists
              // (e.g. login-like routes that go through Layout without
              // a title). Bottom padding always — so the page breathes
              // at the very end of scroll.
              paddingTop:
                title === undefined &&
                country === undefined &&
                nav === undefined
                  ? space[8]
                  : 0,
              paddingBottom: space[8],
              '@media (max-width: 1100px)': {
                paddingTop:
                  title === undefined &&
                  country === undefined &&
                  nav === undefined
                    ? space[6]
                    : 0,
                paddingBottom: space[6],
              },
            })}
          >
            {children}
          </div>
        </main>
      </div>
    </Document>
  )
}

// ── Sidebar ──────────────────────────────────────────────────────────────────

function Sidebar() {
  return ({ locale }: { locale: string }) => {
    let m = messages(locale)
    let nav = buildNav(m)
    let counts = ambientUnreadCounts()
    return (
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
        <div mix={css({ padding: `0 ${space[5]}` })}>
          <BrandMark />
        </div>
        <nav mix={css({ marginTop: space[6], flex: 1 })}>
          <ul mix={css({ listStyle: 'none', padding: 0, margin: 0 })}>
            {nav.map((entry) =>
              entry.kind === 'divider' ? <NavDivider label={entry.label} /> : (
                <NavLink
                  route={entry.route}
                  label={entry.label}
                  icon={entry.icon}
                  badge={
                    entry.unreadKind ? (counts[entry.unreadKind] ?? 0) : 0
                  }
                />
              ),
            )}
          </ul>
        </nav>
        <LogoutLink label={m.nav.signOut} />
      </aside>
    )
  }
}

/// Sits at the very bottom of the sidebar. POSTs to /logout via an inline
/// form so we don't need any client JS — the form's redirect-after-submit
/// lands the user on /login with the session cookie cleared.
function LogoutLink() {
  return ({ label }: { label: string }) => (
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
        {label}
      </button>
    </form>
  )
}

interface BrandMarkProps {
  /// Pixel size of the gradient icon tile. Wordmark scales with it.
  /// Defaults to 28 (the sidebar Brand size).
  size?: number
}

/// The plutus wordmark: a candlestick-chart glyph inside a teal-gradient
/// tile, paired with the "Plutus" text. Exported so the login page reuses
/// the exact same mark.
export function BrandMark() {
  return ({ size = 28 }: BrandMarkProps) => {
    let textSize = size <= 28 ? font.lg : font.xl
    let iconSize = Math.round(size * 0.6)
    return (
      <div
        mix={css({
          display: 'flex',
          alignItems: 'center',
          gap: space[2],
        })}
      >
        <div
          mix={css({
            width: `${size}px`,
            height: `${size}px`,
            borderRadius: radius.md,
            background: `linear-gradient(135deg, ${color.brand}, ${color.brandHover})`,
            color: '#fff',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            flexShrink: 0,
          })}
        >
          <Icon svg={ChartCandlestick} size={iconSize} />
        </div>
        <span
          mix={css({
            fontSize: textSize,
            fontWeight: 700,
            color: color.text,
            letterSpacing: '-0.02em',
          })}
        >
          Plutus
        </span>
      </div>
    )
  }
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
    badge,
  }: {
    route: NavRoute
    label: string
    icon: string
    /// Unread count for this row. Zero (or undefined) hides the chip.
    /// 99+ collapses to "99+" so a runaway count doesn't break layout.
    badge?: number
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
        <span mix={css({ flex: 1 })}>{label}</span>
        {badge !== undefined && badge > 0 && (
          <span
            mix={css({
              marginLeft: 'auto',
              minWidth: '18px',
              padding: `1px ${space[2]}`,
              fontSize: '11px',
              fontWeight: 600,
              fontVariantNumeric: 'tabular-nums',
              textAlign: 'center',
              background: color.borderSoft,
              color: color.text,
              borderRadius: radius.pill,
              lineHeight: 1.4,
            })}
          >
            {badge > 99 ? '99+' : badge}
          </span>
        )}
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

export function LocaleChips() {
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

interface ThemeChipsProps {
  selected: Theme
  country?: string
  locale?: string
}

const THEME_ICONS: Record<Theme, string> = {
  system: Monitor,
  dark: Moon,
  light: Sun,
}

export function ThemeChips() {
  return ({ selected, country, locale }: ThemeChipsProps) => (
    <ChipGroup label="Theme">
      {THEMES.map((t) => (
        <ChipLink
          href={buildHref({ country, locale, theme: t })}
          active={t === selected}
          label={THEME_LABELS[t]}
          icon={THEME_ICONS[t]}
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

interface ChipLinkProps {
  href: string
  active: boolean
  label: string
  /// Optional lucide SVG string rendered before the label. Used for the
  /// theme chips so each row carries its own glyph.
  icon?: string
}

function ChipLink() {
  return ({ href, active, label, icon }: ChipLinkProps) => (
    <a
      href={href}
      mix={css({
        display: 'inline-flex',
        alignItems: 'center',
        gap: space[1],
        padding: `${space[1]} ${space[3]}`,
        fontSize: font.sm,
        fontWeight: 600,
        borderRadius: radius.pill,
        textDecoration: 'none',
        // Active state: white pill + dark text on the inset gray track.
        // Reads clearly without the previous "slate-900 fill" looking
        // overdone next to the rest of the chrome.
        color: active ? color.text : color.textMuted,
        background: active ? color.surface : 'transparent',
        boxShadow: active ? shadow.card : 'none',
        transition: 'background 120ms ease, color 120ms ease',
        '&:hover': active
          ? undefined
          : { color: color.text },
      })}
    >
      {icon && <Icon svg={icon} size={14} />}
      {label}
    </a>
  )
}

/// Build a relative href preserving country + locale + theme together so
/// flipping one chip doesn't reset the others.
function buildHref(params: {
  country?: string
  locale?: string
  theme?: Theme
}): string {
  let qs = new URLSearchParams()
  if (params.country) qs.set('country', params.country)
  if (params.locale) qs.set('locale', params.locale)
  if (params.theme) qs.set('theme', params.theme)
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
        <div mix={css({ textAlign: 'center' })}>
          <div mix={css({ ...labelStyle, marginBottom: space[2] })}>{label}</div>
          <div
            mix={css({
              fontSize: font.xxl,
              fontWeight: 700,
              color: valueColor,
              lineHeight: 1.1,
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
        </div>
      </Card>
    )
  }
}

/// One-or-two-sentence page description rendered just under the sticky
/// header. Same muted-small treatment portfolio-reviews used; lifted here
/// so every page that needs a long intro picks up the same style without
/// reaching for raw `<p mix={...}>` each time.
///
/// Why this is a separate component (and not Layout's `subtitle`):
/// `subtitle` lives in the sticky header and should stay short ("12
/// reviews", "Today's snapshot"). Long descriptive copy goes here so it
/// scrolls with the content and doesn't bloat the pinned area.
export function PageIntro() {
  return ({ children }: { children: RemixNode }) => (
    <p
      mix={css({
        fontSize: font.sm,
        color: color.textMuted,
        marginTop: 0,
        marginBottom: space[4],
        lineHeight: 1.55,
      })}
    >
      {children}
    </p>
  )
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

/// Right-aligned strip that renders a `MarkAllReadButton` and disappears
/// entirely (no leftover margin) when the kind's unread count is zero.
/// Drop this near the top of every list page body.
export function MarkAllReadStrip() {
  return ({ kind }: { kind: EntityKind }) => {
    let count = ambientUnreadCounts()[kind] ?? 0
    if (count === 0) return null
    return (
      <div
        mix={css({
          display: 'flex',
          justifyContent: 'flex-end',
          marginBottom: space[3],
        })}
      >
        <MarkAllReadButton kind={kind} />
      </div>
    )
  }
}

/// "Mark all read" button. Submits a form to the per-kind action handler
/// which calls the API and redirects back. Stateless — no JS required.
/// Reads the current unread count from `ambientUnreadCounts()` and
/// shows it in the label; renders nothing when the count is zero so the
/// list header doesn't carry a useless control.
export function MarkAllReadButton() {
  return ({ kind, label }: { kind: EntityKind; label?: string }) => {
    let count = ambientUnreadCounts()[kind] ?? 0
    if (count === 0) return null
    let text = label ?? `Mark ${count} as read`
    return (
      <form
        method="post"
        action={`/reads/mark-all/${kind}`}
        mix={css({ margin: 0, display: 'inline-block' })}
      >
        <button
          type="submit"
          mix={css({
            display: 'inline-flex',
            alignItems: 'center',
            gap: space[1],
            padding: `${space[1]} ${space[3]}`,
            fontSize: font.xs,
            fontWeight: 600,
            fontFamily: font.sans,
            color: color.textMuted,
            background: color.surface,
            border: `1px solid ${color.border}`,
            borderRadius: radius.pill,
            cursor: 'pointer',
            transition: 'background 120ms ease, color 120ms ease, border-color 120ms ease',
            '&:hover': {
              color: color.text,
              borderColor: color.brand,
            },
          })}
        >
          {text}
        </button>
      </form>
    )
  }
}

/// Small filled circle that flags an unread list item. Pass the row's
/// `read_at` — `null` renders the dot, any string hides it. Sits next to
/// the leading metadata in a card so the eye picks it up first.
export function UnreadDot() {
  return ({ readAt, size = 8 }: { readAt: string | null; size?: number }) => {
    if (readAt) return null
    return (
      <span
        title="Unread"
        aria-label="Unread"
        mix={css({
          display: 'inline-block',
          width: `${size}px`,
          height: `${size}px`,
          borderRadius: '50%',
          background: color.brand,
          flexShrink: 0,
        })}
      />
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
