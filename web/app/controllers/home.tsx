import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import type { routes } from '../routes.ts'
import { Card, Layout, resolveLocale, Stat } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const home: BuildAction<'GET', typeof routes.home> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let [markets, brokers, accounts, stocks, watchlists, transactions, holdings] =
      await Promise.all([
        api.markets().catch(() => []),
        api.brokers().catch(() => []),
        api.accounts().catch(() => []),
        api.stocks().catch(() => []),
        api.watchlists().catch(() => []),
        api.transactions().catch(() => []),
        api.holdings().catch(() => []),
      ])
    let healthy = markets.length > 0
    return render(
      <DashboardPage
        healthy={healthy}
        locale={locale}
        counts={{
          markets: markets.length,
          brokers: brokers.length,
          accounts: accounts.length,
          stocks: stocks.length,
          watchlists: watchlists.length,
          transactions: transactions.length,
          holdings: holdings.length,
        }}
      />,
      request,
      { locale },
    )
  },
}

interface DashboardProps {
  healthy: boolean
  locale: string
  counts: Record<string, number>
}

function DashboardPage() {
  return ({ healthy, locale, counts }: DashboardProps) => (
    <Layout title="Dashboard" locale={locale}>
      <div
        mix={css({
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
          gap: '16px',
        })}
      >
        <Card>
          <Stat label="API status" value={healthy ? '✓ connected' : '✗ unreachable'} />
        </Card>
        <Card>
          <Stat label="Markets" value={String(counts.markets)} />
        </Card>
        <Card>
          <Stat label="Brokers" value={String(counts.brokers)} />
        </Card>
        <Card>
          <Stat label="Accounts" value={String(counts.accounts)} />
        </Card>
        <Card>
          <Stat label="Stocks" value={String(counts.stocks)} />
        </Card>
        <Card>
          <Stat label="Watchlists" value={String(counts.watchlists)} />
        </Card>
        <Card>
          <Stat label="Transactions" value={String(counts.transactions)} />
        </Card>
        <Card>
          <Stat label="Open positions" value={String(counts.holdings)} />
        </Card>
      </div>
      <p
        mix={css({
          marginTop: '24px',
          fontSize: '13px',
          color: '#64748b',
        })}
      >
        API base: <code>{api.base}</code>
      </p>
    </Layout>
  )
}
