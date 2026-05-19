import type { RemixNode } from 'remix/ui'

import { routes } from '../routes.ts'

import { color, font } from './tokens.ts'

export interface DocumentProps {
  children?: RemixNode
  title?: string
  /// BCP-47 language tag for the document. Defaults to "en". Layout passes
  /// the resolved request locale so screen readers and CSS :lang() see the
  /// right value.
  lang?: string
}

const DEFAULT_TITLE = 'plutus'

/// Tiny global stylesheet. Lives inline so we don't have to wire up a static
/// CSS file or PostCSS pipeline yet — fine for the single-user app size.
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
  return ({ title = DEFAULT_TITLE, lang = 'en', children }: DocumentProps) => (
    <html lang={lang}>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>{title}</title>
        {/* Inter as a progressive enhancement. system-ui fallback below. */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          rel="stylesheet"
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
        />
        <style innerHTML={GLOBAL_CSS} />
      </head>
      <body>
        {children}
        <script type="module" src={routes.assets.href({ path: 'app/assets/entry.ts' })}></script>
      </body>
    </html>
  )
}
