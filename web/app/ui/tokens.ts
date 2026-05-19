/// Design tokens for the plutus UI. Single source of truth for colors,
/// spacing, typography, radii, and shadows. Components reach into these via
/// `import { color, space, ... } from './tokens.ts'`.

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

/// Semantic colors. Stick to these in components instead of raw hex so a
/// theme swap is a one-place change.
export const color = {
  // Surfaces
  bg: slate[50],            // page background
  surface: '#ffffff',       // cards
  sidebar: '#ffffff',       // sidebar (light theme)
  hover: slate[100],        // generic hover background
  divider: slate[200],
  border: slate[200],
  borderSoft: slate[100],

  // Text
  text: slate[900],
  textMuted: slate[500],
  textDim: slate[400],
  textOnBrand: '#ffffff',

  // Brand
  brand: teal[500],
  brandHover: teal[600],
  brandSoft: teal[50],
  brandSoftText: teal[700],

  // Status
  success: '#10b981',
  successSoft: '#dcfce7',
  successText: '#166534',
  danger: '#ef4444',
  dangerSoft: '#fee2e2',
  dangerText: '#991b1b',
  warn: '#f59e0b',
  warnSoft: '#fef3c7',
  warnText: '#92400e',
  info: '#3b82f6',
  infoSoft: '#dbeafe',
  infoText: '#1e40af',

  // Active nav highlight
  navActiveBg: teal[50],
  navActiveText: teal[700],
} as const

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
  // Tiny separator shadow; pairs well with a soft border.
  card: '0 1px 2px rgba(15, 23, 42, 0.04), 0 1px 1px rgba(15, 23, 42, 0.03)',
  /// Slightly lifted — for hovered or focused cards.
  cardHover: '0 4px 12px rgba(15, 23, 42, 0.06)',
  /// Stack/popover.
  popover: '0 8px 24px rgba(15, 23, 42, 0.08)',
} as const

export const font = {
  /// Stack tuned for cross-platform consistency. Inter when available,
  /// system-ui everywhere else.
  sans:
    'Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace',

  // Sizes
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
