/// Web-side auth guard. The API enforces tenancy on its own (per-user
/// isolation, password-reset 403 gate), but the web has to decide what
/// page to *render* when a request lands. This module resolves the
/// caller's identity by calling `/auth/me` and provides a `withAuth`
/// wrapper that turns it into a redirect contract:
///
/// - no session  → 303 `/login?next=<original>`
/// - admin       → 303 `/admin` (admin has no per-user data; data pages
///                  would 403 the request anyway)
/// - regular user → handler runs normally

import type { RequestHandler } from 'remix/fetch-router'

import { api, runWithCookie } from '../api.ts'

export interface SignedInUser {
  kind: 'web' | 'api_token' | 'admin'
  username: string
  user_id: number | null
  is_admin: boolean
}

/// Returns the caller's identity, or `null` when no valid session is
/// present. `/auth/me` is reachable to everyone (it returns
/// `kind: "anonymous"` for unauthenticated callers), so we never need to
/// 401-handle here — we just translate "anonymous" to `null`.
export async function resolveMe(request: Request): Promise<SignedInUser | null> {
  let cookie = request.headers.get('cookie')
  try {
    let me = await api.me(cookie)
    if (me.kind === 'anonymous' || me.kind === 'system') return null
    return {
      kind: me.kind as SignedInUser['kind'],
      username: me.label,
      user_id: me.user_id,
      is_admin: me.is_admin,
    }
  } catch {
    return null
  }
}

/// Wrap an `Action`'s handler with an auth guard. The input is the
/// per-route `Action` union (function-form or `{ handler }` object-form)
/// which TS infers with route-specific params. We accept it opaquely
/// (the union resists narrowing across the params type parameter) and
/// always return the object-form, which is a valid member of any Action
/// type so `router.map` continues to type-check.
export function withAuth<A>(action: A): A {
  let inner: RequestHandler<any, any> =
    typeof action === 'function'
      ? (action as unknown as RequestHandler<any, any>)
      : (action as { handler: RequestHandler<any, any> }).handler

  let wrapped: RequestHandler<any, any> = async (ctx) => {
    let req = (ctx as { request: Request }).request
    let cookie = req.headers.get('cookie')
    let me = await resolveMe(req)
    if (!me) {
      let url = new URL(req.url)
      // remix beta sometimes serializes an empty querystring as the literal
      // "?undefined" — strip that artifact so the round-tripped `next`
      // doesn't leak it back into the address bar after login.
      let search = url.search === '?undefined' ? '' : url.search
      let next = url.pathname + search
      return Response.redirect(
        new URL(`/login?next=${encodeURIComponent(next)}`, req.url),
        303,
      )
    }
    // Admin acting on data routes would just 403 anyway — route them
    // to their own management surface instead.
    if (me.is_admin) {
      return Response.redirect(new URL('/admin', req.url), 303)
    }
    // Bind the cookie to an async-local context so every `api.*()` call
    // the handler makes inherits the session automatically — no need to
    // thread the cookie through every controller's call sites.
    return runWithCookie(cookie, () => inner(ctx))
  }
  return { handler: wrapped } as unknown as A
}
