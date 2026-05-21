/// Render a server-side UTC timestamp as the user's local time, falling
/// back to a truncated UTC string when JavaScript is disabled. Emits a
/// semantic `<time datetime="...">` element with a `data-fmt` hint; the
/// inline hydration script in `document.tsx` rewrites the textContent on
/// load using `Intl.DateTimeFormat` with no `timeZone` (= browser default).
///
/// Use only for genuine UTC timestamps (`created_at`, `executed_at`,
/// `published_at`, etc.). Calendar-only dates (`trade_date`, `run_date`,
/// `period_start`, …) are locale-neutral and don't need conversion.
///
/// Three formats:
/// - `date` → `YYYY-MM-DD`
/// - `datetime` → `YYYY-MM-DD HH:MM`
/// - `full` → `YYYY-MM-DD HH:MM:SS`
///
/// All formats use 24-hour time so the visual width is stable across
/// columns. The SSR fallback is a literal `.slice()` on the ISO string so
/// no-JS output matches the JS output in width.

export type LocalTimeFormat = 'date' | 'datetime' | 'full'

export function LocalTime() {
  return ({ value, format = 'date' }: { value: string; format?: LocalTimeFormat }) => {
    let fallback = ssrFallback(value, format)
    return (
      <time datetime={value} data-fmt={format}>
        {fallback}
      </time>
    )
  }
}

/// What the server renders before client-side JS swaps in the local time.
/// Matches the **width** of the formatted client output so the row
/// doesn't jump after hydration.
function ssrFallback(value: string, format: LocalTimeFormat): string {
  if (!value) return ''
  if (format === 'date') return value.slice(0, 10)
  if (format === 'full') return value.slice(0, 19).replace('T', ' ')
  // 'datetime'
  return value.slice(0, 16).replace('T', ' ')
}
