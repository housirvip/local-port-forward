import { useState, useEffect, useRef, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { logsApi, rulesApi, type RequestLog, type Rule } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { ChevronDown, ChevronRight, RefreshCw, Trash2 } from 'lucide-react'
import { toast } from 'sonner'

function statusVariant(status?: number, protocol?: string): 'green' | 'yellow' | 'red' | 'gray' {
  if (protocol === 'tcp') return 'gray'
  if (!status) return 'gray'
  if (status < 300) return 'green'
  if (status < 400) return 'yellow'
  return 'red'
}

function formatBytes(n: number) {
  if (n < 1024) return `${n}B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)}KB`
  return `${(n / 1024 / 1024).toFixed(1)}MB`
}

function formatDuration(ms: number) {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

export default function LogsPage() {
  const { t } = useTranslation()
  const [logs, setLogs] = useState<RequestLog[]>([])
  const [total, setTotal] = useState(0)
  const [page, setPage] = useState(1)
  const pageSize = 50
  const [filterRuleId, setFilterRuleId] = useState<number | undefined>()
  const [liveMode, setLiveMode] = useState(true)
  const [liveConnected, setLiveConnected] = useState(false)
  const [expandedId, setExpandedId] = useState<number | null>(null)
  const [rules, setRules] = useState<Rule[]>([])
  const [loading, setLoading] = useState(false)
  const esRef = useRef<EventSource | null>(null)

  // Load rules for filter dropdown
  useEffect(() => { rulesApi.list().then(setRules).catch(() => {}) }, [])

  const fetchLogs = useCallback(() => {
    setLoading(true)
    logsApi.list({ rule_id: filterRuleId, page, page_size: pageSize })
      .then(data => { setLogs(data.items); setTotal(data.total) })
      .catch(e => toast.error(e.message))
      .finally(() => setLoading(false))
  }, [filterRuleId, page])

  useEffect(() => { fetchLogs() }, [fetchLogs])

  // SSE live mode
  useEffect(() => {
    if (!liveMode) {
      esRef.current?.close()
      esRef.current = null
      return
    }
    const es = logsApi.stream()
    esRef.current = es
    es.onmessage = (ev) => {
      try {
        const entry: RequestLog = JSON.parse(ev.data)
        if (filterRuleId != null && entry.rule_id !== filterRuleId) return
        setLogs(prev => {
          const next = [entry, ...prev]
          return next.slice(0, 200)
        })
        setTotal(t => t + 1)
      } catch { /* ignore parse errors */ }
    }
    es.onopen = () => setLiveConnected(true)
    es.onerror = () => setLiveConnected(false)
    return () => { es.close(); esRef.current = null; setLiveConnected(false) }
  }, [liveMode, filterRuleId])

  async function handleClear() {
    if (!confirm(filterRuleId ? t('confirmClearLogsRule') : t('confirmClearLogs'))) return
    try {
      const res = await logsApi.clear(filterRuleId)
      toast.success(t('toastDeleted', { count: res.deleted }))
      setLogs([])
      setTotal(0)
      setPage(1)
    } catch (e: unknown) {
      toast.error((e as Error).message)
    }
  }

  const ruleName = (id: number) => rules.find(r => r.id === id)?.name || t('fallbackRule', { id })

  const totalPages = Math.ceil(total / pageSize)

  return (
    <div className="space-y-4">
      {/* Filter bar */}
      <div className="flex flex-wrap items-center gap-3">
        <div className="w-48">
          <Select
            value={filterRuleId != null ? String(filterRuleId) : 'all'}
            onValueChange={v => { setFilterRuleId(v === 'all' ? undefined : Number(v)); setPage(1) }}
          >
            <SelectTrigger><SelectValue placeholder={t('filterAllRules')} /></SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('filterAllRules')}</SelectItem>
              {rules.map(r => (
                <SelectItem key={r.id} value={String(r.id)}>
                  {r.name || t('filterPort', { port: r.local_port })}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="flex items-center gap-2">
          <Switch id="live" checked={liveMode} onCheckedChange={(v) => { setLiveMode(v); if (v) setPage(1) }} />
          <Label htmlFor="live">{t('labelLive')}</Label>
          {liveMode && (
            <span className={`inline-flex items-center gap-1 text-xs ${
              liveConnected ? 'text-green-600' : 'text-yellow-500'
            }`}>
              <span className={`w-1.5 h-1.5 rounded-full ${
                liveConnected ? 'bg-green-500' : 'bg-yellow-400 animate-pulse'
              }`} />
              {liveConnected ? t('statusLive') : t('statusReconnecting')}
            </span>
          )}
        </div>

        <Button variant="outline" size="sm" onClick={fetchLogs} disabled={loading}>
          <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          {t('btnRefresh')}
        </Button>

        <Button variant="outline" size="sm" onClick={handleClear}>
          <Trash2 className="h-4 w-4 text-red-500" /> {t('btnClearLogs')}
        </Button>

        <span className="ml-auto text-sm text-zinc-500">{t('totalCount', { count: total })}</span>
      </div>

      {logs.length === 0 ? (
        <div className="rounded-lg border border-dashed border-zinc-300 p-12 text-center text-zinc-500">
          {loading ? t('logsLoading') : t('logsEmpty')}
        </div>
      ) : (
        <div className="rounded-lg border border-zinc-200 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-zinc-50 border-b border-zinc-200">
              <tr>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600 w-6"></th>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600">{t('colTime')}</th>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600">{t('colRule')}</th>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600">{t('colProto')}</th>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600">{t('colMethod')}</th>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600">{t('colPathPreview')}</th>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600">{t('colStatus')}</th>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600">{t('colSize')}</th>
                <th className="px-3 py-2.5 text-left font-medium text-zinc-600">{t('colDuration')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-100">
              {logs.map(log => (
                <>
                  <tr
                    key={log.id}
                    className="hover:bg-zinc-50 cursor-pointer transition-colors"
                    onClick={() => setExpandedId(expandedId === log.id ? null : log.id)}
                  >
                    <td className="px-3 py-2.5 text-zinc-400">
                      {expandedId === log.id
                        ? <ChevronDown className="h-4 w-4" />
                        : <ChevronRight className="h-4 w-4" />}
                    </td>
                    <td className="px-3 py-2.5 text-zinc-500 text-xs whitespace-nowrap">
                      {new Date(log.created_at).toLocaleTimeString()}
                    </td>
                    <td className="px-3 py-2.5 text-xs text-zinc-600">{ruleName(log.rule_id)}</td>
                    <td className="px-3 py-2.5">
                      <Badge variant="gray">{log.protocol}</Badge>
                    </td>
                    <td className="px-3 py-2.5 font-mono text-xs font-semibold text-zinc-700">
                      {log.http_method || '—'}
                    </td>
                    <td className="px-3 py-2.5 font-mono text-xs text-zinc-600 max-w-[300px] truncate">
                      {log.http_path || (log.tcp_preview ? log.tcp_preview.slice(0, 60) : '—')}
                    </td>
                    <td className="px-3 py-2.5">
                      {log.http_status ? (
                        <Badge variant={statusVariant(log.http_status, log.protocol)}>
                          {log.http_status}
                        </Badge>
                      ) : <span className="text-zinc-400 text-xs">—</span>}
                    </td>
                    <td className="px-3 py-2.5 text-xs text-zinc-500">{formatBytes(log.bytes_transferred)}</td>
                    <td className="px-3 py-2.5 text-xs text-zinc-500">{formatDuration(log.duration_ms)}</td>
                  </tr>
                  {expandedId === log.id && (
                    <tr key={`${log.id}-detail`} className="bg-zinc-50">
                      <td colSpan={9} className="px-6 py-4">
                        <LogDetail log={log} />
                      </td>
                    </tr>
                  )}
                </>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Pagination */}
      {!liveMode && total > pageSize && (
        <div className="flex items-center justify-end gap-2 text-sm">
          <Button variant="outline" size="sm" disabled={page <= 1} onClick={() => setPage(p => p - 1)}>{t('btnPrev')}</Button>
          <span className="text-zinc-500">{t('pageOf', { page, total: totalPages })}</span>
          <Button variant="outline" size="sm" disabled={page >= totalPages} onClick={() => setPage(p => p + 1)}>{t('btnNext')}</Button>
        </div>
      )}
    </div>
  )
}

function LogDetail({ log }: { log: RequestLog }) {
  const { t } = useTranslation()

  function prettyJSON(s?: string | null) {
    if (!s) return null
    try { return JSON.stringify(JSON.parse(s), null, 2) } catch { return s }
  }

  if (log.protocol === 'tcp') {
    return (
      <div>
        <p className="text-xs font-semibold text-zinc-500 mb-1">{t('tcpPreview')}</p>
        <pre className="text-xs bg-white rounded border border-zinc-200 p-3 max-h-[300px] overflow-y-auto whitespace-pre-wrap break-all">
          {log.tcp_preview || t('noPreview')}
        </pre>
      </div>
    )
  }

  return (
    <div className="grid grid-cols-2 gap-4">
      <Section title={t('reqHeaders')} content={prettyJSON(log.http_req_headers)} />
      <Section title={t('respHeaders')} content={prettyJSON(log.http_resp_headers)} />
      <Section title={t('reqBody')} content={log.http_req_body} />
      <Section title={t('respBody')} content={log.http_resp_body} />
    </div>
  )
}

function Section({ title, content }: { title: string; content?: string | null }) {
  const { t } = useTranslation()
  return (
    <div>
      <p className="text-xs font-semibold text-zinc-500 mb-1">{title}</p>
      <pre className="text-xs bg-white rounded border border-zinc-200 p-3 max-h-[300px] overflow-y-auto whitespace-pre-wrap break-all">
        {content || <span className="text-zinc-400">{t('sectionEmpty')}</span>}
      </pre>
    </div>
  )
}
