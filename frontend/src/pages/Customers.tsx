import { useEffect, useState, type FormEvent } from 'react'
import Admin, { useMe } from './Admin'
import { api } from '../lib/api'

interface Customer { ref: string; name: string; email: string; mobile: string; status: string }
interface Brand { brand_id: string; identify_name: string; name: string | null }

export default function Customers() {
  const me = useMe()
  const [brands, setBrands] = useState<Brand[]>([])
  const [brand, setBrand] = useState('')
  const [rows, setRows] = useState<Customer[]>([])
  const [msg, setMsg] = useState('')

  const canCreate = me?.role === 'admin' || !!me?.permissions?.resources?.customers?.create

  useEffect(() => {
    api<{ brands: Brand[] }>('/api/brands').then((j) => {
      setBrands(j.brands); if (j.brands.length) setBrand(j.brands[0].brand_id)
    }).catch((e) => setMsg(e.message))
  }, [])
  useEffect(() => {
    if (!brand) return
    api<{ customers: Customer[] }>(`/api/brands/${brand}/customers`)
      .then((j) => setRows(j.customers)).catch((e) => setMsg(e.message))
  }, [brand])

  async function create(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    try {
      await api(`/api/brands/${brand}/customers`, {
        method: 'POST',
        body: JSON.stringify({ name: f.get('name'), email: f.get('email'), mobile: f.get('mobile') }),
      })
      ;(e.target as HTMLFormElement).reset()
      const j = await api<{ customers: Customer[] }>(`/api/brands/${brand}/customers`)
      setRows(j.customers)
    } catch (err: any) { setMsg(err.message) }
  }

  async function suspend(c: Customer) {
    try {
      await api(`/api/customers/${c.ref}`, {
        method: 'PUT', body: JSON.stringify({ status: c.status === 'active' ? 'suspend' : 'active' }),
      })
      const j = await api<{ customers: Customer[] }>(`/api/brands/${brand}/customers`)
      setRows(j.customers)
    } catch (err: any) { setMsg(err.message) }
  }

  return (
    <Admin>
      <h1 className="text-xl font-semibold">Customers</h1>
      {msg && <p className="mt-2 text-sm text-red-600">{msg}</p>}
      <div className="mt-4 flex items-center gap-2 text-sm">
        <label>Brand:</label>
        <select value={brand} onChange={(e) => setBrand(e.target.value)} className="rounded-lg border px-3 py-2 text-sm">
          {brands.map((b) => <option key={b.brand_id} value={b.brand_id}>{b.identify_name}</option>)}
        </select>
      </div>
      {canCreate && (
        <form onSubmit={create} className="mt-3 flex gap-2">
          <input name="name" placeholder="Name" required className="rounded-lg border px-3 py-2 text-sm" />
          <input name="email" placeholder="Email" className="rounded-lg border px-3 py-2 text-sm" />
          <input name="mobile" placeholder="Mobile" className="rounded-lg border px-3 py-2 text-sm" />
          <button className="rounded-lg bg-blue-600 px-4 py-2 text-white">Add</button>
        </form>
      )}
      <table className="mt-6 w-full text-sm">
        <thead><tr className="border-b text-left text-gray-500"><th className="py-2">Name</th><th>Email</th><th>Mobile</th><th>Status</th><th></th></tr></thead>
        <tbody>
          {rows.map((c) => (
            <tr key={c.ref} className="border-b">
              <td className="py-2">{c.name}</td><td>{c.email}</td><td>{c.mobile}</td><td>{c.status}</td>
              <td className="text-right">
                <button onClick={() => suspend(c)} className="text-blue-600">
                  {c.status === 'active' ? 'Suspend' : 'Activate'}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </Admin>
  )
}
