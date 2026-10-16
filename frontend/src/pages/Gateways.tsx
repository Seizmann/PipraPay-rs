import { useEffect, useState } from 'react'
import Admin, { useMe } from './Admin'
import { api } from '../lib/api'

interface Gateway {
  gateway_id: string; brand_id: string; slug: string; name: string | null; display: string | null;
  receive_number: string; min_allow: string; max_allow: string;
  fixed_charge: string; percentage_charge: string; fixed_discount: string; percentage_discount: string;
  status: string; instructions: string[] | null; qr_image: string | null;
  primary_color: string | null; text_color: string | null; btn_color: string | null; btn_text_color: string | null;
}

interface Brand { brand_id: string; name: string | null; identify_name: string }

export default function Gateways() {
  const me = useMe()
  const [brands, setBrands] = useState<Brand[]>([])
  const [brand, setBrand] = useState('')
  const [rows, setRows] = useState<Gateway[]>([])
  const [editing, setEditing] = useState<Gateway | null>(null)
  const [msg, setMsg] = useState('')

  const canEdit = me?.role === 'admin' || !!me?.permissions?.resources?.gateways?.edit

  useEffect(() => {
    api<{ brands: Brand[] }>('/api/brands').then((j) => {
      setBrands(j.brands.map((b) => ({ ...b, name: b.name ?? b.identify_name })))
      if (j.brands.length) setBrand(j.brands[0].brand_id)
    }).catch((e) => setMsg(e.message))
  }, [])

  useEffect(() => {
    if (!brand) return
    api<{ gateways: Gateway[] }>(`/api/brands/${brand}/gateways`)
      .then((j) => setRows(j.gateways)).catch((e) => setMsg(e.message))
  }, [brand])

  async function save(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    if (!editing) return
    const f = new FormData(e.currentTarget)
    const num = (k: string) => { const v = f.get(k); return v && v !== '' ? parseFloat(v as string) : undefined }
    const body = {
      name: (f.get('name') as string) || undefined,
      display: (f.get('display') as string) || undefined,
      receive_number: (f.get('receive_number') as string) || undefined,
      min_allow: num('min_allow'), max_allow: num('max_allow'),
      fixed_charge: num('fixed_charge'), percentage_charge: num('percentage_charge'),
      fixed_discount: num('fixed_discount'), percentage_discount: num('percentage_discount'),
      instructions: ((f.get('instructions') as string) || '').split('\n').filter(Boolean),
      status: f.get('status') as string,
    }
    try {
      await api(`/api/gateways/${editing.gateway_id}`, { method: 'PUT', body: JSON.stringify(body) })
      setEditing(null)
      api<{ gateways: Gateway[] }>(`/api/brands/${brand}/gateways`).then((j) => setRows(j.gateways))
    } catch (err: any) { setMsg(err.message) }
  }

  return (
    <Admin>
      <h1 className="text-xl font-semibold">Gateways</h1>
      {msg && <p className="mt-2 text-sm text-red-600">{msg}</p>}
      <div className="mt-4 flex items-center gap-2 text-sm">
        <label>Brand:</label>
        <select value={brand} onChange={(e) => setBrand(e.target.value)} className="rounded-lg border px-3 py-2 text-sm">
          {brands.map((b) => <option key={b.brand_id} value={b.brand_id}>{b.identify_name}</option>)}
        </select>
      </div>
      <table className="mt-6 w-full text-sm">
        <thead><tr className="border-b text-left text-gray-500">
          <th className="py-2">Name</th><th>Slug</th><th>Receive #</th><th>Status</th><th></th>
        </tr></thead>
        <tbody>
          {rows.map((g) => (
            <tr key={g.gateway_id} className="border-b">
              <td className="py-2">{g.name}</td>
              <td className="font-mono text-xs">{g.slug}</td>
              <td>{g.receive_number}</td>
              <td>{g.status}</td>
              <td className="text-right">{canEdit && <button onClick={() => setEditing(g)} className="text-blue-600">Edit</button>}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {editing && (
        <div className="fixed inset-0 flex items-center justify-center bg-black/40" onClick={() => setEditing(null)}>
          <form onSubmit={save} className="max-h-[85vh] w-[32rem] overflow-auto rounded-xl bg-white p-6" onClick={(e) => e.stopPropagation()}>
            <h2 className="text-lg font-semibold">Gateway: {editing.name}</h2>
            <div className="mt-3 grid grid-cols-2 gap-2 text-sm">
              <label className="flex flex-col">Display name<input name="display" defaultValue={editing.display ?? ''} className="rounded border px-2 py-1" /></label>
              <label className="flex flex-col">Receive number<input name="receive_number" defaultValue={editing.receive_number} className="rounded border px-2 py-1" /></label>
              <label className="flex flex-col">Min<input name="min_allow" type="number" step="0.01" defaultValue={editing.min_allow} className="rounded border px-2 py-1" /></label>
              <label className="flex flex-col">Max<input name="max_allow" type="number" step="0.01" defaultValue={editing.max_allow} className="rounded border px-2 py-1" /></label>
              <label className="flex flex-col">Fixed charge<input name="fixed_charge" type="number" step="0.01" defaultValue={editing.fixed_charge} className="rounded border px-2 py-1" /></label>
              <label className="flex flex-col">Charge %<input name="percentage_charge" type="number" step="0.01" defaultValue={editing.percentage_charge} className="rounded border px-2 py-1" /></label>
              <label className="flex flex-col">Fixed discount<input name="fixed_discount" type="number" step="0.01" defaultValue={editing.fixed_discount} className="rounded border px-2 py-1" /></label>
              <label className="flex flex-col">Discount %<input name="percentage_discount" type="number" step="0.01" defaultValue={editing.percentage_discount} className="rounded border px-2 py-1" /></label>
              <label className="col-span-2 flex flex-col">Status
                <select name="status" defaultValue={editing.status} className="rounded border px-2 py-1">
                  <option value="active">active</option><option value="inactive">inactive</option>
                </select>
              </label>
              <label className="col-span-2 flex flex-col">Instructions (one per line)
                <textarea name="instructions" rows={4} defaultValue={(editing.instructions ?? []).join('\n')} className="rounded border px-2 py-1" />
              </label>
            </div>
            <div className="mt-4 flex justify-end gap-2">
              <button type="button" onClick={() => setEditing(null)} className="rounded-lg border px-4 py-2 text-sm">Cancel</button>
              <button className="rounded-lg bg-blue-600 px-4 py-2 text-sm text-white">Save</button>
            </div>
          </form>
        </div>
      )}
    </Admin>
  )
}
