import { useState, useEffect } from 'react'
import { settingsApi, type AppSettings } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { toast } from 'sonner'

export default function SettingsPage() {
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
      toast.success('设置已保存')
    } catch (e: unknown) {
      toast.error((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  if (!settings) return <div className="text-sm text-zinc-400 p-4">加载中...</div>

  return (
    <div className="max-w-xl space-y-8">
      <form onSubmit={handleSave} className="space-y-8">

        {/* 日志清理 */}
        <section className="space-y-4">
          <h2 className="text-sm font-semibold text-zinc-700 border-b pb-2">日志清理策略</h2>

          <div className="space-y-1.5">
            <Label htmlFor="log_max_rows">最大保留条数</Label>
            <Input
              id="log_max_rows" type="number" min={0}
              value={form.log_max_rows ?? 0}
              onChange={e => setForm(f => ({ ...f, log_max_rows: Number(e.target.value) }))}
            />
            <p className="text-xs text-zinc-400">0 = 不限制；超出条数时每小时自动删除最早的日志</p>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="log_ttl_days">保留天数（TTL）</Label>
            <Input
              id="log_ttl_days" type="number" min={0}
              value={form.log_ttl_days ?? 0}
              onChange={e => setForm(f => ({ ...f, log_ttl_days: Number(e.target.value) }))}
            />
            <p className="text-xs text-zinc-400">0 = 不限制；超过天数的日志每小时自动删除</p>
          </div>
        </section>

        {/* 新建规则默认 */}
        <section className="space-y-4">
          <h2 className="text-sm font-semibold text-zinc-700 border-b pb-2">新建规则默认属性</h2>

          <div className="space-y-1.5">
            <Label>默认协议</Label>
            <Select
              value={form.default_protocol ?? 'auto'}
              onValueChange={v => setForm(f => ({ ...f, default_protocol: v as 'auto' | 'http' | 'tcp' }))}
            >
              <SelectTrigger className="w-40"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">自动检测</SelectItem>
                <SelectItem value="http">HTTP</SelectItem>
                <SelectItem value="tcp">TCP</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-3">
            <Switch
              id="default_log_enabled"
              checked={form.default_log_enabled ?? true}
              onCheckedChange={v => setForm(f => ({ ...f, default_log_enabled: v }))}
            />
            <Label htmlFor="default_log_enabled">默认开启请求日志</Label>
          </div>

          <div className="flex items-center gap-3">
            <Switch
              id="default_log_body"
              checked={form.default_log_body ?? false}
              onCheckedChange={v => setForm(f => ({ ...f, default_log_body: v }))}
            />
            <Label htmlFor="default_log_body">默认记录请求内容（Body）</Label>
          </div>
        </section>

        <Button type="submit" disabled={saving}>{saving ? '保存中...' : '保存设置'}</Button>
      </form>

      {/* 只读运行时信息 */}
      <section className="space-y-3">
        <h2 className="text-sm font-semibold text-zinc-700 border-b pb-2">运行时信息（只读）</h2>
        <div className="space-y-2 text-sm">
          <div className="flex items-center gap-2">
            <span className="text-zinc-500 w-24">监听地址</span>
            <code className="bg-zinc-100 px-2 py-0.5 rounded text-zinc-700">{settings.listen_addr}</code>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-zinc-500 w-24">数据库路径</span>
            <code className="bg-zinc-100 px-2 py-0.5 rounded text-zinc-700">{settings.db_path}</code>
          </div>
          <p className="text-xs text-zinc-400 mt-1">以上配置通过环境变量设置，需重启服务才能生效。</p>
        </div>
      </section>
    </div>
  )
}
