import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api, type Stock, type Watchlist, type WatchlistItem } from '../api.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  Card,
  color,
  EmptyState,
  font,
  Layout,
  radius,
  resolveLocale,
  space,
} from '../ui/layout.tsx'
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
// US/HK/CN) so the country chip is intentionally absent here.
function WatchlistsPage() {
  return ({ groups, stocks, locale }: WatchlistsProps) => (
    <Layout
      title="Watchlists"
      subtitle="User-curated themed groups across markets"
      locale={locale}
    >
      {groups.length === 0 ? (
        <Card>
          <EmptyState
            title="No watchlists yet"
            hint={
              <>
                Create one with <code>POST /api/v1/watchlists</code>.
              </>
            }
          />
        </Card>
      ) : (
        <div
          mix={css({
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
            gap: space[3],
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
                    background: color.surface,
                    border: `1px solid ${color.border}`,
                    borderRadius: radius.lg,
                    padding: `${space[4]} ${space[5]}`,
                    textDecoration: 'none',
                    color: 'inherit',
                    transition: 'border-color 120ms ease, transform 120ms ease',
                    '&:hover': {
                      borderColor: color.brand,
                      transform: 'translateY(-1px)',
                    },
                  })}
                >
                  <div
                    mix={css({
                      display: 'flex',
                      alignItems: 'baseline',
                      justifyContent: 'space-between',
                      gap: space[3],
                    })}
                  >
                    <div
                      mix={css({
                        fontWeight: 600,
                        fontSize: font.md,
                        color: color.text,
                      })}
                    >
                      {w.name}
                    </div>
                    <div
                      mix={css({
                        fontSize: font.sm,
                        fontWeight: 600,
                        color: color.brandHover,
                        whiteSpace: 'nowrap',
                      })}
                    >
                      {items.length} {items.length === 1 ? 'symbol' : 'symbols'}
                    </div>
                  </div>
                  {w.description && (
                    <div
                      mix={css({
                        fontSize: font.sm,
                        color: color.textMuted,
                        marginTop: space[2],
                        lineHeight: 1.5,
                      })}
                    >
                      {w.description}
                    </div>
                  )}
                  {marketsPresent.length > 0 && (
                    <div
                      mix={css({
                        display: 'flex',
                        gap: space[1],
                        marginTop: space[3],
                        flexWrap: 'wrap',
                      })}
                    >
                      {marketsPresent.map((m) => (
                        <Badge tone="brand">{m}</Badge>
                      ))}
                    </div>
                  )}
                  <div
                    mix={css({
                      fontSize: font.xs,
                      color: color.textDim,
                      marginTop: space[3],
                      textTransform: 'uppercase',
                      letterSpacing: '0.08em',
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
