import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import type { routes } from '../routes.ts'
import {
  Card,
  color,
  EmptyState,
  font,
  Layout,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  Stat,
  type Theme,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const home: BuildAction<'GET', typeof routes.home> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let [markets, brokers, accounts, stocks, watchlistItems, transactions, holdings] =
      await Promise.all([
        api.markets().catch(() => []),
        api.brokers().catch(() => []),
        api.accounts().catch(() => []),
        api.stocks().catch(() => []),
        api.watchlistItems().catch(() => []),
        api.transactions().catch(() => []),
        api.holdings().catch(() => []),
      ])
    let healthy = markets.length > 0
    return render(
      <DashboardPage
        healthy={healthy}
        locale={locale}
        theme={theme}
        counts={{
          markets: markets.length,
          brokers: brokers.length,
          accounts: accounts.length,
          stocks: stocks.length,
          watchlist: watchlistItems.length,
          transactions: transactions.length,
          holdings: holdings.length,
        }}
      />,
      request,
      { locale, theme },
    )
  },
}

interface DashboardProps {
  healthy: boolean
  locale: string
  theme: Theme
  counts: Record<string, number>
}

function DashboardPage() {
  return ({ healthy, locale, theme, counts }: DashboardProps) => (
    <Layout title="Dashboard" subtitle="Today's snapshot" locale={locale} theme={theme}>
      <SectionTitle hint="real-time">Quick Stats</SectionTitle>
      <div
        mix={css({
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
          gap: space[3],
          marginBottom: space[8],
        })}
      >
        <Stat
          label="API Status"
          value={healthy ? 'Connected' : 'Down'}
          caption={healthy ? 'all systems go' : 'check the server'}
          trend={healthy ? 'up' : 'down'}
        />
        <Stat label="Markets" value={String(counts.markets)} caption="open" />
        <Stat label="Brokers" value={String(counts.brokers)} caption="active" />
        <Stat label="Accounts" value={String(counts.accounts)} caption="total" />
        <Stat label="Stocks" value={String(counts.stocks)} caption="tracked" />
        <Stat label="Watchlist" value={String(counts.watchlist)} caption="stocks" />
        <Stat label="Transactions" value={String(counts.transactions)} caption="recorded" />
        <Stat label="Open Positions" value={String(counts.holdings)} caption="current" />
      </div>

      <div
        mix={css({
          display: 'grid',
          gridTemplateColumns: '2fr 1fr',
          gap: space[5],
          '@media (max-width: 1000px)': { gridTemplateColumns: '1fr' },
        })}
      >
        <Card>
          <SectionTitle hint="hooked up next phase">Portfolio Performance</SectionTitle>
          <EmptyState
            title="Chart not wired yet"
            hint={
              <>
                Time-series of <code>{`/api/v1/holdings`}</code> + intraday quotes
                lands once the OHLCV ingestion is online.
              </>
            }
          />
        </Card>

        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[4] })}>
          <Card>
            <SectionTitle>Recent Activity</SectionTitle>
            <EmptyState title="No activity yet" hint="agent writes will surface here" />
          </Card>
          <Card>
            <SectionTitle>Top Movers</SectionTitle>
            <EmptyState
              title="No quotes loaded"
              hint={
                <>
                  Push intraday prices via{' '}
                  <code>POST /api/v1/ohlcv</code>.
                </>
              }
            />
          </Card>
        </div>
      </div>

      <p
        mix={css({
          marginTop: space[8],
          fontSize: font.sm,
          color: color.textMuted,
        })}
      >
        API base: <code>{api.base}</code>
      </p>
    </Layout>
  )
}
