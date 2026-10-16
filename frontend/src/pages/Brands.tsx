import { useEffect, useState, type FormEvent } from 'react'
import Admin, { useMe } from './Admin'
import { api } from '../lib/api'

interface Brand { brand_id: string; identify_name: string; name: string }

export default function Brands() {
  const me = useMe()
  const [rows, setRows] = useState<Brand[]>([])
  const [msg, setMsg] = useState('')
  const canCreate = me?.role === 'admin' || !!me?.permissions?.resources?.brands?.create
  const canDelete = me?.role === 'admin' || !!me?.permissions?.resources?.brands?.delete

  async function load() {
    try { setRows((await api<{ brands: Brand[] }>('/api/brands')).brands) } catch (e: any) { setMsg(e.message) }
  }
  useEffect(() => { load() }, [])

  async function create(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    try {
      await api('/api/brands', { method: 'POST', body: JSON.stringify({ name: f.get('name') }) })
      load()
    } catch (err: any) { setMsg(err.message) }
  }

  async function remove(id: string) {
    try { await api(`/api/brands/${id}`, { method: 'DELETE' }); load() } catch (err: any) { setMsg(err.message) }
  }

  return (
    <Admin>
      <h1 className="text-xl font-semibold">Brands</h1>
      {msg && <p className="mt-2 text-sm text-red-600">{msg}</p>}
      {canCreate && (
        <form onSubmit={create} className="mt-4 flex gap-2">
          <input name="name" placeholder="Brand name" required className="rounded-lg border px-3 py-2 text-sm" />
          <button className="rounded-lg bg-blue-600 px-4 py-2 text-white">Add brand</button>
        </form>
      )}
      <table className="mt-6 w-full text-sm">
        <thead><tr className="border-b text-left text-gray-500"><th className="py-2">Identify</th><th>Name</th><th>brand_id</th><th></th></tr></thead>
        <tbody>
          {rows.map((b) => (
            <tr key={b.brand_id} className="border-b">
              <td className="py-2">{b.identify_name}</td>
              <td>{b.name}</td>
              <td className="font-mono text-xs">{b.brand_id}</td>
              <td className="text-right">{canDelete && <button onClick={() => remove(b.brand_id)} className="text-red-600">Delete</button>}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </Admin>
  )
}
