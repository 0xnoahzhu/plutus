/// Design tokens for the plutus UI. Single source of truth for colors,
/// spacing, typography, radii, and shadows. Components reach into these via
/// `import { color, space, ... } from './tokens.ts'`.
///
/// Colors are exposed as CSS-variable strings (`var(--color-bg)`) so a
/// single `<style>` block in [[Document]] can swap the underlying palette
/// for dark mode without each component knowing anything about themes.

/// Neutrals — slate ramp. 50 (lightest) → 950 (darkest). Mirrors Tailwind's
/// slate scale so values feel familiar.
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

/// Brand — cyan/teal ramp keyed to mockup logo. 500/600 are the workhorses.
const teal = {
  50: '#ecfeff',
  100: '#cffafe',
  200: '#a5f3fc',
  300: '#67e8f9',
  400: '#22d3ee',
  500: '#06b6d4',
  600: '#0891b2',
  700: '#0e7490',
  800: '#155e75',
  900: '#164e63',
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

  // Text
  text: 'var(--color-text)',
  textMuted: 'var(--color-text-muted)',
  textDim: 'var(--color-text-dim)',
  textOnBrand: 'var(--color-text-on-brand)',

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

/// Default (light) palette values.
const lightPalette: Palette = {
  '--color-bg': slate[50],
  '--color-surface': '#ffffff',
  '--color-sidebar': '#ffffff',
  '--color-hover': slate[100],
  '--color-divider': slate[200],
  '--color-border': slate[200],
  '--color-border-soft': slate[100],

  '--color-text': slate[900],
  '--color-text-muted': slate[500],
  '--color-text-dim': slate[400],
  '--color-text-on-brand': '#ffffff',

  '--color-brand': teal[500],
  '--color-brand-hover': teal[600],
  '--color-brand-soft': teal[50],
  '--color-brand-soft-text': teal[700],

  '--color-success': '#10b981',
  '--color-success-soft': '#dcfce7',
  '--color-success-text': '#166534',
  '--color-danger': '#ef4444',
  '--color-danger-soft': '#fee2e2',
  '--color-danger-text': '#991b1b',
  '--color-warn': '#f59e0b',
  '--color-warn-soft': '#fef3c7',
  '--color-warn-text': '#92400e',
  '--color-info': '#3b82f6',
  '--color-info-soft': '#dbeafe',
  '--color-info-text': '#1e40af',

  '--color-nav-active-bg': teal[50],
  '--color-nav-active-text': teal[700],
}

/// Dark palette. Surfaces flip to slate-900/800 and text inverts; brand stays
/// teal (cyan reads well on dark). Status colors get softer backgrounds and
/// brighter foregrounds so badges still pop without burning eyes.
const darkPalette: Palette = {
  '--color-bg': slate[950],
  '--color-surface': slate[900],
  '--color-sidebar': slate[900],
  '--color-hover': slate[800],
  '--color-divider': slate[800],
  '--color-border': slate[800],
  '--color-border-soft': slate[800],

  '--color-text': slate[100],
  '--color-text-muted': slate[400],
  '--color-text-dim': slate[500],
  '--color-text-on-brand': slate[950],

  '--color-brand': teal[400],
  '--color-brand-hover': teal[300],
  '--color-brand-soft': 'rgba(34, 211, 238, 0.12)',
  '--color-brand-soft-text': teal[300],

  '--color-success': '#34d399',
  '--color-success-soft': 'rgba(52, 211, 153, 0.12)',
  '--color-success-text': '#6ee7b7',
  '--color-danger': '#f87171',
  '--color-danger-soft': 'rgba(248, 113, 113, 0.12)',
  '--color-danger-text': '#fca5a5',
  '--color-warn': '#fbbf24',
  '--color-warn-soft': 'rgba(251, 191, 36, 0.12)',
  '--color-warn-text': '#fcd34d',
  '--color-info': '#60a5fa',
  '--color-info-soft': 'rgba(96, 165, 250, 0.12)',
  '--color-info-text': '#93c5fd',

  '--color-nav-active-bg': 'rgba(34, 211, 238, 0.12)',
  '--color-nav-active-text': teal[300],
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

export const radius = {
  sm: '4px',
  md: '6px',
  lg: '8px',
  xl: '12px',
  pill: '999px',
} as const

export const shadow = {
  card: '0 1px 2px rgba(15, 23, 42, 0.04), 0 1px 1px rgba(15, 23, 42, 0.03)',
  cardHover: '0 4px 12px rgba(15, 23, 42, 0.06)',
  popover: '0 8px 24px rgba(15, 23, 42, 0.08)',
} as const

export const font = {
  sans:
    'Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace',

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
  fontSize: font.xs,
  fontWeight: 600,
  color: color.textMuted,
  textTransform: 'uppercase' as const,
  letterSpacing: '0.08em',
}

/// Re-export ramps for cases that need the raw scale (rare).
export const palette = { slate, teal }
