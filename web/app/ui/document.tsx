import type { RemixNode } from 'remix/ui'

import { routes } from '../routes.ts'

export interface DocumentProps {
  children?: RemixNode
  title?: string
  /// BCP-47 language tag for the document. Defaults to "en". Layout passes
  /// the resolved request locale so screen readers and CSS :lang() see the
  /// right value.
  lang?: string
}

const DEFAULT_TITLE = decodeURIComponent('plutus')

export function Document() {
  return ({ title = DEFAULT_TITLE, lang = 'en', children }: DocumentProps) => (
    <html lang={lang}>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>{title}</title>
      </head>
      <body>
        {children}
        <script type="module" src={routes.assets.href({ path: 'app/assets/entry.ts' })}></script>
      </body>
    </html>
  )
}
