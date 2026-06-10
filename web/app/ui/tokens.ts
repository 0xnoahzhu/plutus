/// Design tokens for the plutus UI. Single source of truth for colors,
/// spacing, typography, radii, and shadows. Components reach into these via
/// `import { color, space, ... } from './tokens.ts'`.
///
/// Colors are exposed as CSS-variable strings (`var(--color-bg)`) so a
/// single `<style>` block in [[Document]] can swap the underlying palette
/// for dark mode without each component knowing anything about themes.
///
/// The visual language is neumorphic: one monochromatic surface per theme,
/// depth expressed through paired light/dark shadows instead of borders.
/// Raised = interactive or new; inset = tracks, wells, and pressed states.

/// Neutrals — slate ramp. 50 (lightest) → 950 (darkest). Mirrors Tailwind's
/// slate scale so values feel familiar. Kept for one-off needs (avatars,
/// charts); the theme palettes below use the stone surface values directly.
const slate = {
  50: '#f8fafc',
  100: '#f1f5f9',
  200: '#e2e8f0',
  300: '#cbd5e1',
  400: '#94a3b8',
  500: '#64748b',
  600: '#475569',
  700: '#334155',
  800: '#1e293b',
  900: '#0f172a',
  950: '#020617',
} as const

/// Brand — deep teal anchored on #006666 (light) / #00BFB3 (dark).
const teal = {
  50: '#e6f2f2',
  100: '#c2e0e0',
  200: '#8cc6c5',
  300: '#4daaa8',
  400: '#00bfb3',
  500: '#008f87',
  600: '#006666',
  700: '#005252',
  800: '#003d3d',
  900: '#002929',
} as const

/// Each semantic color is a CSS variable. The variable's *value* is set by
/// [[lightPalette]] / [[darkPalette]] below and injected into the document
/// by [[buildThemeCSS]].
export const color = {
  // Surfaces
  bg: 'var(--color-bg)',
  surface: 'var(--color-surface)',
  sidebar: 'var(--color-sidebar)',
  hover: 'var(--color-hover)',
  divider: 'var(--color-divider)',
  border: 'var(--color-border)',
  borderSoft: 'var(--color-border-soft)',
  /// Translucent top-light edge for raised neumorphic surfaces. Use on
  /// cards instead of a hard border — it reads as the lit rim of an
  /// extruded element, not a fence.
  edge: 'var(--color-edge)',

  // Text
  text: 'var(--color-text)',
  textMuted: 'var(--color-text-muted)',
  textDim: 'var(--color-text-dim)',
  textOnBrand: 'var(--color-text-on-brand)',
  textOnDanger: 'var(--color-text-on-danger)',

  // Brand
  brand: 'var(--color-brand)',
  brandHover: 'var(--color-brand-hover)',
  brandSoft: 'var(--color-brand-soft)',
  brandSoftText: 'var(--color-brand-soft-text)',

  // Status
  success: 'var(--color-success)',
  successSoft: 'var(--color-success-soft)',
  successText: 'var(--color-success-text)',
  danger: 'var(--color-danger)',
  dangerSoft: 'var(--color-danger-soft)',
  dangerText: 'var(--color-danger-text)',
  warn: 'var(--color-warn)',
  warnSoft: 'var(--color-warn-soft)',
  warnText: 'var(--color-warn-text)',
  info: 'var(--color-info)',
  infoSoft: 'var(--color-info-soft)',
  infoText: 'var(--color-info-text)',

  // Active nav highlight
  navActiveBg: 'var(--color-nav-active-bg)',
  navActiveText: 'var(--color-nav-active-text)',
} as const

type Palette = Record<string, string>

/// Default (light) palette. One warm stone surface — bg, cards, and sidebar
/// share #E7E5E4; depth comes from the shadow variables, never from
/// background steps. Status hues follow the neumorphism spec, with danger
/// darkened from the spec's #FF2157 so white button text clears WCAG AA.
const lightPalette: Palette = {
  '--color-bg': '#e7e5e4',
  '--color-surface': '#e7e5e4',
  '--color-sidebar': '#e7e5e4',
  '--color-hover': '#dedcda',
  '--color-divider': '#d6d3d1',
  '--color-border': '#cfccc9',
  '--color-border-soft': '#dbd8d6',
  '--color-edge': 'rgba(255, 255, 255, 0.55)',

  '--color-text': '#1e2938',
  '--color-text-muted': '#57606e',
  '--color-text-dim': '#767e8b',
  '--color-text-on-brand': '#ffffff',
  '--color-text-on-danger': '#ffffff',

  '--color-brand': teal[600],
  '--color-brand-hover': teal[700],
  '--color-brand-soft': 'rgba(0, 102, 102, 0.10)',
  '--color-brand-soft-text': '#00585a',

  '--color-success': '#00a63d',
  '--color-success-soft': 'rgba(0, 166, 61, 0.14)',
  '--color-success-text': '#0a6b31',
  '--color-danger': '#d81b47',
  '--color-danger-soft': 'rgba(255, 33, 87, 0.10)',
  '--color-danger-text': '#9f0f35',
  '--color-warn': '#fe9900',
  '--color-warn-soft': 'rgba(254, 153, 0, 0.16)',
  '--color-warn-text': '#8a5300',
  '--color-info': '#2563eb',
  '--color-info-soft': 'rgba(37, 99, 235, 0.12)',
  '--color-info-text': '#1d4ed8',

  '--color-nav-active-bg': 'rgba(0, 102, 102, 0.12)',
  '--color-nav-active-text': '#00585a',

  // Neumorphic depth. Dark shadow is the surface tone deepened; light
  // shadow is the same surface catching light. Raised pairs for cards and
  // controls, inset pairs for tracks and pressed states.
  '--shadow-card':
    '5px 5px 10px rgba(163, 158, 153, 0.42), -5px -5px 10px rgba(255, 255, 255, 0.85)',
  '--shadow-card-hover':
    '7px 7px 14px rgba(163, 158, 153, 0.50), -7px -7px 14px rgba(255, 255, 255, 0.90)',
  '--shadow-popover':
    '12px 12px 28px rgba(140, 135, 130, 0.45), -10px -10px 24px rgba(255, 255, 255, 0.80)',
  '--shadow-inset':
    'inset 2px 2px 5px rgba(163, 158, 153, 0.50), inset -2px -2px 5px rgba(255, 255, 255, 0.80)',
  '--shadow-pressed':
    'inset 3px 3px 7px rgba(163, 158, 153, 0.55), inset -3px -3px 7px rgba(255, 255, 255, 0.70)',
}

/// Dark palette. Same monochrome philosophy on a graphite surface: the
/// light shadow drops to a faint white sheen because dark-on-dark shadows
/// barely read — luminance does the lifting. Brand brightens to keep
/// contrast; status colors lift the same way.
const darkPalette: Palette = {
  '--color-bg': '#26282c',
  '--color-surface': '#26282c',
  '--color-sidebar': '#26282c',
  '--color-hover': '#2d3035',
  '--color-divider': '#1e2023',
  '--color-border': '#3a3d42',
  '--color-border-soft': '#313438',
  '--color-edge': 'rgba(255, 255, 255, 0.05)',

  '--color-text': '#e8eaed',
  '--color-text-muted': '#a6adb8',
  '--color-text-dim': '#7e848e',
  '--color-text-on-brand': '#06302c',
  '--color-text-on-danger': '#3b0716',

  '--color-brand': teal[400],
  '--color-brand-hover': '#2ad3c8',
  '--color-brand-soft': 'rgba(0, 191, 179, 0.13)',
  '--color-brand-soft-text': '#5eded4',

  '--color-success': '#34d399',
  '--color-success-soft': 'rgba(52, 211, 153, 0.14)',
  '--color-success-text': '#7ee8bc',
  '--color-danger': '#ff5c7e',
  '--color-danger-soft': 'rgba(255, 92, 126, 0.14)',
  '--color-danger-text': '#ff9fb3',
  '--color-warn': '#ffad33',
  '--color-warn-soft': 'rgba(255, 173, 51, 0.14)',
  '--color-warn-text': '#ffc97a',
  '--color-info': '#6ca1ff',
  '--color-info-soft': 'rgba(108, 161, 255, 0.14)',
  '--color-info-text': '#9fc0ff',

  '--color-nav-active-bg': 'rgba(0, 191, 179, 0.14)',
  '--color-nav-active-text': '#5eded4',

  '--shadow-card':
    '5px 5px 10px rgba(0, 0, 0, 0.45), -5px -5px 10px rgba(255, 255, 255, 0.04)',
  '--shadow-card-hover':
    '7px 7px 14px rgba(0, 0, 0, 0.55), -7px -7px 14px rgba(255, 255, 255, 0.05)',
  '--shadow-popover':
    '14px 14px 32px rgba(0, 0, 0, 0.60), -10px -10px 24px rgba(255, 255, 255, 0.04)',
  '--shadow-inset':
    'inset 2px 2px 5px rgba(0, 0, 0, 0.50), inset -2px -2px 5px rgba(255, 255, 255, 0.04)',
  '--shadow-pressed':
    'inset 3px 3px 7px rgba(0, 0, 0, 0.60), inset -3px -3px 7px rgba(255, 255, 255, 0.03)',
}

function paletteCSS(p: Palette): string {
  return Object.entries(p)
    .map(([k, v]) => `  ${k}: ${v};`)
    .join('\n')
}

/// CSS that defines the palette variables. Light is the default on `:root`.
/// In `system` mode the dark variant kicks in via `prefers-color-scheme`. An
/// explicit `data-theme` attribute on `<html>` always wins over the media
/// query.
export const THEME_CSS = `
:root {
${paletteCSS(lightPalette)}
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
${paletteCSS(darkPalette)}
  }
}
[data-theme="dark"] {
${paletteCSS(darkPalette)}
}
`

export const space = {
  0: '0',
  1: '4px',
  2: '8px',
  3: '12px',
  4: '16px',
  5: '20px',
  6: '24px',
  8: '32px',
  10: '40px',
  12: '48px',
  16: '64px',
} as const

/// Radius scale {4, 8, 12, 16, pill}. Soft extrusion wants generous
/// corners — cards sit at `lg`, controls at `md`.
export const radius = {
  sm: '4px',
  md: '8px',
  lg: '12px',
  xl: '16px',
  pill: '999px',
} as const

/// Depth lives in the theme palettes (shadow geometry is theme-dependent:
/// the light shadow that sculpts stone in light mode would glow like a
/// halo on graphite). `inset` carves tracks and wells; `pressed` is the
/// :active state of raised controls.
export const shadow = {
  card: 'var(--shadow-card)',
  cardHover: 'var(--shadow-card-hover)',
  popover: 'var(--shadow-popover)',
  inset: 'var(--shadow-inset)',
  pressed: 'var(--shadow-pressed)',
} as const

export const font = {
  /// Body copy. Deliberately NOT Space Mono: news summaries and briefs are
  /// long-form (often Chinese, which Space Mono can't shape anyway), and
  /// paragraph-length monospace is fatiguing. The mono personality lives in
  /// `display` and `mono` below.
  sans:
    'ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Hiragino Sans GB", "Noto Sans SC", "Microsoft YaHei", Roboto, "Helvetica Neue", Arial, sans-serif',
  /// Display voice — page titles, brand, stat values. Space Mono only
  /// ships 400/700; stick to those weights.
  display:
    '"Space Mono", "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace',
  /// Data voice — labels, badges, timestamps, numbers, tickers.
  mono: '"JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace',

  xs: '11px',
  sm: '12px',
  base: '14px',
  md: '15px',
  lg: '18px',
  xl: '22px',
  xxl: '28px',
} as const

/// Standard "tag-like" uppercase label seen in section headers and stat
/// labels. Bundled so we don't repeat the same five rules everywhere.
export const labelStyle = {
  fontFamily: font.mono,
  fontSize: font.xs,
  fontWeight: 600,
  color: color.textMuted,
  textTransform: 'uppercase' as const,
  letterSpacing: '0.08em',
}

/// Re-export ramps for cases that need the raw scale (rare).
export const palette = { slate, teal }
