/// Server-side, in-memory one-shot store for values that need to survive
/// a POST → 303 → GET handoff WITHOUT going through the URL, a cookie,
/// or any other channel that ends up in logs / browser history.
///
/// Designed for the freshly-minted-API-key flow: the plain token is shown
/// to the user exactly once after creation. Putting it in the redirect
/// URL, a `Set-Cookie`, or browser history leaks it into access logs and
/// browser sessions. This store holds it in process memory just long
/// enough for the next GET to read it back.
///
/// Single-process Node — no shared state across replicas. That's fine
/// here: plutus's web layer is one Node process backed by one Rust API
/// behind it; horizontal scale-out would need Redis or similar.

import { randomUUID } from 'node:crypto'

interface Entry<T> {
  value: T
  expiresAt: number
}

const store = new Map<string, Entry<unknown>>()

/// How long a flash value can sit unread before being garbage-collected.
/// 60s is plenty for the next request to come in, short enough that an
/// abandoned flow doesn't keep the value around indefinitely.
const TTL_MS = 60_000

/// Store `value` under a fresh UUID and return the UUID. The caller should
/// pass this UUID to the next request (e.g. as a `?fresh=<uuid>` query
/// param on a redirect) so the next handler can call `takeFlash` to
/// retrieve and remove it.
export function setFlash<T>(value: T): string {
  pruneExpired()
  let id = randomUUID()
  store.set(id, { value, expiresAt: Date.now() + TTL_MS })
  return id
}

/// Atomic take-and-delete. Returns the value if present and unexpired,
/// `null` otherwise. The entry is removed before returning so a refresh
/// of the same URL produces no banner.
export function takeFlash<T>(id: string | null | undefined): T | null {
  if (!id) return null
  let entry = store.get(id)
  if (!entry) return null
  store.delete(id)
  if (entry.expiresAt < Date.now()) return null
  return entry.value as T
}

/// Walk the map and drop expired entries. Called on every write — cheap
/// because the map should never carry more than a handful of entries
/// (each one is the trailing tail of a recent POST flow).
function pruneExpired(): void {
  let now = Date.now()
  for (let [id, entry] of store.entries()) {
    if (entry.expiresAt < now) store.delete(id)
  }
}
