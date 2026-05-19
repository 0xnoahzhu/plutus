import type { RemixNode } from 'remix/ui'
import { renderToStream } from 'remix/ui/server'

import { router } from '../router.ts'
import { localeCookie, themeCookie, type Locale, type Theme } from '../ui/layout.tsx'

export interface RenderOptions extends ResponseInit {
  /// Resolved request locale to persist as a cookie. So the next request
  /// without `?locale=` lands on the same language.
  locale?: Locale
  /// Resolved color-scheme to persist. Mirrors `locale` — without this the
  /// next page request would forget the user's pick.
  theme?: Theme
}

export function render(node: RemixNode, request: Request, opts?: RenderOptions) {
  let stream = renderToStream(node, {
    frameSrc: request.url,
    async resolveFrame(src, target) {
      let headers = new Headers({ accept: 'text/html' })
      let cookie = request.headers.get('cookie')
      if (cookie) headers.set('cookie', cookie)
      if (target) headers.set('x-remix-target', target)

      let response = await router.fetch(new Request(new URL(src, request.url), { headers }))
      return response.body ?? response.text()
    },
  })

  let headers = new Headers(opts?.headers)
  if (!headers.has('Content-Type')) {
    headers.set('Content-Type', 'text/html; charset=utf-8')
  }
  if (opts?.locale) {
    // append so we don't clobber other Set-Cookie callers
    headers.append('Set-Cookie', localeCookie(opts.locale))
  }
  if (opts?.theme) {
    headers.append('Set-Cookie', themeCookie(opts.theme))
  }

  return new Response(stream, { ...opts, headers })
}
