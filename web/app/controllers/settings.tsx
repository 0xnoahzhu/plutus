import type { BuildAction } from 'remix/fetch-router'
import { css } from 'remix/ui'

import { messages } from '../i18n/messages.ts'
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
  return ({ locale, theme }: SettingsProps) => {
    let m = messages(locale)
    return (
      <Layout
        title={m.pages.settings.title}
        subtitle={m.pages.settings.subtitle}
        locale={locale}
        theme={theme}
      >
        <div mix={css({ display: 'flex', flexDirection: 'column', gap: space[5] })}>
          <Card>
            <SectionTitle>{m.settings.colorScheme.title}</SectionTitle>
            <p
              mix={css({
                margin: `0 0 ${space[4]}`,
                fontSize: font.sm,
                color: color.textMuted,
                lineHeight: 1.5,
              })}
            >
              {m.settings.colorScheme.description}
            </p>
            <ThemeChips selected={theme} locale={locale} />
          </Card>

          <Card>
            <SectionTitle>{m.settings.language.title}</SectionTitle>
            <p
              mix={css({
                margin: `0 0 ${space[4]}`,
                fontSize: font.sm,
                color: color.textMuted,
                lineHeight: 1.5,
              })}
            >
              {m.settings.language.description}
            </p>
            <LocaleChips selected={locale} />
          </Card>
        </div>
      </Layout>
    )
  }
}
