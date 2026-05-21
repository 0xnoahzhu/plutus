/// Format a monetary value as a string with thousand separators and
/// exactly two decimal places. Accepts either a number or the decimal
/// string the API returns (e.g. `"150.250000"`). `null`, `undefined`,
/// empty string, and non-finite numbers render as `—`.
///
/// Examples:
///   fmtMoney(1234.5)         → "1,234.50"
///   fmtMoney("150.250000")   → "150.25"
///   fmtMoney("-0.01")        → "-0.01"
///   fmtMoney(null)           → "—"
export function fmtMoney(value: string | number | null | undefined): string {
  if (value === null || value === undefined || value === '') return '—'
  let n = typeof value === 'number' ? value : Number.parseFloat(value)
  if (!Number.isFinite(n)) return '—'
  let sign = n < 0 ? '-' : ''
  let abs = Math.abs(n)
  let int = Math.floor(abs)
  let cents = Math.round((abs - int) * 100)
  // Edge case: rounding `0.999` * 100 → 100, which would render as
  // `0.100`. Promote the carry into the integer part.
  if (cents === 100) {
    int += 1
    cents = 0
  }
  let centsStr = cents.toString().padStart(2, '0')
  let intStr = int.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return `${sign}${intStr}.${centsStr}`
}
