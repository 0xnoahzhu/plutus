# Plutus Web Design System — Neumorphism

Tactile "carved stone" terminal: one monochromatic surface per theme where
controls extrude and content presses in. Derived from the
[typeui.sh neumorphism skill](https://github.com/bergside/awesome-design-skills/tree/main/skills/neumorphism),
adapted for accessibility and bilingual (en / zh-CN) content.

## Core idea

Depth replaces borders. Raised (dual outer shadows) means interactive or
new; inset (dual inner shadows) means tracks, wells, inputs, and pressed
states; flush (no shadow, hairline border) means read/secondary content.
The unread system rides this metaphor: **unread cards physically extrude
from the surface; read cards sit flush.**

## Tokens (`app/ui/tokens.ts`)

- **Surface (light)**: everything sits on `#E7E5E4`. Cards/sidebar share it —
  no background steps; the shadow pairs carry hierarchy.
- **Surface (dark)**: graphite `#26282C`; the light shadow drops to a faint
  white sheen (`rgba(255,255,255,0.04)`) because dark-on-dark shadows don't read.
- **Brand**: `#006666` light / `#00BFB3` dark. `textOnBrand` flips
  white → dark-teal accordingly.
- **Danger**: `#D81B47` light (darkened from the spec's `#FF2157` so white
  button text clears WCAG AA 4.5:1); the spec hue survives in `dangerSoft` tints.
- **Shadows**: theme-dependent CSS vars — `card`, `cardHover`, `popover`,
  `inset`, `pressed`. Never hand-roll a box-shadow; pick from `shadow.*`.
- **Edge**: `color.edge` is the translucent lit rim for raised surfaces.
  Use it instead of `color.border` on cards/buttons. `color.border`/`divider`
  remain for table rules and hairlines.
- **Radius scale**: `{4, 8, 12, 16, pill}` — cards `lg`, controls `md`.

## Typography

Three voices:

- `font.display` — Space Mono (400/700 only). Page titles, brand, stat values.
- `font.mono` — JetBrains Mono. Labels (`labelStyle`), badges, timestamps
  (`<time>` is globally mono + tabular-nums), tickers, numbers.
- `font.sans` — system sans + CJK fallbacks. Body copy, summaries, markdown.
  Deliberately NOT mono: long-form (often Chinese) text in monospace is
  fatiguing, and Space Mono has no CJK glyphs anyway.

## Component rules

- **Card**: `shadow.card` + `color.edge` rim. No hard borders.
- **Chips/tabs**: inset track (`color.hover` + `shadow.inset`), active option
  is a raised surface pill (`shadow.card`). Never a solid brand fill.
- **Buttons**: raised; `:active` flips to `shadow.pressed` + `scale(0.98)` —
  the signature press-in interaction. Primary buttons: `color.brand` bg +
  `color.textOnBrand` text (never literal `#fff`).
- **Inputs**: inset wells (`color.hover` + `shadow.inset`), focus ring via
  `borderColor: color.brand`.
- **Unread**: `unreadCardStyle` — brandSoft tint + brand border + raised;
  read = flush + `borderSoft` hairline.
- **Sidebar**: monochrome column; current section gets `navActiveBg/Text`
  (set via `ambientPath()`); collapses to a horizontal scroll strip ≤900px.
- **Inner wells** (table header strips, code blocks): `color.hover`, never
  `color.bg` (bg == surface, it would vanish).

## Don'ts

- No gradients, no glassmorphism, no `#fff`/raw hex in components — tokens only.
- No new shadow geometries; the five `shadow.*` vars are the system.
- Don't put `color.text` fills behind active chips; soft-active only.
- Don't use Space Mono for paragraphs or anything that can contain Chinese.
- Keep `prefers-reduced-motion` and `:focus-visible` rules intact (document.tsx).
