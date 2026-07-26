const BASE = '/api'

export interface Rule {
  id: number
  name: string
  local_port: number
  remote_host: string
  remote_port: number
  protocol: 'auto' | 'http' | 'tcp'
  enabled: boolean
  log_enabled: boolean
  log_body: boolean
  created_at: string
  updated_at: string
  bind_error: string | null
}

export interface RequestLog {
  id: number
  rule_id: number
  created_at: string
  protocol: string
  src_addr: string
  http_method?: string
  http_path?: string
  http_status?: number
  http_req_headers?: string
  http_req_body?: string
  http_resp_headers?: string
  http_resp_body?: string
  tcp_preview?: string
  bytes_transferred: number
  duration_ms: number
}

export interface LogsPage {
  total: number
  page: number
  page_size: number
  items: RequestLog[]
}

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(body.error || res.statusText)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

export const rulesApi = {
  list: (): Promise<Rule[]> =>
    fetch(`${BASE}/rules`).then(r => handleResponse<Rule[]>(r)),

  create: (data: Omit<Rule, 'id' | 'created_at' | 'updated_at' | 'bind_error'>): Promise<Rule> =>
    fetch(`${BASE}/rules`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    }).then(r => handleResponse<Rule>(r)),

  update: (id: number, data: Partial<Omit<Rule, 'id' | 'created_at' | 'updated_at' | 'bind_error'>>): Promise<Rule> =>
    fetch(`${BASE}/rules/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    }).then(r => handleResponse<Rule>(r)),

  delete: (id: number): Promise<void> =>
    fetch(`${BASE}/rules/${id}`, { method: 'DELETE' }).then(r => handleResponse<void>(r)),

  toggle: (id: number): Promise<Rule> =>
    fetch(`${BASE}/rules/${id}/toggle`, { method: 'POST' }).then(r => handleResponse<Rule>(r)),
}

export const logsApi = {
  list: (params?: { rule_id?: number; page?: number; page_size?: number }): Promise<LogsPage> => {
    const p = new URLSearchParams()
    if (params?.rule_id != null) p.set('rule_id', String(params.rule_id))
    if (params?.page != null) p.set('page', String(params.page))
    if (params?.page_size != null) p.set('page_size', String(params.page_size))
    return fetch(`${BASE}/logs?${p}`).then(r => handleResponse<LogsPage>(r))
  },

  clear: (rule_id?: number): Promise<{ deleted: number }> =>
    fetch(`${BASE}/logs${rule_id != null ? `?rule_id=${rule_id}` : ''}`, { method: 'DELETE' }).then(r =>
      handleResponse<{ deleted: number }>(r)
    ),

  stream: () => new EventSource(`${BASE}/logs/stream`),
}

export interface AppSettings {
  log_max_rows:        number
  log_ttl_days:        number
  default_protocol:    'auto' | 'http' | 'tcp'
  default_log_enabled: boolean
  default_log_body:    boolean
  listen_addr:         string
  db_path:             string
}

export type SettingsInput = Omit<AppSettings, 'listen_addr' | 'db_path'>

export const settingsApi = {
  get: (): Promise<AppSettings> =>
    fetch(`${BASE}/settings`).then(r => handleResponse<AppSettings>(r)),

  update: (data: Partial<SettingsInput>): Promise<AppSettings> =>
    fetch(`${BASE}/settings`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    }).then(r => handleResponse<AppSettings>(r)),
}
