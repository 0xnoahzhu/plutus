import { createRouter } from 'remix/fetch-router'

import { assets } from './assets.ts'
import { audit } from './controllers/audit.tsx'
import { briefs } from './controllers/briefs.tsx'
import { catalysts } from './controllers/catalysts.tsx'
import { correlations } from './controllers/correlations.tsx'
import { earnings } from './controllers/earnings.tsx'
import { holdings } from './controllers/holdings.tsx'
import { home } from './controllers/home.tsx'
import { macroEvents } from './controllers/macro-events.tsx'
import { news } from './controllers/news.tsx'
import { newsDetail } from './controllers/news-detail.tsx'
import { portfolioReviews } from './controllers/portfolio-reviews.tsx'
import { recommendations } from './controllers/recommendations.tsx'
import { screeners } from './controllers/screeners.tsx'
import { selfExams } from './controllers/self-exams.tsx'
import { stockDetail } from './controllers/stock-detail.tsx'
import { stocks } from './controllers/stocks.tsx'
import { transactions } from './controllers/transactions.tsx'
import { watchlistDetail } from './controllers/watchlist-detail.tsx'
import { watchlists } from './controllers/watchlists.tsx'
import { routes } from './routes.ts'

export const router = createRouter()

router.get(routes.assets, async ({ request }) => {
  let response = await assets.fetch(request)
  return response ?? new Response('Not Found', { status: 404 })
})

router.map(routes.home, home)
router.map(routes.holdings, holdings)
router.map(routes.stocks, stocks)
router.map(routes.stockDetail, stockDetail)
router.map(routes.transactions, transactions)
router.map(routes.watchlists, watchlists)
router.map(routes.watchlistDetail, watchlistDetail)
router.map(routes.news, news)
router.map(routes.newsDetail, newsDetail)
router.map(routes.briefs, briefs)
router.map(routes.earnings, earnings)
router.map(routes.macroEvents, macroEvents)
router.map(routes.catalysts, catalysts)
router.map(routes.screeners, screeners)
router.map(routes.recommendations, recommendations)
router.map(routes.portfolioReviews, portfolioReviews)
router.map(routes.correlations, correlations)
router.map(routes.selfExams, selfExams)
router.map(routes.audit, audit)
