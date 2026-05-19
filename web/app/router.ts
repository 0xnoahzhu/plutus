import { createRouter } from 'remix/fetch-router'

import { assets } from './assets.ts'
import { accountCreate, accountDelete, accounts } from './controllers/accounts.tsx'
import { admin, adminUserCreate, adminUserDelete, adminUserReset } from './controllers/admin.tsx'
import {
  adminBrokerCreate,
  adminBrokerDelete,
  adminBrokerRename,
  adminBrokers,
} from './controllers/admin-brokers.tsx'
import {
  adminTokenCreate,
  adminTokenDelete,
  adminTokens,
} from './controllers/admin-tokens.tsx'
import { apiKeyCreate, apiKeyDelete, apiKeys } from './controllers/api-keys.tsx'
import { audit } from './controllers/audit.tsx'
import { briefs } from './controllers/briefs.tsx'
import { catalysts } from './controllers/catalysts.tsx'
import { changePassword } from './controllers/change-password.tsx'
import { correlations } from './controllers/correlations.tsx'
import { earnings } from './controllers/earnings.tsx'
import { holdings } from './controllers/holdings.tsx'
import { home } from './controllers/home.tsx'
import { login } from './controllers/login.tsx'
import { logout } from './controllers/logout.tsx'
import { macroEvents } from './controllers/macro-events.tsx'
import { news } from './controllers/news.tsx'
import { newsDetail } from './controllers/news-detail.tsx'
import { portfolioReviews } from './controllers/portfolio-reviews.tsx'
import { recommendations } from './controllers/recommendations.tsx'
import { screeners } from './controllers/screeners.tsx'
import { selfExams } from './controllers/self-exams.tsx'
import { settings } from './controllers/settings.tsx'
import { stockDetail } from './controllers/stock-detail.tsx'
import { stocks } from './controllers/stocks.tsx'
import { transactions } from './controllers/transactions.tsx'
import { watchlists } from './controllers/watchlists.tsx'
import { routes } from './routes.ts'
import { withAuth } from './utils/auth.ts'

export const router = createRouter()

router.get(routes.assets, async ({ request }) => {
  let response = await assets.fetch(request)
  return response ?? new Response('Not Found', { status: 404 })
})

// Public routes — no auth guard. /login + /logout obviously, and the
// /admin/* surface enforces admin via the API instead (admin.tsx
// redirects non-admins to /login on its own). /change-password also
// stays open: the API's password-reset gate (Step 3) lets a logged-in
// user who must change their password reach this exact endpoint.
router.map(routes.login.index, login.index)
router.map(routes.login.action, login.action)
router.map(routes.logout, logout)
router.map(routes.changePassword.index, changePassword.index)
router.map(routes.changePassword.action, changePassword.action)
router.map(routes.admin, admin)
router.map(routes.adminBrokers, adminBrokers)
router.map(routes.adminUserCreate, adminUserCreate)
router.map(routes.adminUserReset, adminUserReset)
router.map(routes.adminUserDelete, adminUserDelete)
router.map(routes.adminBrokerCreate, adminBrokerCreate)
router.map(routes.adminBrokerRename, adminBrokerRename)
router.map(routes.adminBrokerDelete, adminBrokerDelete)
router.map(routes.adminTokens, adminTokens)
router.map(routes.adminTokenCreate, adminTokenCreate)
router.map(routes.adminTokenDelete, adminTokenDelete)

// Protected user routes — anonymous lands on /login, admin lands on /admin.
router.map(routes.home, withAuth(home))
router.map(routes.holdings, withAuth(holdings))
router.map(routes.stocks, withAuth(stocks))
router.map(routes.stockDetail, withAuth(stockDetail))
router.map(routes.transactions, withAuth(transactions))
router.map(routes.watchlists, withAuth(watchlists))
router.map(routes.news, withAuth(news))
router.map(routes.newsDetail, withAuth(newsDetail))
router.map(routes.briefs, withAuth(briefs))
router.map(routes.earnings, withAuth(earnings))
router.map(routes.macroEvents, withAuth(macroEvents))
router.map(routes.catalysts, withAuth(catalysts))
router.map(routes.screeners, withAuth(screeners))
router.map(routes.recommendations, withAuth(recommendations))
router.map(routes.portfolioReviews, withAuth(portfolioReviews))
router.map(routes.correlations, withAuth(correlations))
router.map(routes.selfExams, withAuth(selfExams))
router.map(routes.audit, withAuth(audit))
router.map(routes.settings, withAuth(settings))
router.map(routes.apiKeys, withAuth(apiKeys))
router.map(routes.apiKeyCreate, withAuth(apiKeyCreate))
router.map(routes.apiKeyDelete, withAuth(apiKeyDelete))
router.map(routes.accounts, withAuth(accounts))
router.map(routes.accountCreate, withAuth(accountCreate))
router.map(routes.accountDelete, withAuth(accountDelete))
