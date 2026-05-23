import type { BuildAction } from 'remix/fetch-router'

import { api, type EntityKind } from '../api.ts'
import type { routes } from '../routes.ts'

const VALID: ReadonlySet<EntityKind> = new Set<EntityKind>([
  'news',
  'market_brief',
  'macro_event',
  'earnings_event',
  'catalyst',
  'screener_run',
  'recommendation',
  'portfolio_review',
  'correlation_run',
  'self_exam',
])

/// POST /reads/mark-all/:kind — server-side form action behind the
/// "标记全部已读" button. Calls the API, then redirects back to the
/// referring list page so the user lands where they were with refreshed
/// unread counts and dots.
export const markAllRead: BuildAction<'POST', typeof routes.markAllRead> = {
  async handler({ request, params }) {
    let kind = params.kind as EntityKind
    if (!VALID.has(kind)) {
      return new Response('Bad kind', { status: 400 })
    }
    let cookie = request.headers.get('cookie')
    await api.unreadMarkAll(kind, cookie).catch(() => 0)
    // Land back on the referring page so the user sees the refreshed
    // counts. Fall back to "/" if the referer is missing (rare — the
    // button is only ever submitted from inside an authenticated page).
    let referer = request.headers.get('referer') ?? '/'
    return Response.redirect(new URL(referer, request.url), 303)
  },
}
