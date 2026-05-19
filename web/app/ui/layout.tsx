import { css, type RemixNode } from 'remix/ui'

import { routes } from '../routes.ts'
import { Document } from './document.tsx'

export interface LayoutProps {
  children?: RemixNode
  title?: string
  /** Currently-selected country (ISO alpha-2). Only pages that filter on
   *  country pass this — the chip is hidden otherwise. */
  country?: string
  /** Resolved locale for this request — "en" or "zh-CN". Used both for
   *  rendering and as the dropdown's selected option. Compute via
   *  [[resolveLocale]] and pass through. */
  locale: string
}

type NavEntry =
  | { kind: 'link'; route: keyof typeof routes; label: string }
  | { kind: 'divider'; label: string }

const link = (route: keyof typeof routes, label: string): NavEntry => ({
  kind: 'link',
  route,
  label,
})
const divider = (label: string): NavEntry => ({ kind: 'divider', label })

const NAV: NavEntry[] = [
  link('home', 'Dashboard'),
  link('holdings', 'Holdings'),
  link('stocks', 'Stocks'),
  link('transactions', 'Transactions'),
  link('watchlists', 'Watchlists'),
  divider('Calendar'),
  link('news', 'News'),
  link('briefs', 'Briefs'),
  link('earnings', 'Earnings'),
  link('macroEvents', 'Macro'),
  link('catalysts', 'Catalysts'),
  divider('Analysis'),
  link('screeners', 'Screeners'),
  link('recommendations', 'Recommendations'),
  link('portfolioReviews', 'Reviews'),
  link('correlations', 'Correlations'),
  link('selfExams', 'Self-Exam'),
  divider(''),
  link('audit', 'Audit'),
]

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
  let c = search.get('country');
  if (c && ALL_COUNTRIES.includes(c)) return c;
  return DEFAULT_COUNTRY;
}

/// Supported UI locales. Source language for stored content is English;
/// zh-CN comes from the translations JSON map on each agent-output row.
export const LOCALES = ['en', 'zh-CN'] as const
export type Locale = (typeof LOCALES)[number]

/// Labels for the language dropdown.
export const LOCALE_LABELS: Record<Locale, string> = {
  en: 'English',
  'zh-CN': '中文',
}

/// Fallback when nothing else (URL / cookie / Accept-Language) gives us a
/// usable locale. English because the canonical content stored on most rows
/// is English; only ever fires when the browser declines to advertise a
/// language at all.
export const DEFAULT_LOCALE: Locale = 'en'

/// Name of the persistence cookie. One year.
const LOCALE_COOKIE = 'plutus_locale'

function isLocale(v: string | null): v is Locale {
  return v === 'en' || v === 'zh-CN'
}

/// Read the `locale` query parameter. Used by chip components and tests.
export function parseLocale(search: URLSearchParams): Locale {
  let l = search.get('locale')
  return isLocale(l) ? l : DEFAULT_LOCALE
}

/// Resolve the locale for this request: `?locale=` > cookie > Accept-Language
/// > English. First visit picks up the browser language automatically; once
/// the user clicks a real option, the cookie pins it.
export function resolveLocale(request: Request, search: URLSearchParams): Locale {
  // 1. Explicit URL override.
  let q = search.get('locale')
  if (isLocale(q)) return q

  // 2. Saved choice.
  let cookie = request.headers.get('cookie') ?? ''
  for (let part of cookie.split(';')) {
    let [k, v] = part.split('=').map((s) => s.trim())
    if (k === LOCALE_COOKIE && isLocale(v)) return v
  }

  // 3. Best-effort Accept-Language sniff. Any zh-* variant → zh-CN; otherwise
  //    English. A real BCP-47 parser isn't worth the bytes for two languages.
  let al = (request.headers.get('accept-language') ?? '').toLowerCase()
  if (al.includes('zh')) return 'zh-CN'

  return DEFAULT_LOCALE
}

/// `Set-Cookie` header for persisting the resolved locale. Path=/ so every
/// page reuses it; SameSite=Lax so cross-site nav doesn't strip it.
export function localeCookie(locale: Locale): string {
  return `${LOCALE_COOKIE}=${locale}; Path=/; Max-Age=31536000; SameSite=Lax`
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

export function Layout() {
  return ({ title, children, country, locale }: LayoutProps) => (
    <Document title={title ? `${title} · plutus` : 'plutus'} lang={locale}>
      <div
        mix={css({
          display: 'grid',
          gridTemplateColumns: '220px 1fr',
          minHeight: '100vh',
          fontFamily: "ui-sans-serif, system-ui, sans-serif",
          color: '#1f2937',
          background: '#f8fafc',
        })}
      >
        <aside
          mix={css({
            background: '#0f172a',
            color: '#cbd5e1',
            padding: '24px 16px',
            borderRight: '1px solid #1e293b',
          })}
        >
          <h1
            mix={css({
              fontSize: '20px',
              fontWeight: 700,
              color: '#f8fafc',
              marginBottom: '24px',
              letterSpacing: '0.05em',
            })}
          >
            plutus
          </h1>
          <nav>
            <ul mix={css({ listStyle: 'none', padding: 0, margin: 0 })}>
              {NAV.map((entry) =>
                entry.kind === 'divider' ? (
                  <li
                    mix={css({
                      marginTop: '16px',
                      marginBottom: '4px',
                      padding: '0 12px',
                      fontSize: '10px',
                      textTransform: 'uppercase',
                      letterSpacing: '0.1em',
                      color: '#64748b',
                      minHeight: entry.label ? undefined : '8px',
                    })}
                  >
                    {entry.label}
                  </li>
                ) : (
                  <li mix={css({ marginBottom: '4px' })}>
                    <a
                      href={routes[entry.route].href({} as never)}
                      mix={css({
                        display: 'block',
                        padding: '8px 12px',
                        color: '#cbd5e1',
                        textDecoration: 'none',
                        borderRadius: '6px',
                        fontSize: '14px',
                        transition: 'background 120ms ease',
                        '&:hover': { background: '#1e293b', color: '#f8fafc' },
                      })}
                    >
                      {entry.label}
                    </a>
                  </li>
                ),
              )}
            </ul>
          </nav>
          <div
            mix={css({
              marginTop: '32px',
              fontSize: '11px',
              color: '#64748b',
              padding: '12px',
              borderTop: '1px solid #1e293b',
            })}
          >
            single-user mode
          </div>
        </aside>
        <main mix={css({ padding: '32px 40px' })}>
          {title && (
            <h2
              mix={css({
                fontSize: '24px',
                fontWeight: 600,
                margin: '0 0 24px',
                color: '#0f172a',
              })}
            >
              {title}
            </h2>
          )}
          {/* Per-page filter row: country chip (if applicable) + language
              chips. Both live below the title so the user sees them in the
              context of the current page rather than as a global header. */}
          <div
            mix={css({
              display: 'flex',
              gap: '16px',
              flexWrap: 'wrap',
              alignItems: 'center',
            })}
          >
            {country !== undefined && (
              <CountryChips
                selected={country}
                options={ALL_COUNTRIES}
                locale={locale}
              />
            )}
            <LocaleChips selected={locale as Locale} country={country} />
          </div>
          <div mix={css({ marginTop: '24px' })}>{children}</div>
        </main>
      </div>
    </Document>
  )
}

interface CountryChipsProps {
  selected: string
  options: string[]
  /** Current locale, so chip hrefs preserve `?locale=` when switching country. */
  locale?: string
}

/// Single-select country picker. No "All" — pages always show exactly one
/// country's data.
function CountryChips() {
  return ({ selected, options, locale }: CountryChipsProps) => (
    <div
      mix={css({
        display: 'flex',
        gap: '8px',
        flexWrap: 'wrap',
        alignItems: 'center',
      })}
    >
      <span
        mix={css({
          fontSize: '12px',
          color: '#64748b',
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          marginRight: '4px',
        })}
      >
        Country
      </span>
      {options.map((c) => {
        let active = c === selected
        return (
          <a
            href={buildHref({ country: c, locale })}
            mix={css({
              padding: '4px 12px',
              fontSize: '12px',
              fontWeight: 500,
              borderRadius: '999px',
              textDecoration: 'none',
              background: active ? '#0f172a' : '#e2e8f0',
              color: active ? '#fff' : '#475569',
              '&:hover': { background: active ? '#0f172a' : '#cbd5e1' },
            })}
          >
            {c}
          </a>
        )
      })}
    </div>
  )
}

interface LocaleChipsProps {
  /** Currently-selected locale ("en" or "zh-CN"). */
  selected: Locale
  /** Current country, preserved when the user switches locale so the country
   *  filter doesn't reset. */
  country?: string
}

/// Single-select language picker. Two chips, anchor-link style, navigates
/// with one click. Pairs visually with [[CountryChips]] when present.
function LocaleChips() {
  return ({ selected, country }: LocaleChipsProps) => (
    <div
      mix={css({
        display: 'flex',
        gap: '8px',
        flexWrap: 'wrap',
        alignItems: 'center',
      })}
    >
      <span
        mix={css({
          fontSize: '12px',
          color: '#64748b',
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          marginRight: '4px',
        })}
      >
        Language
      </span>
      {LOCALES.map((l) => {
        let active = l === selected
        return (
          <a
            href={buildHref({ locale: l, country })}
            mix={css({
              padding: '4px 12px',
              fontSize: '12px',
              fontWeight: 500,
              borderRadius: '999px',
              textDecoration: 'none',
              background: active ? '#0f172a' : '#e2e8f0',
              color: active ? '#fff' : '#475569',
              '&:hover': { background: active ? '#0f172a' : '#cbd5e1' },
            })}
          >
            {LOCALE_LABELS[l]}
          </a>
        )
      })}
    </div>
  )
}

/// Build a relative href that preserves the params we care about. When both
/// country and locale are provided they're encoded together so flipping one
/// chip keeps the other intact.
function buildHref(params: { country?: string; locale?: string }): string {
  let qs = new URLSearchParams()
  if (params.country) qs.set('country', params.country)
  if (params.locale) qs.set('locale', params.locale)
  let s = qs.toString()
  return s ? `?${s}` : '?'
}

export function Card() {
  return ({ children }: { children: RemixNode }) => (
    <div
      mix={css({
        background: '#fff',
        borderRadius: '8px',
        padding: '16px 20px',
        border: '1px solid #e2e8f0',
        boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
      })}
    >
      {children}
    </div>
  )
}

export function Stat() {
  return ({ label, value }: { label: string; value: string }) => (
    <div>
      <div
        mix={css({
          fontSize: '11px',
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          color: '#64748b',
        })}
      >
        {label}
      </div>
      <div
        mix={css({
          fontSize: '20px',
          fontWeight: 600,
          marginTop: '4px',
          color: '#0f172a',
        })}
      >
        {value}
      </div>
    </div>
  )
}
