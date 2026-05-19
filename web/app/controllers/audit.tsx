import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { api } from '../api.ts'
import type { routes } from '../routes.ts'
import { Layout, resolveLocale } from '../ui/layout.tsx'
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
    let rows = (await api.audit().catch(() => [])) as AuditRow[]
    return render(<AuditPage rows={rows} locale={locale} />, request, { locale })
  },
}

function AuditPage() {
  return ({ rows, locale }: { rows: AuditRow[]; locale: string }) => (
    <Layout title="Audit log" locale={locale}>
      {rows.length === 0 ? (
        <p mix={css({ color: '#64748b' })}>
          No audit entries yet. (Audit writes from handlers are wired up in a follow-up; for now
          the table holds the spec but isn't populated by every endpoint.)
        </p>
      ) : (
        <table
          mix={css({
            width: '100%',
            borderCollapse: 'collapse',
            background: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '8px',
            fontSize: '13px',
          })}
        >
          <thead mix={css({ background: '#f1f5f9' })}>
            <tr>
              <th mix={css({ textAlign: 'left', padding: '10px 14px' })}>When</th>
              <th mix={css({ textAlign: 'left', padding: '10px 14px' })}>Action</th>
              <th mix={css({ textAlign: 'left', padding: '10px 14px' })}>Entity</th>
              <th mix={css({ textAlign: 'left', padding: '10px 14px' })}>Actor</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr mix={css({ borderTop: '1px solid #e2e8f0' })}>
                <td mix={css({ padding: '10px 14px', color: '#475569' })}>{r.created_at}</td>
                <td mix={css({ padding: '10px 14px' })}>{r.action}</td>
                <td mix={css({ padding: '10px 14px' })}>
                  <code>{r.entity_type}</code>/<code>{r.entity_id}</code>
                </td>
                <td mix={css({ padding: '10px 14px' })}>{r.actor_label}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </Layout>
  )
}
