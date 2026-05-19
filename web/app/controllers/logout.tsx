import type { BuildAction } from 'remix/fetch-router'

import { api } from '../api.ts'
import type { routes } from '../routes.ts'

/// POST /logout — hand the existing session cookie up to the API so it can
/// invalidate, forward the `Set-Cookie` (which clears the cookie) back to
/// the browser, then redirect to /login.
export const logout: BuildAction<'POST', typeof routes.logout> = {
  async handler({ request }) {
    let cookie = request.headers.get('cookie')
    let upstream = await api.logoutRaw(cookie).catch(() => null)

    let headers = new Headers({ Location: '/login' })
    let setCookie = upstream?.headers.get('set-cookie')
    if (setCookie) headers.set('Set-Cookie', setCookie)
    return new Response(null, { status: 303, headers })
  },
}
