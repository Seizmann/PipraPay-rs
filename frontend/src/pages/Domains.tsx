import { useEffect, useState, type FormEvent } from 'react'
import Admin, { useMe } from './Admin'
import { api } from '../lib/api'

interface DomainRow { id: number; domain: string; status: string }

export default function Domains() {
  const me = useMe()
  const [rows, setRows] = useState<DomainRow[]>([])
  const [msg, setMsg] = useState('')

  const canWhitelist = me?.role === 'admin' || !!me?.permissions?.resources?.domains?.whitelist

  async function load() {
    try { setRows((await api<{ domains: DomainRow[] }>('/api/domains')).domains) } catch (e: any) { setMsg(e.message) }
  }
  useEffect(() => { load() }, [])

  async function create(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    try { await api('/api/domains', { method: 'POST', body: JSON.stringify({ domain: f.get('domain') }) }); load() }
    catch (err: any) { setMsg(err.message) }
  }
  async function toggleStatus(r: DomainRow) {
    try {
      await api(`/api/domains/${r.id}`, { method: 'PUT', body: JSON.stringify({ status: r.status === 'active' ? 'inactive' : 'active' }) })
      load()
    } catch (err: any) { setMsg(err.message) }
  }

  return (
    <Admin>
      <h1 className="text-xl font-semibold">Domains</h1>
      <p className="mt-1 text-sm text-gray-500">Allowlisted domains for API/webhook URLs.</p>
      {msg && <p className="mt-2 text-sm text-red-600">{msg}</p>}
      {canWhitelist && (
        <form onSubmit={create} className="mt-4 flex gap-2">
          <input name="domain" placeholder="example.com" required className="rounded-lg border px-3 py-2 text-sm" />
          <button className="rounded-lg bg-blue-600 px-4 py-2 text-white">Add domain</button>
        </form>
      )}
      <table className="mt-6 w-full text-sm">
        <thead><tr className="border-b text-left text-gray-500"><th className="py-2">Domain</th><th>Status</th><th></th></tr></thead>
        <tbody>
          {rows.map((d) => (
            <tr key={d.id} className="border-b">
              <td className="py-2">{d.domain}</td><td>{d.status}</td>
              <td className="text-right"><button onClick={() => toggleStatus(d)} className="text-blue-600">Toggle</button></td>
            </tr>
          ))}
        </tbody>
      </table>
    </Admin>
  )
}
