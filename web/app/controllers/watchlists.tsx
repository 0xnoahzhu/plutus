import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Stock, type Watchlist, type WatchlistItem } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

interface LoadedGroup {
  watchlist: Watchlist
  items: WatchlistItem[]
}

export const watchlists: BuildAction<'GET', typeof routes.watchlists> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let [lists, allStocks] = await Promise.all([
      api.watchlists().catch(() => []),
      api.stocks().catch(() => []),
    ])
    let stockMap = new Map<number, Stock>(allStocks.map((s) => [s.id, s]))

    // Fetch items per group only to show counts and market badges on the list
    // view. The detail page hits the items endpoint again — fine at this scale.
    let groups: LoadedGroup[] = await Promise.all(
      lists.map(async (w) => ({
        watchlist: w,
        items: await api.watchlistItems(w.id).catch(() => []),
      })),
    )

    return render(
      <WatchlistsPage groups={groups} stocks={stockMap} locale={locale} />,
      request,
      { locale },
    )
  },
}

interface WatchlistsProps {
  groups: LoadedGroup[]
  stocks: Map<number, Stock>
  locale: string
}

// Watchlists are user-curated themed groups (e.g. "AI 芯片股" spanning
// US/HK/CN) so the country chip is intentionally absent here — filtering
// would defeat the cross-market grouping that's the whole point.
function WatchlistsPage() {
  return ({ groups, stocks, locale }: WatchlistsProps) => (
    <Layout title="Watchlists" locale={locale}>
      <p
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          marginBottom: '16px',
        })}
      >
        Click a group to see the stocks inside. Groups and items are created via
        the API:{' '}
        <code>POST /api/v1/watchlists</code>,{' '}
        <code>POST /api/v1/watchlists/:id/items</code>.
      </p>
      {groups.length === 0 ? (
        <p mix={css({ color: '#64748b' })}>No groups yet.</p>
      ) : (
        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))',
            gap: '12px',
          })}
        >
          {groups
            .slice()
            .sort((a, b) => a.watchlist.sort_order - b.watchlist.sort_order)
            .map(({ watchlist: w, items }) => {
              let marketsPresent = Array.from(
                new Set(
                  items
                    .map((it) => stocks.get(it.stock_id)?.market_code)
                    .filter((m): m is string => !!m),
                ),
              )
              return (
                <a
                  href={`/watchlists/${w.id}`}
                  mix={css({
                    display: 'block',
                    background: '#fff',
                    border: '1px solid #e2e8f0',
                    borderRadius: '8px',
                    padding: '16px 20px',
                    textDecoration: 'none',
                    color: 'inherit',
                    transition: 'border-color 120ms ease, transform 120ms ease',
                    '&:hover': {
                      borderColor: '#1d4ed8',
                      transform: 'translateY(-1px)',
                    },
                  })}
                >
                  <div
                    mix={css({
                      display: 'flex',
                      alignItems: 'baseline',
                      justifyContent: 'space-between',
                      gap: '12px',
                    })}
                  >
                    <div
                      mix={css({
                        fontWeight: 600,
                        fontSize: '15px',
                        color: '#0f172a',
                      })}
                    >
                      {w.name}
                    </div>
                    <div
                      mix={css({
                        fontSize: '12px',
                        fontWeight: 600,
                        color: '#1d4ed8',
                        whiteSpace: 'nowrap',
                      })}
                    >
                      {items.length}{' '}
                      {items.length === 1 ? 'symbol' : 'symbols'}
                    </div>
                  </div>
                  {w.description && (
                    <div mix={css({ fontSize: '13px', color: '#64748b', marginTop: '6px' })}>
                      {w.description}
                    </div>
                  )}
                  {marketsPresent.length > 0 && (
                    <div
                      mix={css({
                        display: 'flex',
                        gap: '6px',
                        marginTop: '10px',
                        flexWrap: 'wrap',
                      })}
                    >
                      {marketsPresent.map((m) => (
                        <span
                          mix={css({
                            fontSize: '10px',
                            fontWeight: 600,
                            padding: '1px 6px',
                            borderRadius: '999px',
                            background: '#e0e7ff',
                            color: '#3730a3',
                          })}
                        >
                          {m}
                        </span>
                      ))}
                    </div>
                  )}
                  <div
                    mix={css({
                      fontSize: '11px',
                      color: '#94a3b8',
                      marginTop: '10px',
                      textTransform: 'uppercase',
                      letterSpacing: '0.06em',
                    })}
                  >
                    created {w.created_at.slice(0, 10)}
                  </div>
                </a>
              )
            })}
        </div>
      )}
    </Layout>
  )
}
