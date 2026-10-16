import { useEffect, useState, type FormEvent } from 'react'
import Admin, { useMe } from './Admin'
import { api } from '../lib/api'

export default function SystemSettings() {
  const me = useMe()
  const [s, setS] = useState<Record<string, string>>({})
  const [msg, setMsg] = useState('')

  const canManage = me?.role === 'admin' || !!me?.permissions?.resources?.system_settings?.manage_general

  useEffect(() => {
    api<{ settings: Record<string, string> }>('/api/settings')
      .then((j) => setS(j.settings)).catch((e) => setMsg(e.message))
  }, [])

  async function save(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    const out: Record<string, string> = {}
    f.forEach((v, k) => { out[k] = v as string })
    try {
      await api('/api/settings', { method: 'PUT', body: JSON.stringify({ settings: out }) })
      setMsg('Saved.')
    } catch (err: any) { setMsg(err.message) }
  }

  return (
    <Admin>
      <h1 className="text-xl font-semibold">System settings</h1>
      {msg && <p className="mt-2 text-sm text-gray-500">{msg}</p>}
      {canManage && (
        <form onSubmit={save} className="mt-4 max-w-lg space-y-3 text-sm">
          <label className="flex flex-col">Default timezone
            <input name="geneal-application-settings-default_timezone" defaultValue={s['geneal-application-settings-default_timezone'] ?? ''} className="rounded border px-2 py-1" />
          </label>
          <label className="flex flex-col">Webhook retry attempts limit
            <input name="webhook_retry_attempts_limit" type="number" defaultValue={s['webhook_retry_attempts_limit'] ?? '5'} className="rounded border px-2 py-1" />
          </label>
          <label className="flex flex-col">Cron key (background task guard)
            <input name="cron-key" defaultValue={s['cron-key'] ?? ''} className="rounded border px-2 py-1" />
          </label>
          <button className="rounded-lg bg-blue-600 px-4 py-2 text-white">Save</button>
        </form>
      )}
    </Admin>
  )
}
