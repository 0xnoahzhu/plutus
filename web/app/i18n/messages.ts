/// Static UI string table, keyed by locale.
///
/// Lookup: `let m = messages(locale); m.nav.holdings`. The `Messages` type
/// is inferred from the `en` table so every other locale has to provide
/// the full set — TypeScript flags missing keys at compile time.
///
/// Translatable *data* (agent output, brief content, etc.) is handled
/// separately by the storage layer's per-locale JSON projection — this
/// module is only for hardcoded UI chrome.

export type Locale = 'en' | 'zh-CN'

const en = {
  nav: {
    dashboard: 'Dashboard',
    holdings: 'Holdings',
    stocks: 'Stocks',
    transactions: 'Transactions',
    watchlist: 'Watchlist',
    news: 'News',
    briefs: 'Briefs',
    earnings: 'Earnings',
    macro: 'Macro',
    catalysts: 'Catalysts',
    screeners: 'Screeners',
    recommendations: 'Recommendations',
    reviews: 'Reviews',
    correlations: 'Correlations',
    selfExam: 'Self-Exam',
    audit: 'Audit',
    settings: 'Settings',
    sectionCalendar: 'Calendar',
    sectionAnalysis: 'Analysis',
    signOut: 'Sign out',
  },

  pages: {
    dashboard: { title: 'Dashboard', subtitle: "Today's snapshot" },
    holdings: { title: 'Holdings' },
    stocks: { title: 'Stocks' },
    transactions: { title: 'Transactions' },
    watchlist: { title: 'Watchlist' },
    news: { title: 'News' },
    newsDetail: { title: 'News' },
    briefs: { title: 'Market Briefs' },
    earnings: { title: 'Earnings' },
    macroEvents: { title: 'Macro calendar' },
    catalysts: { title: 'Catalysts' },
    screeners: { title: 'Screeners' },
    recommendations: { title: 'Recommendations' },
    portfolioReviews: { title: 'Portfolio reviews' },
    correlations: { title: 'Correlation map' },
    selfExams: { title: 'Self-exam' },
    audit: { title: 'Audit log', subtitle: 'Server-side write log' },
    settings: {
      title: 'Settings',
      subtitle: 'Local preferences. Persisted via cookies, applied per request.',
    },
  },

  auth: {
    login: {
      title: 'Sign in',
      username: 'Username',
      password: 'Password',
      submit: 'Sign in',
      errBadCredentials: 'Wrong username or password.',
      errMissing: 'Enter your username and password.',
      errServer: 'Login failed.',
    },
    changePassword: {
      title: 'Change password',
      hintForced:
        'Your administrator reset your password. Choose a new one to continue.',
      hintOptional: 'Enter your current password and a new one.',
      current: 'Current password',
      next: 'New password',
      confirm: 'Confirm new password',
      submit: 'Update password',
      errWrongCurrent: 'Current password is incorrect.',
      errMismatch: 'New passwords do not match.',
      errMissing: 'Fill in all fields.',
      errForbidden: 'Sign in again to change your password.',
      errServer: 'Password change failed.',
    },
  },

  admin: {
    title: 'Admin',
    subtitle:
      'Manage end-user accounts. Admin credentials live in env vars, not the database.',
    createSection: 'Create user',
    createUsername: 'username',
    createPassword: 'initial password',
    createSubmit: 'Create',
    usersSection: 'Users',
    emptyTitle: 'No users yet',
    emptyHint: 'Create the first user above.',
    resetBadge: 'reset pending',
    resetPlaceholder: 'new temp password',
    resetSubmit: 'Reset password',
    deleteSubmit: 'Delete',
    flashCreated: 'User created.',
    flashReset:
      'Password reset. The user will be forced to change it on next login.',
    flashDeleted: 'User deleted.',
    errMissingCreate: 'Username and password are required.',
    errMissingReset: 'New password is required.',
    errBadId: 'Bad user id.',
    errTaken: 'That username is already taken (or matches the admin name).',
    errForbidden: 'Admin privileges required.',
    errNotFound: 'User not found.',
    errServer: 'Request failed.',
  },

  settings: {
    colorScheme: {
      title: 'Color scheme',
      description:
        '**System** follows the OS `prefers-color-scheme` setting. **Dark** and **Light** pin the palette regardless.',
      system: 'System',
      dark: 'Dark',
      light: 'Light',
    },
    language: {
      title: 'Language',
      description:
        'Controls which translation gets rendered on every agent-output row. The base columns stay English; zh-CN is layered on via the `translations` JSON field on each record.',
    },
  },
}

/// Inferred from the English table — used to constrain other locales so
/// missing keys are caught at compile time. Exported for components that
/// want to type a sub-tree (e.g. layout's `buildNav` takes `Messages`).
export type Messages = typeof en

const zhCN: Messages = {
  nav: {
    dashboard: '仪表盘',
    holdings: '持仓',
    stocks: '股票',
    transactions: '交易',
    watchlist: '自选股',
    news: '新闻',
    briefs: '每日简报',
    earnings: '财报',
    macro: '宏观',
    catalysts: '催化剂',
    screeners: '选股',
    recommendations: '投资建议',
    reviews: '组合复盘',
    correlations: '相关性',
    selfExam: '自我复盘',
    audit: '审计日志',
    settings: '设置',
    sectionCalendar: '日历',
    sectionAnalysis: '分析',
    signOut: '退出登录',
  },

  pages: {
    dashboard: { title: '仪表盘', subtitle: '今日概览' },
    holdings: { title: '持仓' },
    stocks: { title: '股票' },
    transactions: { title: '交易' },
    watchlist: { title: '自选股' },
    news: { title: '新闻' },
    newsDetail: { title: '新闻' },
    briefs: { title: '盘前盘后简报' },
    earnings: { title: '财报日历' },
    macroEvents: { title: '宏观日历' },
    catalysts: { title: '催化剂事件' },
    screeners: { title: '选股' },
    recommendations: { title: '投资建议' },
    portfolioReviews: { title: '组合复盘' },
    correlations: { title: '相关性矩阵' },
    selfExams: { title: '自我复盘' },
    audit: { title: '审计日志', subtitle: '服务端写入日志' },
    settings: {
      title: '设置',
      subtitle: '本地偏好。通过 cookie 持久化，每次请求时应用。',
    },
  },

  auth: {
    login: {
      title: '登录',
      username: '用户名',
      password: '密码',
      submit: '登录',
      errBadCredentials: '用户名或密码错误。',
      errMissing: '请输入用户名和密码。',
      errServer: '登录失败。',
    },
    changePassword: {
      title: '修改密码',
      hintForced: '管理员已重置你的密码，请设置新密码后继续。',
      hintOptional: '请输入当前密码和新密码。',
      current: '当前密码',
      next: '新密码',
      confirm: '确认新密码',
      submit: '更新密码',
      errWrongCurrent: '当前密码不正确。',
      errMismatch: '两次输入的新密码不一致。',
      errMissing: '请填写所有字段。',
      errForbidden: '请重新登录后再修改密码。',
      errServer: '修改密码失败。',
    },
  },

  admin: {
    title: '管理员',
    subtitle: '管理终端用户账号。管理员凭证存放于环境变量，不入库。',
    createSection: '创建用户',
    createUsername: '用户名',
    createPassword: '初始密码',
    createSubmit: '创建',
    usersSection: '用户列表',
    emptyTitle: '暂无用户',
    emptyHint: '请在上方创建第一个用户。',
    resetBadge: '待重置',
    resetPlaceholder: '新临时密码',
    resetSubmit: '重置密码',
    deleteSubmit: '删除',
    flashCreated: '用户已创建。',
    flashReset: '密码已重置。用户下次登录时将被要求修改。',
    flashDeleted: '用户已删除。',
    errMissingCreate: '请填写用户名和密码。',
    errMissingReset: '请填写新密码。',
    errBadId: '用户 ID 无效。',
    errTaken: '该用户名已被使用（或与管理员账号冲突）。',
    errForbidden: '需要管理员权限。',
    errNotFound: '用户不存在。',
    errServer: '请求失败。',
  },

  settings: {
    colorScheme: {
      title: '配色方案',
      description:
        '**跟随系统** 使用操作系统的 `prefers-color-scheme` 设置。**深色** 与 **浅色** 强制对应配色。',
      system: '跟随系统',
      dark: '深色',
      light: '浅色',
    },
    language: {
      title: '语言',
      description:
        '控制所有 agent 输出行的翻译展示。基础列保持英文，zh-CN 通过每条记录的 `translations` JSON 字段覆盖。',
    },
  },
}

const tables: Record<Locale, Messages> = {
  en,
  'zh-CN': zhCN,
}

export function messages(locale: string): Messages {
  return tables[locale as Locale] ?? en
}
