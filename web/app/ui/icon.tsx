/// Wrapper around lucide-static SVG strings. lucide ships each icon as a
/// raw `<svg>...</svg>` string with `currentColor` strokes, so we just inject
/// it via `innerHTML` and style the host span.

import { css } from 'remix/ui'

export interface IconProps {
  /// SVG string imported from lucide-static, e.g.
  /// `import { Wallet } from 'lucide-static'`.
  svg: string
  /// Square pixel size. Defaults to 16 (the nav-row icon size).
  size?: number
  /// Stroke width override. lucide ships at `stroke-width="2"`; some uses
  /// look better at 1.5 or 1.8.
  strokeWidth?: number
  /// Optional override for the icon color. Falls back to `currentColor` so
  /// the icon inherits the parent's text color.
  color?: string
}

export function Icon() {
  return ({ svg, size = 16, strokeWidth, color }: IconProps) => (
    <span
      // `aria-hidden` on the host span — every consumer either has its own
      // text label adjacent (sidebar nav, chip) or sets a title elsewhere.
      aria-hidden="true"
      innerHTML={svg}
      mix={css({
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: `${size}px`,
        height: `${size}px`,
        flexShrink: 0,
        color: color ?? 'currentColor',
        '& svg': {
          width: '100%',
          height: '100%',
          // lucide's `class="lucide lucide-foo"` markers are unused — leaving
          // them in is fine.
        },
        ...(strokeWidth != null
          ? { '& svg path, & svg circle, & svg rect, & svg line, & svg polyline, & svg polygon': { strokeWidth: String(strokeWidth) } }
          : {}),
      })}
    />
  )
}
