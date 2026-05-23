/// Shared layout for the 9 entity detail pages added for unread state
/// (briefs, earnings, macro events, catalysts, screeners, recommendations,
/// portfolio reviews, correlations, self-exams). Each controller fetches
/// its entity, derives a title + sections + side metadata, and hands
/// them off to `EntityDetailPage` — keeping the per-entity code thin
/// and the visual treatment uniform.
///
/// The dedicated `news-detail.tsx` predates this and stays bespoke; its
/// related-stocks / sectors / macro-indicators sidebar is too custom to
/// fit the generic shape.

import { css, type RemixNode } from 'remix/ui'

import {
  Badge,
  Card,
  color,
  font,
  Layout,
  radius,
  space,
  type Theme,
} from './layout.tsx'
import { MarkdownToggle } from './markdown.tsx'

export interface EntityDetailSection {
  /// Section heading (e.g. "Summary", "Bull case"). Omitted heading just
  /// renders the markdown body without a label.
  label?: string
  /// Markdown source. `null`/empty renders nothing — pass the optional
  /// field directly without a guard.
  markdown: string | null | undefined
}

export interface EntityDetailProps {
  /// Page title — usually the item's headline or a derived label.
  title: string
  /// Optional muted subtitle under the title (date / period / source).
  subtitle?: string
  /// Back-link rendered as a breadcrumb above the title.
  back: { href: string; label: string }
  /// Inline meta strip at the top of the hero card. Pass a row of
  /// `<Badge>`s, plain spans, or any RemixNode — the wrapper only
  /// provides the flex/spacing.
  meta?: RemixNode
  /// Ordered markdown sections in the hero card. Pass undefined-friendly
  /// values; the component skips empty ones automatically.
  sections?: EntityDetailSection[]
  /// Optional side card content (related items, key/value tables, …).
  /// When omitted the hero card spans the full content width.
  side?: RemixNode
  /// Resolved locale for Layout's `lang` attribute.
  locale: string
  /// Resolved theme for Layout's `data-theme`.
  theme: Theme
}

/// Generic detail page used by 9 of the 10 entity types. News keeps its
/// own bespoke layout because it ships a richer sidebar.
export function EntityDetailPage() {
  return ({
    title,
    subtitle,
    back,
    meta,
    sections,
    side,
    locale,
    theme,
  }: EntityDetailProps) => (
    <Layout title={title} locale={locale} theme={theme}>
      <Breadcrumb href={back.href} label={back.label} />
      <div
        mix={css({
          marginTop: space[3],
          display: 'grid',
          gridTemplateColumns: side ? '2fr 1fr' : '1fr',
          gap: space[4],
          '@media (max-width: 880px)': { gridTemplateColumns: '1fr' },
        })}
      >
        <Card>
          {meta && (
            <div
              mix={css({
                display: 'flex',
                flexWrap: 'wrap',
                gap: space[2],
                alignItems: 'center',
                fontSize: font.xs,
                color: color.textMuted,
                marginBottom: space[3],
              })}
            >
              {meta}
            </div>
          )}
          <h1
            mix={css({
              margin: `${space[2]} 0 ${subtitle ? space[1] : space[3]}`,
              fontSize: font.xl,
              fontWeight: 700,
              color: color.text,
              lineHeight: 1.3,
              letterSpacing: '-0.01em',
            })}
          >
            {title}
          </h1>
          {subtitle && (
            <p
              mix={css({
                margin: `0 0 ${space[4]}`,
                fontSize: font.sm,
                color: color.textMuted,
                lineHeight: 1.55,
              })}
            >
              {subtitle}
            </p>
          )}
          {sections?.map((s) =>
            s.markdown ? (
              <div mix={css({ marginBottom: space[4] })}>
                {s.label && (
                  <div
                    mix={css({
                      fontSize: font.xs,
                      fontWeight: 700,
                      textTransform: 'uppercase',
                      letterSpacing: '0.08em',
                      color: color.textMuted,
                      marginBottom: space[2],
                    })}
                  >
                    {s.label}
                  </div>
                )}
                <MarkdownToggle source={s.markdown} />
              </div>
            ) : null,
          )}
        </Card>
        {side && (
          <div
            mix={css({ display: 'flex', flexDirection: 'column', gap: space[4] })}
          >
            {side}
          </div>
        )}
      </div>
    </Layout>
  )
}

function Breadcrumb() {
  return ({ href, label }: { href: string; label: string }) => (
    <div
      mix={css({
        display: 'flex',
        alignItems: 'center',
        gap: space[2],
        fontSize: font.sm,
        color: color.textMuted,
      })}
    >
      <a
        href={href}
        mix={css({
          color: color.textMuted,
          textDecoration: 'none',
          '&:hover': { color: color.text },
        })}
      >
        {label}
      </a>
      <span>·</span>
      <span mix={css({ color: color.text, fontWeight: 500 })}>Detail</span>
    </div>
  )
}

/// Compact `<dl>`-style key/value list for the side card. Pass tuples
/// `[label, value]`; `null`/`undefined` values are skipped so callers
/// can write `['EPS estimate', item.eps_estimate]` without guarding.
export function MetaList() {
  return ({ items }: { items: Array<[string, RemixNode | null | undefined]> }) => {
    let visible = items.filter(([, v]) => v != null && v !== '')
    if (visible.length === 0) return null
    return (
      <Card>
        <dl
          mix={css({
            margin: 0,
            display: 'grid',
            gridTemplateColumns: 'auto 1fr',
            rowGap: space[2],
            columnGap: space[3],
            fontSize: font.sm,
          })}
        >
          {visible.map(([k, v]) => (
            <>
              <dt
                mix={css({
                  color: color.textMuted,
                  fontWeight: 500,
                })}
              >
                {k}
              </dt>
              <dd mix={css({ margin: 0, color: color.text })}>{v}</dd>
            </>
          ))}
        </dl>
      </Card>
    )
  }
}

/// Pill matching the existing Badge style; thin wrapper so detail pages
/// don't have to import Badge directly when only a single chip is needed.
export function ChipText() {
  return ({ children }: { children: RemixNode }) => (
    <Badge tone="neutral">{children}</Badge>
  )
}
