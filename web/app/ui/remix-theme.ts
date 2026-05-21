/// `@remix-run/ui` theme contract bound to plutus's design tokens.
///
/// Headless primitives like `Select`, `Combobox`, and `Popover` read
/// their colors / spacing / type / shadows from a CSS-variable contract
/// (`--rmx-*`). `createTheme` emits the rules; mount the returned
/// component once in the document head and the primitives style
/// themselves automatically.
///
/// We do not adopt the remix tokens as canonical — the rest of the app
/// keeps using `tokens.ts`. This file is a bridge that maps OUR tokens
/// into the contract the primitives expect.
///
/// Two themes are emitted: a light one on `:root` and a dark one on
/// `[data-theme="dark"]`. The dark variant intentionally passes
/// `reset: false` so its `<style>` only outputs CSS-variable overrides
/// — the base reset (margin / box-sizing) is owned by the light theme
/// and inherited.
import { createTheme } from 'remix/ui/theme'

import { palette, radius, shadow, space } from './tokens.ts'

const { slate, teal } = palette

const sharedScale = {
  space: {
    none: space[0],
    px: '1px',
    xs: space[1],
    sm: space[2],
    md: space[3],
    lg: space[4],
    xl: space[5],
    xxl: space[6],
  },
  radius: {
    none: '0',
    sm: radius.sm,
    md: radius.md,
    lg: radius.lg,
    xl: radius.xl,
    full: radius.pill,
  },
  fontFamily: {
    sans:
      'Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace',
  },
  fontSize: {
    xxxs: '10px',
    xxs: '11px',
    xs: '12px',
    sm: '13px',
    md: '14px',
    lg: '16px',
    xl: '20px',
    xxl: '28px',
  },
  lineHeight: {
    tight: '1.25',
    normal: '1.5',
    relaxed: '1.65',
  },
  letterSpacing: {
    tight: '-0.02em',
    normal: '0',
    meta: '0.04em',
    wide: '0.08em',
  },
  fontWeight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
  },
  control: {
    height: {
      sm: '28px',
      md: '34px',
      lg: '40px',
    },
  },
}

export const RemixThemeLight = createTheme(
  {
    ...sharedScale,
    surface: {
      lvl0: slate[50],
      lvl1: '#ffffff',
      lvl2: slate[100],
      lvl3: '#ffffff',
      lvl4: slate[100],
    },
    shadow: {
      xs: '0 1px 1px rgba(15, 23, 42, 0.04)',
      sm: shadow.card,
      md: shadow.cardHover,
      lg: shadow.popover,
      xl: '0 24px 48px rgba(15, 23, 42, 0.12)',
    },
    colors: {
      text: {
        primary: slate[900],
        secondary: slate[600],
        muted: slate[500],
        link: teal[600],
      },
      border: {
        subtle: slate[100],
        default: slate[200],
        strong: slate[300],
      },
      focus: { ring: teal[500] },
      overlay: { scrim: 'rgba(15, 23, 42, 0.45)' },
      action: {
        primary: {
          background: teal[500],
          backgroundHover: teal[600],
          backgroundActive: teal[700],
          foreground: '#ffffff',
          border: teal[500],
        },
        secondary: {
          background: '#ffffff',
          backgroundHover: slate[100],
          backgroundActive: slate[200],
          foreground: slate[900],
          border: slate[200],
        },
        danger: {
          background: '#ef4444',
          backgroundHover: '#dc2626',
          backgroundActive: '#b91c1c',
          foreground: '#ffffff',
          border: '#ef4444',
        },
      },
    },
  },
  { selector: ':root', reset: false },
)

export const RemixThemeDark = createTheme(
  {
    ...sharedScale,
    surface: {
      lvl0: slate[950],
      lvl1: slate[900],
      lvl2: slate[800],
      lvl3: slate[900],
      lvl4: slate[800],
    },
    shadow: {
      xs: '0 1px 1px rgba(0, 0, 0, 0.3)',
      sm: '0 1px 2px rgba(0, 0, 0, 0.4)',
      md: '0 6px 18px rgba(0, 0, 0, 0.45)',
      lg: '0 16px 34px rgba(0, 0, 0, 0.5)',
      xl: '0 24px 52px rgba(0, 0, 0, 0.55)',
    },
    colors: {
      text: {
        primary: slate[100],
        secondary: slate[300],
        muted: slate[400],
        link: teal[300],
      },
      border: {
        subtle: slate[800],
        default: slate[700],
        strong: slate[600],
      },
      focus: { ring: teal[400] },
      overlay: { scrim: 'rgba(0, 0, 0, 0.55)' },
      action: {
        primary: {
          background: teal[400],
          backgroundHover: teal[300],
          backgroundActive: teal[500],
          foreground: slate[950],
          border: teal[400],
        },
        secondary: {
          background: slate[800],
          backgroundHover: slate[700],
          backgroundActive: slate[600],
          foreground: slate[100],
          border: slate[700],
        },
        danger: {
          background: '#f87171',
          backgroundHover: '#ef4444',
          backgroundActive: '#dc2626',
          foreground: slate[950],
          border: '#f87171',
        },
      },
    },
  },
  { selector: '[data-theme="dark"]', reset: false },
)
