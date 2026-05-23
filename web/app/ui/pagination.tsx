/// Shared chrome for paginated index pages: a search box and a
/// prev / page-jump-input / next pagination bar. Both are plain
/// `<form method="get">` so they work without JS — typing a number
/// in the page-jump input and pressing Enter submits the form,
/// and clicking the Go button does the same thing.
///
/// Each page wires these by passing:
///   - `action` — the page's own URL (e.g. `/stocks`); the form
///     submits as GET to this path so other consumer pages see
///     `?q=&page=` query params they can read in their loader.
///   - `params` — any other query params that need to ride along
///     (e.g. country chip, tab selection). Rendered as hidden
///     inputs so the form preserves them across page jumps.

import { css } from 'remix/ui'

import { messages } from '../i18n/messages.ts'

import { color, font, radius, space } from './tokens.ts'

interface SearchBarProps {
  /// Form action — typically the current page's path.
  action: string
  /// Current query string from `?q=`. Pre-fills the input so the
  /// user can refine an existing search instead of retyping.
  query: string
  /// Locale-translated placeholder; falls back to the project's
  /// default if absent.
  placeholder: string
  locale: string
  /// Extra hidden inputs to keep `?country=`, `?tab=`, etc. alive
  /// across the GET submission. Keyed by query-param name. Skip
  /// the empty-string values; rendering `<input name=x value="">`
  /// would still post the key with an empty value and pollute the
  /// URL.
  extraParams?: Record<string, string | undefined>
}

export function SearchBar() {
  return ({ action, query, placeholder, locale, extraParams }: SearchBarProps) => {
    let m = messages(locale).common
    return (
      <form
        method="get"
        action={action}
        mix={css({
          display: 'flex',
          gap: space[2],
          alignItems: 'center',
          margin: 0,
          // Cap so the search row doesn't stretch to fill a full-width
          // card on wide monitors. A symbol/code search doesn't need
          // more than ~400px of typing space.
          maxWidth: '480px',
          width: '100%',
        })}
      >
        {extraParams &&
          Object.entries(extraParams).map(([k, v]) =>
            v ? <input type="hidden" name={k} value={v} /> : null,
          )}
        <input
          type="text"
          name="q"
          value={query}
          placeholder={placeholder}
          autocomplete="off"
          mix={css({
            flex: 1,
            padding: `${space[2]} ${space[3]}`,
            fontSize: font.base,
            fontFamily: font.sans,
            color: color.text,
            background: color.bg,
            border: `1px solid ${color.border}`,
            borderRadius: radius.md,
            outline: 'none',
            '&:focus': { borderColor: color.brand },
            '&::placeholder': { color: color.textDim },
          })}
        />
        <button
          type="submit"
          mix={css({
            padding: `${space[2]} ${space[4]}`,
            fontSize: font.sm,
            fontWeight: 600,
            color: color.textOnBrand,
            background: color.brand,
            border: 'none',
            borderRadius: radius.md,
            cursor: 'pointer',
            transition: 'background 120ms ease',
            '&:hover': { background: color.brandHover },
          })}
        >
          {m.searchSubmit}
        </button>
        {query !== '' && (
          <a
            href={`${action}${buildQs({ ...extraParams, q: undefined, page: undefined })}`}
            mix={css({
              fontSize: font.sm,
              color: color.textMuted,
              textDecoration: 'none',
              padding: `${space[2]} ${space[3]}`,
              '&:hover': { color: color.text },
            })}
          >
            {m.searchClear}
          </a>
        )}
      </form>
    )
  }
}

interface PaginationProps {
  /// Form action — the page's own path.
  action: string
  /// 1-indexed current page.
  page: number
  /// Total page count derived from total / perPage.
  totalPages: number
  /// Total matching row count, for the "Showing A-B of C" summary.
  total: number
  perPage: number
  /// Current search query (kept across page jumps).
  query: string
  locale: string
  /// Same as SearchBar.extraParams — any other context that needs
  /// to survive the pagination form submission.
  extraParams?: Record<string, string | undefined>
}

export function Pagination() {
  return ({
    action,
    page,
    totalPages,
    total,
    perPage,
    query,
    locale,
    extraParams,
  }: PaginationProps) => {
    let m = messages(locale).common
    let start = total === 0 ? 0 : (page - 1) * perPage + 1
    let end = Math.min(page * perPage, total)
    // Build query string for the prev/next links. Strip out `page`
    // and write a fresh value for each link.
    let prevHref = `${action}${buildQs({ ...extraParams, q: query || undefined, page: String(Math.max(1, page - 1)) })}`
    let nextHref = `${action}${buildQs({ ...extraParams, q: query || undefined, page: String(Math.min(totalPages, page + 1)) })}`
    let pillBase = css({
      display: 'inline-flex',
      alignItems: 'center',
      padding: `${space[2]} ${space[4]}`,
      fontSize: font.sm,
      fontWeight: 600,
      borderRadius: radius.md,
      border: `1px solid ${color.border}`,
      background: color.surface,
      color: color.text,
      textDecoration: 'none',
      transition: 'background 120ms ease',
      '&:hover': { background: color.bg },
    })
    let pillDisabled = css({
      display: 'inline-flex',
      alignItems: 'center',
      padding: `${space[2]} ${space[4]}`,
      fontSize: font.sm,
      fontWeight: 600,
      borderRadius: radius.md,
      border: `1px solid ${color.borderSoft}`,
      background: 'transparent',
      color: color.textDim,
      cursor: 'not-allowed',
    })
    let canPrev = page > 1
    let canNext = page < totalPages
    return (
      <div
        mix={css({
          marginTop: space[4],
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: space[3],
          flexWrap: 'wrap',
        })}
      >
        <span mix={css({ fontSize: font.sm, color: color.textMuted })}>
          {total === 0 ? m.paginationEmpty : m.paginationShowing(start, end, total)}
        </span>
        <div mix={css({ display: 'inline-flex', alignItems: 'center', gap: space[2] })}>
          {canPrev ? (
            <a href={prevHref} mix={pillBase}>
              {m.paginationPrev}
            </a>
          ) : (
            <span mix={pillDisabled}>{m.paginationPrev}</span>
          )}
          {/* Page-jump form. The input is the only visible control;
              hidden inputs carry the existing q and extraParams so
              the submission goes to the same filtered view. */}
          <form
            method="get"
            action={action}
            mix={css({
              display: 'inline-flex',
              alignItems: 'center',
              gap: space[2],
              margin: 0,
            })}
          >
            {extraParams &&
              Object.entries(extraParams).map(([k, v]) =>
                v ? <input type="hidden" name={k} value={v} /> : null,
              )}
            {query && <input type="hidden" name="q" value={query} />}
            <input
              type="number"
              name="page"
              value={String(page)}
              min={1}
              max={totalPages}
              mix={css({
                width: '64px',
                padding: `${space[2]} ${space[2]}`,
                fontSize: font.sm,
                fontFamily: font.mono,
                textAlign: 'center',
                color: color.text,
                background: color.bg,
                border: `1px solid ${color.border}`,
                borderRadius: radius.md,
                outline: 'none',
                '&:focus': { borderColor: color.brand },
              })}
            />
            <span
              mix={css({
                fontSize: font.sm,
                color: color.textMuted,
                whiteSpace: 'nowrap',
              })}
            >
              / {totalPages}
            </span>
            <button
              type="submit"
              mix={css({
                padding: `${space[2]} ${space[3]}`,
                fontSize: font.sm,
                fontWeight: 600,
                color: color.text,
                background: color.surface,
                border: `1px solid ${color.border}`,
                borderRadius: radius.md,
                cursor: 'pointer',
                transition: 'background 120ms ease',
                '&:hover': { background: color.bg },
              })}
            >
              {m.paginationGo}
            </button>
          </form>
          {canNext ? (
            <a href={nextHref} mix={pillBase}>
              {m.paginationNext}
            </a>
          ) : (
            <span mix={pillDisabled}>{m.paginationNext}</span>
          )}
        </div>
      </div>
    )
  }
}

/// Build a `?a=b&c=d` query string from a map. Skips `undefined`
/// and empty values. Returns the leading `?` (or `""` if all
/// values were skipped) so callers can do `${action}${buildQs(...)}`.
function buildQs(params: Record<string, string | undefined>): string {
  let qs = new URLSearchParams()
  for (let [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== '') qs.set(k, v)
  }
  let s = qs.toString()
  return s ? `?${s}` : ''
}
