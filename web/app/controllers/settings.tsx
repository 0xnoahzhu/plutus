import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import type { routes } from '../routes.ts'
import {
  Card,
  color,
  font,
  Layout,
  type Locale,
  LocaleChips,
  resolveLocale,
  resolveTheme,
  SectionTitle,
  space,
  type Theme,
  ThemeChips,
} from '../ui/layout.tsx'
import { render } from '../utils/render.tsx'

/// User preferences page. Hosts the Language + Color scheme switchers
/// that used to live in the page header. Each chip is still a plain
/// anchor — clicking writes `?locale=` / `?theme=` to the URL, the
/// handler picks it up via [[resolveLocale]] / [[resolveTheme]], the
/// matching cookie is set, and the next navigation honors it.
export const settings: BuildAction<'GET', typeof routes.settings> = {
  async handler({ request }) {
    let url = new URL(request.url)
    let locale = resolveLocale(request, url.searchParams)
    let theme = resolveTheme(request, url.searchParams)
    return render(
      <SettingsPage locale={locale} theme={theme} />,
      request,
      { locale, theme },
    )
  },
}

interface SettingsProps {
  locale: Locale
  theme: Theme
}

function SettingsPage() {
  return ({ locale, theme }: SettingsProps) => (
    <Layout
      title="Settings"
      subtitle="Local preferences. Persisted via cookies, applied per request."
      locale={locale}
      theme={theme}
    >
      <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[5] })}>
        <Card>
          <SectionTitle>Language</SectionTitle>
          <p
            mix={css({
              margin: `0 0 ${space[4]}`,
              fontSize: font.sm,
              color: color.textMuted,
              lineHeight: 1.5,
            })}
          >
            Controls which translation gets rendered on every agent-output row.
            The base columns stay English; zh-CN is layered on via the
            <code mix={css({ marginLeft: '4px', marginRight: '4px' })}>translations</code>
            JSON field on each record.
          </p>
          <LocaleChips selected={locale} />
        </Card>

        <Card>
          <SectionTitle>Color scheme</SectionTitle>
          <p
            mix={css({
              margin: `0 0 ${space[4]}`,
              fontSize: font.sm,
              color: color.textMuted,
              lineHeight: 1.5,
            })}
          >
            <strong mix={css({ color: color.text })}>System</strong> follows the OS{' '}
            <code>prefers-color-scheme</code> setting.{' '}
            <strong mix={css({ color: color.text })}>Dark</strong> and{' '}
            <strong mix={css({ color: color.text })}>Light</strong> pin the
            palette regardless.
          </p>
          <ThemeChips selected={theme} locale={locale} />
        </Card>
      </div>
    </Layout>
  )
}
