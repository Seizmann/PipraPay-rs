import { useEffect, useState, type FormEvent } from 'react'
import Admin, { useMe } from './Admin'
import { api } from '../lib/api'

interface Brand {
  brand_id: string; identify_name: string; name: string | null;
  support_email_address: string | null; support_phone_number: string | null; support_website: string | null;
  whatsapp_number: string | null; telegram: string | null; timezone: string;
  payment_tolerance: string; notify_email_enabled: boolean; notify_telegram_enabled: boolean;
  notify_telegram_chat_id: string | null;
}
interface BrandLite { brand_id: string; identify_name: string; name: string | null }

export default function BrandSettings() {
  const me = useMe()
  const [brands, setBrands] = useState<BrandLite[]>([])
  const [brandId, setBrandId] = useState('')
  const [b, setB] = useState<Brand | null>(null)
  const [msg, setMsg] = useState('')

  const canEdit = me?.role === 'admin' || !!me?.permissions?.resources?.brand_settings?.edit

  useEffect(() => {
    api<{ brands: BrandLite[] }>('/api/brands').then((j) => {
      setBrands(j.brands); if (j.brands.length) setBrandId(j.brands[0].brand_id)
    }).catch((e) => setMsg(e.message))
  }, [])
  useEffect(() => {
    if (!brandId) return
    api<{ brand: Brand }>(`/api/brands/${brandId}`).then((j) => setB(j.brand)).catch((e) => setMsg(e.message))
  }, [brandId])

  async function save(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    const val = (k: string) => (f.get(k) as string) || undefined
    try {
      await api(`/api/brands/${brandId}`, {
        method: 'PUT',
        body: JSON.stringify({
          name: val('name'), identify_name: val('identify_name'),
          support_email_address: val('support_email_address'),
          support_phone_number: val('support_phone_number'),
          support_website: val('support_website'),
          whatsapp_number: val('whatsapp_number'), telegram: val('telegram'),
          timezone: val('timezone'),
          payment_tolerance: val('payment_tolerance') ? parseFloat(val('payment_tolerance')!) : undefined,
          notify_email_enabled: f.get('notify_email_enabled') === 'on',
          notify_telegram_enabled: f.get('notify_telegram_enabled') === 'on',
          notify_telegram_chat_id: val('notify_telegram_chat_id'),
        }),
      })
      setMsg('Saved.')
      const j = await api<{ brand: Brand }>(`/api/brands/${brandId}`)
      setB(j.brand)
    } catch (err: any) { setMsg(err.message) }
  }

  return (
    <Admin>
      <h1 className="text-xl font-semibold">Brand settings</h1>
      {msg && <p className="mt-2 text-sm text-gray-500">{msg}</p>}
      <div className="mt-4 flex items-center gap-2 text-sm">
        <label>Brand:</label>
        <select value={brandId} onChange={(e) => setBrandId(e.target.value)} className="rounded-lg border px-3 py-2 text-sm">
          {brands.map((x) => <option key={x.brand_id} value={x.brand_id}>{x.identify_name}</option>)}
        </select>
      </div>
      {b && (
        <form onSubmit={save} className="mt-4 grid max-w-2xl grid-cols-2 gap-2 text-sm">
          <label className="flex flex-col">Identify name<input name="identify_name" defaultValue={b.identify_name} className="rounded border px-2 py-1" /></label>
          <label className="flex flex-col">Name<input name="name" defaultValue={b.name ?? ''} className="rounded border px-2 py-1" /></label>
          <label className="flex flex-col">Support email<input name="support_email_address" defaultValue={b.support_email_address ?? ''} className="rounded border px-2 py-1" /></label>
          <label className="flex flex-col">Support phone<input name="support_phone_number" defaultValue={b.support_phone_number ?? ''} className="rounded border px-2 py-1" /></label>
          <label className="flex flex-col">Website<input name="support_website" defaultValue={b.support_website ?? ''} className="rounded border px-2 py-1" /></label>
          <label className="flex flex-col">WhatsApp<input name="whatsapp_number" defaultValue={b.whatsapp_number ?? ''} className="rounded border px-2 py-1" /></label>
          <label className="flex flex-col">Telegram handle<input name="telegram" defaultValue={b.telegram ?? ''} className="rounded border px-2 py-1" /></label>
          <label className="flex flex-col">Timezone<input name="timezone" defaultValue={b.timezone} className="rounded border px-2 py-1" /></label>
          <label className="flex flex-col">Payment tolerance (BDT)<input name="payment_tolerance" type="number" step="0.01" defaultValue={b.payment_tolerance} className="rounded border px-2 py-1" /></label>
          <div className="col-span-2 mt-2 space-y-1">
            <label className="inline-flex items-center gap-2"><input type="checkbox" name="notify_email_enabled" defaultChecked={b.notify_email_enabled} /> Email notifications</label><br />
            <label className="inline-flex items-center gap-2"><input type="checkbox" name="notify_telegram_enabled" defaultChecked={b.notify_telegram_enabled} /> Telegram notifications</label>
            <label className="flex flex-col">Telegram chat id<input name="notify_telegram_chat_id" defaultValue={b.notify_telegram_chat_id ?? ''} className="rounded border px-2 py-1" /></label>
          </div>
          {canEdit && <button className="col-span-2 mt-3 w-fit rounded-lg bg-blue-600 px-4 py-2 text-white">Save</button>}
        </form>
      )}
    </Admin>
  )
}
