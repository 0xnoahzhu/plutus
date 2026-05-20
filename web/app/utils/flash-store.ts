/// Server-side, in-memory flash store for values that need to survive
/// a POST → 303 → GET handoff WITHOUT going through the URL, a cookie,
/// or any other channel that ends up in logs / browser history.
///
/// Designed for the freshly-minted-API-key flow: the plain token is shown
/// to the user once after creation. Putting it in the redirect URL, a
/// `Set-Cookie`, or browser history leaks it into access logs and
/// browser sessions. This store holds it in process memory just long
/// enough for the next GET(s) to read it back, then drops it.
///
/// Single-process Node — no shared state across replicas. That's fine
/// here: plutus's web layer is one Node process backed by one Rust API
/// behind it; horizontal scale-out would need Redis or similar.
///
/// "Multiple reads" caveat: the global JS form interceptor in
/// `ui/document.tsx` does a fetch+follow that lands on the GET handler
/// (read 1), then `location.replace(resp.url)` triggers another GET
/// (read 2) so the user's URL bar reflects the destination. If the
/// flash were strict take-once, read 2 would find nothing and the
/// banner would disappear. We allow `MAX_READS` reads per entry; after
/// that, or after `TTL_MS`, the entry is dropped — F5 on the resulting
/// URL won't re-show the token.

import { randomUUID } from 'node:crypto'

interface Entry<T> {
  value: T
  expiresAt: number
  /// Remaining read budget. Decremented on each `takeFlash` call.
  /// At zero the entry is deleted on the way out.
  remainingReads: number
}

const store = new Map<string, Entry<unknown>>()

/// How long a flash value can sit before being garbage-collected.
/// 60s is plenty for the next request to come in, short enough that an
/// abandoned flow doesn't keep the value around indefinitely.
const TTL_MS = 60_000

/// How many reads an entry survives. 2 covers the fetch-follow +
/// location.replace double-GET that the JS form interceptor produces.
/// After 2 reads, the entry is dropped and refreshes return null.
const MAX_READS = 2

/// Store `value` under a fresh UUID and return the UUID. The caller should
/// pass this UUID to the next request (e.g. as a `?fresh=<uuid>` query
/// param on a redirect) so the next handler can call `takeFlash` to
/// retrieve it.
export function setFlash<T>(value: T): string {
  pruneExpired()
  let id = randomUUID()
  store.set(id, {
    value,
    expiresAt: Date.now() + TTL_MS,
    remainingReads: MAX_READS,
  })
  return id
}

/// Read the value, decrementing the remaining-read budget. Returns the
/// value if present + unexpired + budget > 0; returns `null` otherwise.
/// The entry is removed once the budget hits zero, so an F5 after the
/// allowed reads produces no banner.
export function takeFlash<T>(id: string | null | undefined): T | null {
  if (!id) return null
  let entry = store.get(id)
  if (!entry) return null
  if (entry.expiresAt < Date.now()) {
    store.delete(id)
    return null
  }
  entry.remainingReads -= 1
  if (entry.remainingReads <= 0) {
    store.delete(id)
  }
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
