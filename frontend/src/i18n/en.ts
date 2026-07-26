const en = {
  // App
  appTitle: 'Port Forward Manager',
  tabRules: 'Rules',
  tabLogs: 'Logs',
  tabSettings: 'Settings',

  // RulesPage — section
  rulesHeading: 'Port Forwarding Rules',
  btnAddRule: 'Add Rule',
  btnEditRule: 'Edit Rule',
  emptyRules: 'No rules yet. Click "Add Rule" to get started.',

  // RulesPage — dialog form
  dialogAddTitle: 'Add Rule',
  dialogEditTitle: 'Edit Rule',
  fieldName: 'Name',
  placeholderName: 'My service',
  fieldLocalPort: 'Local Port',
  fieldProtocol: 'Protocol',
  optAuto: 'Auto Detect',
  optHttp: 'HTTP',
  optTcp: 'TCP',
  fieldRemoteHost: 'Remote Host',
  placeholderRemoteHost: '192.168.1.10',
  fieldRemotePort: 'Remote Port',
  fieldEnabled: 'Enabled',
  fieldLogTraffic: 'Log Traffic',
  fieldLogBody: 'Log Body',
  btnCancel: 'Cancel',
  btnSaving: 'Saving…',
  btnUpdate: 'Update',
  btnCreate: 'Create',

  // RulesPage — table headers
  colName: 'Name',
  colForwarding: 'Forwarding',
  colProtocol: 'Protocol',
  colEnabled: 'Enabled',
  colLog: 'Log',
  colActions: 'Actions',

  // RulesPage — table cells
  logStatusBody: 'body',
  logStatusHeaders: 'headers',
  logStatusOff: 'off',

  // RulesPage — delete dialog
  confirmDeleteTitle: 'Delete rule "{{name}}"?',
  btnConfirmDelete: 'Confirm Delete',

  // RulesPage — toasts
  toastRuleUpdated: 'Rule updated',
  toastRuleCreated: 'Rule created',
  toastRuleDeleted: 'Rule deleted',
  errLocalPort: 'Local port must be 1–65535',
  errRemoteHost: 'Remote host is required',
  errRemotePort: 'Remote port must be 1–65535',
  toastPortConflict: 'Port {{port}} is already in use by another process: {{error}}',
  badgePortConflict: 'Port conflict',
  badgePortConflictTooltip: 'This rule is enabled but not listening — {{error}}',

  // LogsPage — filter bar
  filterAllRules: 'All rules',
  filterPort: 'Port {{port}}',
  labelLive: 'Live',
  statusLive: 'Live',
  statusReconnecting: 'Reconnecting...',
  btnRefresh: 'Refresh',
  btnClearLogs: 'Clear Logs',
  totalCount: '{{count}} total',

  // LogsPage — empty / loading
  logsLoading: 'Loading…',
  logsEmpty: 'No logs yet. Enable a rule and make some requests.',

  // LogsPage — table headers
  colTime: 'Time',
  colRule: 'Rule',
  colProto: 'Proto',
  colMethod: 'Method',
  colPathPreview: 'Path / Preview',
  colStatus: 'Status',
  colSize: 'Size',
  colDuration: 'Duration',

  // LogsPage — table cells / detail
  fallbackRule: 'Rule #{{id}}',
  tcpPreview: 'TCP Preview',
  noPreview: '(no preview captured)',
  reqHeaders: 'Request Headers',
  respHeaders: 'Response Headers',
  reqBody: 'Request Body',
  respBody: 'Response Body',
  sectionEmpty: '(empty)',

  // LogsPage — confirm + toast
  confirmClearLogs: 'Clear all logs',
  confirmClearLogsRule: 'Clear all logs for this rule',
  toastDeleted: 'Deleted {{count}} log',
  toastDeleted_plural: 'Deleted {{count}} logs',

  // LogsPage — pagination
  btnPrev: 'Prev',
  btnNext: 'Next',
  pageOf: 'Page {{page}} of {{total}}',

  // SettingsPage — headings
  settingsLogPolicy: 'Log Cleanup Policy',
  settingsRuleDefaults: 'New Rule Defaults',
  settingsRuntime: 'Runtime Info (Read-only)',

  // SettingsPage — fields
  fieldMaxRows: 'Max Retained Rows',
  hintMaxRows: '0 = unlimited; oldest logs auto-deleted hourly when exceeded',
  fieldTtlDays: 'Retention Days (TTL)',
  hintTtlDays: '0 = unlimited; logs older than this are auto-deleted hourly',
  fieldDefaultProtocol: 'Default Protocol',
  fieldDefaultLogEnabled: 'Enable request logging by default',
  fieldDefaultLogBody: 'Log request body by default',

  // SettingsPage — buttons / toasts / loading
  btnSaveSettings: 'Save Settings',
  btnSavingSettings: 'Saving...',
  toastSettingsSaved: 'Settings saved',
  settingsLoading: 'Loading...',

  // SettingsPage — runtime info
  labelListenAddr: 'Listen Address',
  labelDbPath: 'Database Path',
  hintRuntime: 'Configured via environment variables; service restart required to apply changes.',

  // Language switcher
  langSwitchLabel: 'Language',
  langEn: 'English',
  langZh: '中文',
} as const

export default en
