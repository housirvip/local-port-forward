import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { rulesApi, type Rule } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Badge } from '@/components/ui/badge'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger,
} from '@/components/ui/dialog'
import { Plus, Pencil, Trash2, ArrowRight, AlertTriangle } from 'lucide-react'
import { toast } from 'sonner'

const emptyForm = (): Omit<Rule, 'id' | 'created_at' | 'updated_at' | 'bind_error'> => ({
  name: '',
  local_port: 3000,
  remote_host: '',
  remote_port: 3000,
  protocol: 'auto',
  enabled: true,
  log_enabled: true,
  log_body: false,
})

function ConfirmDialog({ open, title, onConfirm, onCancel }: {
  open: boolean; title: string;
  onConfirm: () => void; onCancel: () => void;
}) {
  const { t } = useTranslation()
  return (
    <Dialog open={open} onOpenChange={(v) => { if (!v) onCancel() }}>
      <DialogContent className="max-w-sm">
        <DialogHeader><DialogTitle>{title}</DialogTitle></DialogHeader>
        <div className="flex justify-end gap-2 mt-4">
          <Button variant="outline" onClick={onCancel}>{t('btnCancel')}</Button>
          <Button variant="destructive" onClick={onConfirm}>{t('btnConfirmDelete')}</Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}

export default function RulesPage() {
  const { t } = useTranslation()
  const [rules, setRules] = useState<Rule[]>([])
  const [dialogOpen, setDialogOpen] = useState(false)
  const [editing, setEditing] = useState<Rule | null>(null)
  const [form, setForm] = useState(emptyForm())
  const [saving, setSaving] = useState(false)
  const [confirmState, setConfirmState] = useState<{ rule: Rule } | null>(null)

  const load = () => rulesApi.list().then(setRules).catch(e => toast.error(e.message))

  useEffect(() => { load() }, [])

  function openAdd() {
    setEditing(null)
    setForm(emptyForm())
    setDialogOpen(true)
  }

  function openEdit(rule: Rule) {
    setEditing(rule)
    setForm({
      name: rule.name,
      local_port: rule.local_port,
      remote_host: rule.remote_host,
      remote_port: rule.remote_port,
      protocol: rule.protocol,
      enabled: rule.enabled,
      log_enabled: rule.log_enabled,
      log_body: rule.log_body,
    })
    setDialogOpen(true)
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (form.local_port < 1 || form.local_port > 65535) { toast.error(t('errLocalPort')); return }
    if (!form.remote_host.trim()) { toast.error(t('errRemoteHost')); return }
    if (form.remote_port < 1 || form.remote_port > 65535) { toast.error(t('errRemotePort')); return }
    setSaving(true)
    try {
      if (editing) {
        const updated = await rulesApi.update(editing.id, form)
        setRules(rs => rs.map(r => r.id === updated.id ? updated : r))
        toast.success(t('toastRuleUpdated'))
      } else {
        const created = await rulesApi.create(form)
        setRules(rs => [...rs, created])
        toast.success(t('toastRuleCreated'))
      }
      setDialogOpen(false)
    } catch (e: unknown) {
      toast.error((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  function handleDelete(rule: Rule) {
    setConfirmState({ rule })
  }

  async function doDelete(rule: Rule) {
    try {
      await rulesApi.delete(rule.id)
      setRules(rs => rs.filter(r => r.id !== rule.id))
      toast.success(t('toastRuleDeleted'))
    } catch (e: unknown) {
      toast.error((e as Error).message)
    } finally {
      setConfirmState(null)
    }
  }

  async function handleToggle(rule: Rule) {
    // Optimistic update
    setRules(rs => rs.map(r => r.id === rule.id ? { ...r, enabled: !r.enabled } : r))
    try {
      const updated = await rulesApi.toggle(rule.id)
      setRules(rs => rs.map(r => r.id === updated.id ? updated : r))
      if (updated.enabled && updated.bind_error) {
        toast.error(t('toastPortConflict', { port: updated.local_port, error: updated.bind_error }))
      }
    } catch (e: unknown) {
      // Revert
      setRules(rs => rs.map(r => r.id === rule.id ? rule : r))
      toast.error((e as Error).message)
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">{t('rulesHeading')}</h2>
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger asChild>
            <Button onClick={openAdd} size="sm">
              <Plus className="h-4 w-4" /> {t('btnAddRule')}
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{t(editing ? 'dialogEditTitle' : 'dialogAddTitle')}</DialogTitle>
            </DialogHeader>
            <form onSubmit={handleSubmit} className="space-y-4 mt-2">
              <div className="space-y-1">
                <Label>{t('fieldName')}</Label>
                <Input placeholder={t('placeholderName')} value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>{t('fieldLocalPort')}</Label>
                  <Input type="number" min={1} max={65535} value={form.local_port}
                    onChange={e => setForm(f => ({ ...f, local_port: Number(e.target.value) }))} />
                </div>
                <div className="space-y-1">
                  <Label>{t('fieldProtocol')}</Label>
                  <Select value={form.protocol} onValueChange={v => setForm(f => ({ ...f, protocol: v as Rule['protocol'] }))}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="auto">{t('optAuto')}</SelectItem>
                      <SelectItem value="http">{t('optHttp')}</SelectItem>
                      <SelectItem value="tcp">{t('optTcp')}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>{t('fieldRemoteHost')}</Label>
                  <Input placeholder={t('placeholderRemoteHost')} value={form.remote_host}
                    onChange={e => setForm(f => ({ ...f, remote_host: e.target.value }))} />
                </div>
                <div className="space-y-1">
                  <Label>{t('fieldRemotePort')}</Label>
                  <Input type="number" min={1} max={65535} value={form.remote_port}
                    onChange={e => setForm(f => ({ ...f, remote_port: Number(e.target.value) }))} />
                </div>
              </div>
              <div className="flex gap-6">
                <div className="flex items-center gap-2">
                  <Switch checked={form.enabled} onCheckedChange={v => setForm(f => ({ ...f, enabled: v }))} id="sw-enabled" />
                  <Label htmlFor="sw-enabled">{t('fieldEnabled')}</Label>
                </div>
                <div className="flex items-center gap-2">
                  <Switch checked={form.log_enabled} onCheckedChange={v => setForm(f => ({ ...f, log_enabled: v }))} id="sw-log" />
                  <Label htmlFor="sw-log">{t('fieldLogTraffic')}</Label>
                </div>
                <div className="flex items-center gap-2">
                  <Switch checked={form.log_body} onCheckedChange={v => setForm(f => ({ ...f, log_body: v }))} id="sw-body" />
                  <Label htmlFor="sw-body">{t('fieldLogBody')}</Label>
                </div>
              </div>
              <div className="flex justify-end gap-2 pt-2">
                <Button type="button" variant="outline" onClick={() => setDialogOpen(false)}>{t('btnCancel')}</Button>
                <Button type="submit" disabled={saving}>
                  {saving ? t('btnSaving') : editing ? t('btnUpdate') : t('btnCreate')}
                </Button>
              </div>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      {rules.length === 0 ? (
        <div className="rounded-lg border border-dashed border-zinc-300 p-12 text-center text-zinc-500">
          {t('emptyRules')}
        </div>
      ) : (
        <div className="rounded-lg border border-zinc-200 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-zinc-50 border-b border-zinc-200">
              <tr>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-600">{t('colName')}</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-600">{t('colForwarding')}</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-600">{t('colProtocol')}</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-600">{t('colEnabled')}</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-600">{t('colLog')}</th>
                <th className="px-4 py-2.5 text-right font-medium text-zinc-600">{t('colActions')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-100">
              {rules.map(rule => (
                <tr key={rule.id} className="hover:bg-zinc-50 transition-colors">
                  <td className="px-4 py-3 font-medium">{rule.name || <span className="text-zinc-400">—</span>}</td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-1.5 font-mono text-xs">
                      <span className="rounded bg-blue-50 px-1.5 py-0.5 text-blue-700">:{rule.local_port}</span>
                      <ArrowRight className="h-3 w-3 text-zinc-400" />
                      <span className="rounded bg-zinc-100 px-1.5 py-0.5 text-zinc-700">{rule.remote_host}:{rule.remote_port}</span>
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <Badge variant={rule.protocol === 'http' ? 'green' : rule.protocol === 'tcp' ? 'gray' : 'default'}>
                      {rule.protocol}
                    </Badge>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <Switch checked={rule.enabled} onCheckedChange={() => handleToggle(rule)} />
                      {rule.enabled && rule.bind_error && (
                        <span title={t('badgePortConflictTooltip', { error: rule.bind_error })}>
                          <Badge variant="red" className="inline-flex items-center gap-1">
                            <AlertTriangle className="h-3 w-3" />
                            {t('badgePortConflict')}
                          </Badge>
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-4 py-3 text-zinc-500 text-xs">
                    {rule.log_enabled ? (rule.log_body ? t('logStatusBody') : t('logStatusHeaders')) : t('logStatusOff')}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex justify-end gap-1">
                      <Button variant="ghost" size="icon" onClick={() => openEdit(rule)}>
                        <Pencil className="h-4 w-4" />
                      </Button>
                      <Button variant="ghost" size="icon" onClick={() => handleDelete(rule)}>
                        <Trash2 className="h-4 w-4 text-red-500" />
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {confirmState && (
        <ConfirmDialog
          open
          title={t('confirmDeleteTitle', { name: confirmState.rule.name || confirmState.rule.local_port })}
          onConfirm={() => doDelete(confirmState.rule)}
          onCancel={() => setConfirmState(null)}
        />
      )}
    </div>
  )
}
