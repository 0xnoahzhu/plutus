import { get, route } from 'remix/fetch-router/routes'

export const routes = route({
  assets: get('/assets/*path'),
  home: '/',
  holdings: '/holdings',
  stocks: '/stocks',
  stockDetail: '/stocks/:id',
  watchlists: '/watchlists',
  watchlistDetail: '/watchlists/:id',
  transactions: '/transactions',
  news: '/news',
  newsDetail: '/news/:id',
  briefs: '/briefs',
  earnings: '/earnings',
  macroEvents: '/macro-events',
  catalysts: '/catalysts',
  screeners: '/screeners',
  recommendations: '/recommendations',
  portfolioReviews: '/portfolio-reviews',
  correlations: '/correlations',
  selfExams: '/self-exams',
  audit: '/audit',
})
