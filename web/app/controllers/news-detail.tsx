import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import {
  api,
  type NewsCountryLink,
  type NewsItem,
  type NewsMacroLink,
  type NewsSectorLink,
  type NewsStockLink,
  type NewsTranslation,
  type Sector,
  type Stock,
} from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

export const newsDetail: BuildAction<'GET', typeof routes.newsDetail> = {
  async handler({ request, params }) {
    let id = Number.parseInt(params.id, 10)
    if (!Number.isFinite(id)) return new Response('Bad news id', { status: 400 })
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)

    let [item, stockLinks, sectorLinks, macroLinks, countryLinks, translations, stocks, sectors] =
      await Promise.all([
        // `locale` flows through to the server which merges the matching
        // news_translations row over title/summary/content_md/agent_summary_md.
        api.newsItem(id, locale).catch(() => null),
        api.newsStockLinks(id).catch(() => [] as NewsStockLink[]),
        api.newsSectorLinks(id).catch(() => [] as NewsSectorLink[]),
        api.newsMacroLinks(id).catch(() => [] as NewsMacroLink[]),
        api.newsCountryLinks(id).catch(() => [] as NewsCountryLink[]),
        api.newsTranslations(id).catch(() => [] as NewsTranslation[]),
        api.stocks().catch(() => [] as Stock[]),
        api.sectors().catch(() => [] as Sector[]),
      ])
    if (!item) return new Response('News not found', { status: 404 })

    let stockMap = new Map<number, Stock>(stocks.map((s) => [s.id, s]))
    let sectorMap = new Map<string, Sector>(sectors.map((s) => [s.code, s]))

    return render(
      <NewsDetailPage
        item={item}
        stockLinks={stockLinks}
        sectorLinks={sectorLinks}
        macroLinks={macroLinks}
        countryLinks={countryLinks}
        translations={translations}
        locale={locale}
        stocks={stockMap}
        sectors={sectorMap}
      />,
      request,
      { locale },
    )
  },
}

interface NewsDetailProps {
  item: NewsItem
  stockLinks: NewsStockLink[]
  sectorLinks: NewsSectorLink[]
  macroLinks: NewsMacroLink[]
  countryLinks: NewsCountryLink[]
  translations: NewsTranslation[]
  /** Active locale from the global switcher in Layout. */
  locale: string
  stocks: Map<number, Stock>
  sectors: Map<string, Sector>
}

function NewsDetailPage() {
  return ({
    item: n,
    stockLinks,
    sectorLinks,
    macroLinks,
    countryLinks,
    translations,
    locale,
    stocks,
    sectors,
  }: NewsDetailProps) => {
    // Server already applied the translation for `locale` into n's display
    // fields. We just need to know whether one existed so the provenance
    // banner can be rendered.
    let chosen = translations.find((t) => t.locale === locale) ?? null
    let isOriginal = locale === n.language
    let missingTranslation = !isOriginal && !chosen

    return (
    <Layout title={n.title} locale={locale}>
      <a
        href="/news"
        mix={css({
          fontSize: '13px',
          color: '#64748b',
          textDecoration: 'none',
          '&:hover': { color: '#0f172a' },
        })}
      >
        ← Back to news
      </a>

      {/* Meta strip */}
      <div
        mix={css({
          marginTop: '12px',
          display: 'flex',
          flexWrap: 'wrap',
          gap: '8px',
          alignItems: 'center',
          fontSize: '12px',
          color: '#64748b',
        })}
      >
        <strong mix={css({ color: '#0f172a' })}>{n.source}</strong>
        <span>·</span>
        <span>{fmtDate(n.published_at)}</span>
        <span>·</span>
        <span>{n.language}</span>
        <span>·</span>
        <span>region {n.region}</span>
        <span>·</span>
        <span>importance {n.importance}</span>
        {n.sentiment && (
          <>
            <span>·</span>
            <span>sentiment {n.sentiment} {n.sentiment_score ? `(${n.sentiment_score})` : ''}</span>
          </>
        )}
      </div>

      {/* Two-column body */}
      <div
        mix={css({
          marginTop: '16px',
          display: 'grid',
          gridTemplateColumns: '2fr 1fr',
          gap: '16px',
          '@media (max-width: 720px)': { gridTemplateColumns: '1fr' },
        })}
      >
        <Card>
          {/* Translation provenance + missing-translation hint. The locale
              itself is controlled by the global switcher in Layout. */}
          {chosen ? (
            <div
              mix={css({
                fontSize: '11px',
                color: '#94a3b8',
              })}
            >
              translation ({chosen.locale}) by {chosen.translator} · updated{' '}
              {chosen.updated_at.slice(0, 10)}
            </div>
          ) : isOriginal ? (
            <div
              mix={css({
                fontSize: '11px',
                color: '#94a3b8',
              })}
            >
              original language: {n.language}
            </div>
          ) : missingTranslation ? (
            <div
              mix={css({
                padding: '8px 10px',
                background: '#fef9c3',
                borderRadius: '4px',
                fontSize: '12px',
                color: '#713f12',
              })}
            >
              No translation for <code>{locale}</code> yet — showing original ({n.language}).
              Add one with{' '}
              <code>{`PUT /api/v1/news/${n.id}/translations/${locale}`}</code>.
            </div>
          ) : null}

          <div
            mix={css({
              marginTop: '14px',
              fontSize: '20px',
              fontWeight: 700,
              color: '#0f172a',
              lineHeight: 1.35,
            })}
          >
            {n.title}
          </div>

          {n.summary && (
            <p
              mix={css({
                margin: '12px 0 14px',
                fontSize: '14px',
                color: '#475569',
                lineHeight: 1.6,
                fontStyle: 'italic',
              })}
            >
              {n.summary}
            </p>
          )}
          {n.agent_summary_md && (
            <div
              mix={css({
                margin: '0 0 14px',
                padding: '12px 14px',
                background: '#eef2ff',
                borderLeft: '3px solid #1d4ed8',
                borderRadius: '4px',
                fontSize: '13px',
                color: '#3730a3',
              })}
            >
              <div
                mix={css({
                  fontSize: '10px',
                  fontWeight: 700,
                  textTransform: 'uppercase',
                  letterSpacing: '0.08em',
                  marginBottom: '4px',
                })}
              >
                Agent take
              </div>
              <BodyText text={n.agent_summary_md} />
            </div>
          )}
          {n.content_md ? (
            <BodyText text={n.content_md} />
          ) : (
            <p
              mix={css({
                color: '#94a3b8',
                fontStyle: 'italic',
                fontSize: '13px',
              })}
            >
              (no full content stored — follow the original link)
            </p>
          )}

          <Links item={n} />
        </Card>

        <Card>
          <SectionTitle>Related stocks</SectionTitle>
          {stockLinks.length === 0 ? (
            <Dim>none</Dim>
          ) : (
            <ul mix={css({ listStyle: 'none', margin: 0, padding: 0 })}>
              {stockLinks.map((l) => {
                let s = stocks.get(l.stock_id)
                return (
                  <li mix={css({ marginBottom: '6px' })}>
                    {s ? (
                      <a
                        href={`/stocks/${s.id}`}
                        mix={css({
                          color: '#1d4ed8',
                          textDecoration: 'none',
                          fontWeight: 600,
                          fontFamily: 'ui-monospace, monospace',
                          '&:hover': { textDecoration: 'underline' },
                        })}
                      >
                        {s.symbol}
                      </a>
                    ) : (
                      <span>#{l.stock_id}</span>
                    )}
                    <span
                      mix={css({
                        marginLeft: '8px',
                        fontSize: '11px',
                        color: '#64748b',
                      })}
                    >
                      {l.relation}
                      {l.relevance ? ` · ${l.relevance}` : ''}
                    </span>
                  </li>
                )
              })}
            </ul>
          )}

          <SectionTitle>Sectors</SectionTitle>
          {sectorLinks.length === 0 ? (
            <Dim>none</Dim>
          ) : (
            <ChipRow>
              {sectorLinks.map((l) => {
                let s = sectors.get(l.sector_code)
                return (
                  <Chip>
                    {l.sector_code}
                    {s ? ` · ${s.name}` : ''}
                  </Chip>
                )
              })}
            </ChipRow>
          )}

          <SectionTitle>Macro indicators</SectionTitle>
          {macroLinks.length === 0 ? (
            <Dim>none</Dim>
          ) : (
            <ChipRow>
              {macroLinks.map((l) => (
                <Chip>{l.indicator_code}</Chip>
              ))}
            </ChipRow>
          )}

          <SectionTitle>Countries</SectionTitle>
          {countryLinks.length === 0 ? (
            <Dim>none</Dim>
          ) : (
            <ChipRow>
              {countryLinks.map((l) => (
                <Chip>{l.country}</Chip>
              ))}
            </ChipRow>
          )}
        </Card>
      </div>
    </Layout>
    )
  }
}

function Links() {
  return ({ item: n }: { item: NewsItem }) => (
    <div
      mix={css({
        marginTop: '20px',
        paddingTop: '16px',
        borderTop: '1px solid #e2e8f0',
        fontSize: '12px',
        color: '#64748b',
        display: 'flex',
        flexDirection: 'column',
        gap: '4px',
      })}
    >
      <div>
        Source URL:{' '}
        <a
          href={n.url}
          target="_blank"
          rel="noopener noreferrer"
          mix={css({ color: '#1d4ed8' })}
        >
          {n.url}
        </a>
        {n.url_status && (
          <span mix={css({ marginLeft: '8px', color: '#94a3b8' })}>
            HTTP {n.url_status}
          </span>
        )}
      </div>
      {n.canonical_url && n.canonical_url !== n.url && (
        <div>
          Canonical:{' '}
          <a href={n.canonical_url} target="_blank" rel="noopener noreferrer"
            mix={css({ color: '#1d4ed8' })}>
            {n.canonical_url}
          </a>
        </div>
      )}
      {n.archive_url && (
        <div>
          Archive:{' '}
          <a href={n.archive_url} target="_blank" rel="noopener noreferrer"
            mix={css({ color: '#1d4ed8' })}>
            {n.archive_url}
          </a>
        </div>
      )}
      {n.last_verified_at && (
        <div>last verified {fmtDate(n.last_verified_at)}</div>
      )}
    </div>
  )
}

function fmtDate(iso: string): string {
  return iso.slice(0, 16).replace('T', ' ')
}

function Card() {
  return ({ children }: { children: import('remix/ui').RemixNode }) => (
    <div
      mix={css({
        background: '#fff',
        borderRadius: '8px',
        padding: '20px',
        border: '1px solid #e2e8f0',
        boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
      })}
    >
      {children}
    </div>
  )
}

function SectionTitle() {
  return ({ children }: { children: string }) => (
    <h3
      mix={css({
        margin: '16px 0 8px',
        fontSize: '11px',
        fontWeight: 600,
        textTransform: 'uppercase',
        letterSpacing: '0.08em',
        color: '#64748b',
      })}
    >
      {children}
    </h3>
  )
}

function Dim() {
  return ({ children }: { children: string }) => (
    <span mix={css({ fontSize: '13px', color: '#94a3b8', fontStyle: 'italic' })}>
      {children}
    </span>
  )
}

function ChipRow() {
  return ({ children }: { children: import('remix/ui').RemixNode }) => (
    <div mix={css({ display: 'flex', flexWrap: 'wrap', gap: '6px' })}>{children}</div>
  )
}

function Chip() {
  return ({ children }: { children: import('remix/ui').RemixNode }) => (
    <span
      mix={css({
        padding: '2px 8px',
        background: '#f1f5f9',
        color: '#475569',
        borderRadius: '999px',
        fontSize: '11px',
        fontWeight: 500,
      })}
    >
      {children}
    </span>
  )
}

function BodyText() {
  return ({ text }: { text: string }) => (
    <pre
      mix={css({
        margin: 0,
        fontSize: '14px',
        lineHeight: 1.65,
        color: '#1f2937',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
        fontFamily: 'inherit',
      })}
    >
      {text}
    </pre>
  )
}
