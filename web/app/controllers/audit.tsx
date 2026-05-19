import type { BuildAction } from 'remix/fetch-router'
import { css, type RemixNode } from 'remix/ui'

import { api } from '../api.ts'
import { messages } from '../i18n/messages.ts'
import type { routes } from '../routes.ts'
import {
  Badge,
  type BadgeTone,
  Card,
  color,
  EmptyState,
  font,
  Layout,
  resolveLocale,
  resolveTheme,
  space,
  type Theme,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

interface AuditRow {
  id: number
  entity_type: string
  entity_id: string
  action: string
  actor_kind: string
  actor_label: string
  request_id: string
  created_at: string
}

export const audit: BuildAction<'GET', typeof routes.audit> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    let rows = (await api.audit().catch(() => [])) as AuditRow[]
    return render(<AuditPage rows={rows} locale={locale} theme={theme} />, request, { locale, theme })
  },
}

function AuditPage() {
  return ({ rows, locale, theme }: { rows: AuditRow[]; locale: string; theme: Theme }) => {
    let p = messages(locale).pages.audit
    return (
    <Layout title={p.title} subtitle={p.subtitle} locale={locale} theme={theme}>
      {rows.length === 0 ? (
        <Card>
          <EmptyState
            title="No audit entries yet"
            hint="Audit writes from handlers are wired up in a follow-up; for now the table holds the spec but isn't populated by every endpoint."
          />
        </Card>
      ) : (
        <Card padding="0">
          <table
            mix={css({
              width: '100%',
              borderCollapse: 'collapse',
              fontSize: font.base,
            })}
          >
            <thead>
              <tr>
                <Th>When</Th>
                <Th>Action</Th>
                <Th>Entity</Th>
                <Th>Actor</Th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => (
                <tr
                  mix={css({
                    borderTop: `1px solid ${color.borderSoft}`,
                    '&:hover td': { background: color.bg },
                  })}
                >
                  <Td>
                    <span mix={css({ color: color.textMuted })}>{r.created_at}</span>
                  </Td>
                  <Td>
                    <Badge tone={actionTone(r.action)}>{r.action}</Badge>
                  </Td>
                  <Td>
                    <code
                      mix={css({
                        fontFamily: font.mono,
                        fontSize: font.sm,
                        color: color.textMuted,
                      })}
                    >
                      {r.entity_type}
                    </code>
                    <span mix={css({ color: color.textDim })}>/</span>
                    <code
                      mix={css({
                        fontFamily: font.mono,
                        fontSize: font.sm,
                        color: color.textMuted,
                      })}
                    >
                      {r.entity_id}
                    </code>
                  </Td>
                  <Td>
                    <span mix={css({ color: color.text })}>{r.actor_label}</span>
                    <span
                      mix={css({
                        marginLeft: space[2],
                        fontSize: font.xs,
                        color: color.textDim,
                      })}
                    >
                      {r.actor_kind}
                    </span>
                  </Td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}
    </Layout>
    )
  }
}

/// Map common action verbs to a Badge tone. Falls back to neutral so unknown
/// values still render cleanly without a special case at the call site.
function actionTone(action: string): BadgeTone {
  let a = action.toLowerCase()
  if (a.includes('create') || a.includes('insert') || a.includes('add')) return 'success'
  if (a.includes('delete') || a.includes('remove')) return 'danger'
  if (a.includes('update') || a.includes('edit') || a.includes('patch')) return 'info'
  return 'neutral'
}

function Th() {
  return ({
    children,
    align = 'left',
  }: {
    children: RemixNode
    align?: 'left' | 'right'
  }) => (
    <th
      mix={css({
        textAlign: align,
        padding: `${space[3]} ${space[4]}`,
        fontSize: font.xs,
        textTransform: 'uppercase',
        letterSpacing: '0.08em',
        color: color.textMuted,
        fontWeight: 600,
        background: color.bg,
        borderBottom: `1px solid ${color.border}`,
      })}
    >
      {children}
    </th>
  )
}

function Td() {
  return ({
    children,
    align = 'left',
    mono,
  }: {
    children: RemixNode
    align?: 'left' | 'right'
    mono?: boolean
  }) => (
    <td
      mix={css({
        padding: `${space[3]} ${space[4]}`,
        textAlign: align,
        fontVariantNumeric: 'tabular-nums',
        fontFamily: mono ? font.mono : 'inherit',
        color: color.text,
      })}
    >
      {children}
    </td>
  )
}
