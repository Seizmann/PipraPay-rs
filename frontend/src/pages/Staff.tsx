import { useEffect, useState, type FormEvent } from 'react'
import Admin, { useMe } from './Admin'
import { api } from '../lib/api'

interface StaffRow { user_id: string; full_name: string; username: string; email: string; role: string }

export default function StaffPage() {
  const me = useMe()
  const [rows, setRows] = useState<StaffRow[]>([])
  const [msg, setMsg] = useState('')
  const [editing, setEditing] = useState<string | null>(null)
  const [perm, setPerm] = useState<Record<string, Record<string, boolean>>>({})
  const [permPages, setPermPages] = useState<Record<string, boolean>>({})

  const canCreate = me?.role === 'admin' || !!me?.permissions?.resources?.staff?.create
  const canDelete = me?.role === 'admin' || !!me?.permissions?.resources?.staff?.delete
  const canPerm = me?.role === 'admin' || !!me?.permissions?.resources?.staff?.edit_permission

  async function load() {
    try {
      const j = await api<{ staff: StaffRow[] }>('/api/staff')
      setRows(j.staff)
    } catch (e: any) { setMsg(e.message) }
  }
  useEffect(() => { load() }, [])

  async function create(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    try {
      await api('/api/staff', {
        method: 'POST',
        body: JSON.stringify({
          full_name: f.get('full_name'), username: f.get('username'),
          email: f.get('email'), password: f.get('password'), brand_ids: [],
        }),
      })
      ;(e.target as HTMLFormElement).reset()
      load()
    } catch (err: any) { setMsg(err.message) }
  }

  async function openPerms(id: string) {
    setEditing(id)
    try {
      const j = await api<{ permissions: { brand_id: string; permission: any }[] }>(`/api/staff/${id}/permissions`)
      const p = j.permissions[0]?.permission ?? { resources: {}, pages: {} }
      setPerm(p.resources ?? {})
      setPermPages(p.pages ?? {})
    } catch (err: any) { setMsg(err.message) }
  }

  async function savePerms() {
    if (!editing) return
    try {
      await api(`/api/staff/${editing}/permissions`, {
        method: 'PUT',
        body: JSON.stringify({ entries: [{ brand_id: me?.brand_id, permission: { resources: perm, pages: permPages } }] }),
      })
      setEditing(null)
    } catch (err: any) { setMsg(err.message) }
  }

  async function remove(id: string) {
    try { await api(`/api/staff/${id}`, { method: 'DELETE' }); load() }
    catch (err: any) { setMsg(err.message) }
  }

  return (
    <Admin>
      <h1 className="text-xl font-semibold">Staff management</h1>
      {msg && <p className="mt-2 text-sm text-red-600">{msg}</p>}
      {canCreate && (
        <form onSubmit={create} className="mt-4 flex flex-wrap gap-2">
          <input name="full_name" placeholder="Full name" required className="rounded-lg border px-3 py-2 text-sm" />
          <input name="username" placeholder="Username" required className="rounded-lg border px-3 py-2 text-sm" />
          <input name="email" type="email" placeholder="Email" required className="rounded-lg border px-3 py-2 text-sm" />
          <input name="password" type="password" placeholder="Password" required className="rounded-lg border px-3 py-2 text-sm" />
          <button className="rounded-lg bg-blue-600 px-4 py-2 text-white">Add staff</button>
        </form>
      )}
      <table className="mt-6 w-full text-sm">
        <thead><tr className="border-b text-left text-gray-500">
          <th className="py-2">Name</th><th>Username</th><th>Email</th><th>Role</th><th></th>
        </tr></thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.user_id} className="border-b">
              <td className="py-2">{r.full_name}</td>
              <td>{r.username}</td>
              <td>{r.email}</td>
              <td>{r.role}</td>
              <td className="space-x-2 text-right">
                {canPerm && <button onClick={() => openPerms(r.user_id)} className="text-blue-600">Permissions</button>}
                {canDelete && r.role !== 'admin' && <button onClick={() => remove(r.user_id)} className="text-red-600">Delete</button>}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {editing && (
        <div className="fixed inset-0 flex items-center justify-center bg-black/40" onClick={() => setEditing(null)}>
          <div className="max-h-[80vh] w-[36rem] overflow-auto rounded-xl bg-white p-6" onClick={(e) => e.stopPropagation()}>
            <h2 className="text-lg font-semibold">Edit permissions</h2>
            <PermEditor perm={perm} setPerm={setPerm} permPages={permPages} setPermPages={setPermPages} />
            <div className="mt-4 flex justify-end gap-2">
              <button onClick={() => setEditing(null)} className="rounded-lg border px-4 py-2 text-sm">Cancel</button>
              <button onClick={savePerms} className="rounded-lg bg-blue-600 px-4 py-2 text-sm text-white">Save</button>
            </div>
          </div>
        </div>
      )}
    </Admin>
  )
}

// ponytail: schema lists inlined; move to /api/permissions/schema if the UI needs it editable.
const RESOURCE_KEYS: Record<string, string[]> = {
  customers: ['create', 'edit', 'delete'],
  transaction: ['edit', 'delete', 'approve', 'cancel', 'refund', 'send_ipn'],
  invoice: ['create', 'edit', 'delete'],
  payment_link: ['create', 'edit', 'delete'],
  gateways: ['create', 'edit', 'delete'],
  brand_settings: ['view', 'edit'],
  theme_settings: ['view', 'edit'],
  faq_settings: ['view', 'create', 'edit', 'delete'],
  sms_data: ['create', 'edit', 'delete'],
  device: ['connect', 'delete', 'balance_verification_for'],
  brands: ['create', 'edit', 'delete'],
  staff: ['create', 'edit', 'delete', 'assign_brand_to', 'edit_permission', 'view_permission_list', 'delete_permission_of'],
  domains: ['whitelist', 'edit', 'delete'],
  system_settings: ['manage_general', 'manage_cron', 'manage_import'],
}
const PAGE_KEYS = ['dashboard', 'reports', 'customers', 'transaction', 'invoice', 'payment_link',
  'gateways', 'brand_settings', 'sms_data', 'device', 'brands', 'staff_management', 'domains', 'system_settings']

function PermEditor({ perm, setPerm, permPages, setPermPages }: {
  perm: Record<string, Record<string, boolean>>
  setPerm: (p: Record<string, Record<string, boolean>>) => void
  permPages: Record<string, boolean>
  setPermPages: (p: Record<string, boolean>) => void
}) {
  return (
    <div className="mt-3">
      <h3 className="text-sm font-semibold">Resources</h3>
      <table className="mt-2 text-xs">
        <tbody>
          {Object.entries(RESOURCE_KEYS).map(([mod, actions]) => (
            <tr key={mod}>
              <td className="pr-3 py-1 font-medium">{mod}</td>
              {actions.map((a) => (
                <td key={a} className="px-2">
                  <label className="inline-flex items-center gap-1">
                    <input type="checkbox" checked={!!perm[mod]?.[a]}
                      onChange={(e) => setPerm({ ...perm, [mod]: { ...perm[mod], [a]: e.target.checked } })} />
                    {a}
                  </label>
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <h3 className="mt-4 text-sm font-semibold">Pages</h3>
      <div className="mt-2 flex flex-wrap gap-3 text-xs">
        {PAGE_KEYS.map((p) => (
          <label key={p} className="inline-flex items-center gap-1">
            <input type="checkbox" checked={!!permPages[p]}
              onChange={(e) => setPermPages({ ...permPages, [p]: e.target.checked })} />
            {p}
          </label>
        ))}
      </div>
    </div>
  )
}
