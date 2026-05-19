import type { RemixNode } from 'remix/ui'

import { routes } from '../routes.ts'

import { color, font, THEME_CSS } from './tokens.ts'

export interface DocumentProps {
  children?: RemixNode
  title?: string
  /// BCP-47 language tag for the document. Defaults to "en".
  lang?: string
  /// Resolved color-scheme choice: `system` lets CSS `prefers-color-scheme`
  /// decide; `dark` / `light` force the palette via a `data-theme` attribute.
  /// Defaults to `system`.
  theme?: 'system' | 'dark' | 'light'
}

const DEFAULT_TITLE = 'Plutus'

/// Tiny global stylesheet. Lives inline so we don't have to wire up a static
/// CSS file or PostCSS pipeline yet — fine for the single-user app size.
/// `body` colors are token-driven so the theme switch swaps them in CSS.
const GLOBAL_CSS = `
  *, *::before, *::after { box-sizing: border-box; }
  html, body {
    margin: 0;
    padding: 0;
    background: ${color.bg};
    color: ${color.text};
    font-family: ${font.sans};
    font-size: ${font.base};
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-rendering: optimizeLegibility;
  }
  a { color: inherit; }
  button { font-family: inherit; }
  table { border-collapse: collapse; }
`

export function Document() {
  return ({ title = DEFAULT_TITLE, lang = 'en', theme = 'system', children }: DocumentProps) => {
    // `data-theme="dark"|"light"` pins the palette; `system` omits the attr
    // and lets the `prefers-color-scheme` rule in THEME_CSS decide.
    let themeAttr: Record<string, string> =
      theme === 'system' ? {} : { 'data-theme': theme }
    return (
      <html lang={lang} {...themeAttr}>
        <head>
          <meta charSet="utf-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1" />
          <meta name="color-scheme" content="light dark" />
          <title>{title}</title>
          {/* Inter as a progressive enhancement. system-ui fallback below. */}
          <link rel="preconnect" href="https://fonts.googleapis.com" />
          <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
          <link
            rel="stylesheet"
            href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
          />
          <style innerHTML={THEME_CSS} />
          <style innerHTML={GLOBAL_CSS} />
        </head>
        <body>
          {children}
          <script type="module" src={routes.assets.href({ path: 'app/assets/entry.ts' })}></script>
        </body>
      </html>
    )
  }
}
