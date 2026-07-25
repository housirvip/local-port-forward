import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { settingsApi, type AppSettings } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { toast } from 'sonner'

export default function SettingsPage() {
  const { t } = useTranslation()
  const [settings, setSettings] = useState<AppSettings | null>(null)
  const [form, setForm] = useState<Partial<AppSettings>>({})
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    settingsApi.get().then(s => { setSettings(s); setForm(s) }).catch(e => toast.error(e.message))
  }, [])

  async function handleSave(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    try {
      const updated = await settingsApi.update({
        log_max_rows:        Number(form.log_max_rows ?? 0),
        log_ttl_days:        Number(form.log_ttl_days ?? 0),
        default_protocol:    form.default_protocol ?? 'auto',
        default_log_enabled: form.default_log_enabled ?? true,
        default_log_body:    form.default_log_body ?? false,
      })
      setSettings(updated)
      setForm(updated)
      toast.success(t('toastSettingsSaved'))
    } catch (e: unknown) {
      toast.error((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  if (!settings) return <div className="text-sm text-zinc-400 p-4">{t('settingsLoading')}</div>

  return (
    <div className="max-w-xl space-y-8">
      <form onSubmit={handleSave} className="space-y-8">

        {/* Log cleanup policy */}
        <section className="space-y-4">
          <h2 className="text-sm font-semibold text-zinc-700 border-b pb-2">{t('settingsLogPolicy')}</h2>

          <div className="space-y-1.5">
            <Label htmlFor="log_max_rows">{t('fieldMaxRows')}</Label>
            <Input
              id="log_max_rows" type="number" min={0}
              value={form.log_max_rows ?? 0}
              onChange={e => setForm(f => ({ ...f, log_max_rows: Number(e.target.value) }))}
            />
            <p className="text-xs text-zinc-400">{t('hintMaxRows')}</p>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="log_ttl_days">{t('fieldTtlDays')}</Label>
            <Input
              id="log_ttl_days" type="number" min={0}
              value={form.log_ttl_days ?? 0}
              onChange={e => setForm(f => ({ ...f, log_ttl_days: Number(e.target.value) }))}
            />
            <p className="text-xs text-zinc-400">{t('hintTtlDays')}</p>
          </div>
        </section>

        {/* New rule defaults */}
        <section className="space-y-4">
          <h2 className="text-sm font-semibold text-zinc-700 border-b pb-2">{t('settingsRuleDefaults')}</h2>

          <div className="space-y-1.5">
            <Label>{t('fieldDefaultProtocol')}</Label>
            <Select
              value={form.default_protocol ?? 'auto'}
              onValueChange={v => setForm(f => ({ ...f, default_protocol: v as 'auto' | 'http' | 'tcp' }))}
            >
              <SelectTrigger className="w-40"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">{t('optAuto')}</SelectItem>
                <SelectItem value="http">{t('optHttp')}</SelectItem>
                <SelectItem value="tcp">{t('optTcp')}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-3">
            <Switch
              id="default_log_enabled"
              checked={form.default_log_enabled ?? true}
              onCheckedChange={v => setForm(f => ({ ...f, default_log_enabled: v }))}
            />
            <Label htmlFor="default_log_enabled">{t('fieldDefaultLogEnabled')}</Label>
          </div>

          <div className="flex items-center gap-3">
            <Switch
              id="default_log_body"
              checked={form.default_log_body ?? false}
              onCheckedChange={v => setForm(f => ({ ...f, default_log_body: v }))}
            />
            <Label htmlFor="default_log_body">{t('fieldDefaultLogBody')}</Label>
          </div>
        </section>

        <Button type="submit" disabled={saving}>
          {saving ? t('btnSavingSettings') : t('btnSaveSettings')}
        </Button>
      </form>

      {/* Runtime info (read-only) */}
      <section className="space-y-3">
        <h2 className="text-sm font-semibold text-zinc-700 border-b pb-2">{t('settingsRuntime')}</h2>
        <div className="space-y-2 text-sm">
          <div className="flex items-center gap-2">
            <span className="text-zinc-500 w-24">{t('labelListenAddr')}</span>
            <code className="bg-zinc-100 px-2 py-0.5 rounded text-zinc-700">{settings.listen_addr}</code>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-zinc-500 w-24">{t('labelDbPath')}</span>
            <code className="bg-zinc-100 px-2 py-0.5 rounded text-zinc-700">{settings.db_path}</code>
          </div>
          <p className="text-xs text-zinc-400 mt-1">{t('hintRuntime')}</p>
        </div>
      </section>
    </div>
  )
}
