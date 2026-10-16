import { useEffect, useState, type ReactNode } from 'react'
import { NavLink, Outlet, useNavigate } from 'react-router-dom'
import { api } from '../lib/api'

// Page nav items with the PHP permission page keys.
const NAV: { key: string; label: string; path: string }[] = [
  { key: 'dashboard', label: 'Dashboard', path: '/admin/dashboard' },
  { key: 'brands', label: 'Brands', path: '/admin/brands' },
  { key: 'customers', label: 'Customers', path: '/admin/customers' },
  { key: 'invoice', label: 'Invoices', path: '/admin/invoices' },
  { key: 'payment_link', label: 'Payment Links', path: '/admin/payment-links' },
  { key: 'transaction', label: 'Transactions', path: '/admin/transactions' },
  { key: 'gateways', label: 'Gateways', path: '/admin/gateways' },
  { key: 'device', label: 'Devices', path: '/admin/devices' },
  { key: 'sms_data', label: 'SMS Data', path: '/admin/sms-data' },
  { key: 'brand_settings', label: 'Brand Settings', path: '/admin/brand-settings' },
  { key: 'reports', label: 'Reports', path: '/admin/reports' },
  { key: 'staff_management', label: 'Staff', path: '/admin/staff' },
  { key: 'domains', label: 'Domains', path: '/admin/domains' },
  { key: 'system_settings', label: 'Settings', path: '/admin/settings' },
]

export interface Me {
  user_id: string
  role: string
  full_name: string
  username: string
  email: string
  brand_id: string
  permissions: { pages: Record<string, boolean>; resources: Record<string, Record<string, boolean>> } | null
}

export function useMe(): Me | null {
  const [me, setMe] = useState<Me | null>(null)
  useEffect(() => {
    api<Me>('/api/auth/me').then((j: any) => setMe(j)).catch(() => {})
  }, [])
  return me
}

export default function Admin({ children }: { children?: ReactNode }) {
  const me = useMe()
  const nav = useNavigate()

  if (!me) return <div className="p-8 text-gray-500">Loading…</div>

  const visible = me.role === 'admin' ? NAV : NAV.filter((n) => me.permissions?.pages?.[n.key])

  async function logout() {
    await api('/api/auth/logout', { method: 'POST' }).catch(() => {})
    nav('/admin/login')
  }

  return (
    <div className="flex min-h-screen">
      <aside className="w-56 border-r bg-gray-50 p-4">
        <div className="mb-4">
          <div className="font-semibold">PipraPay</div>
          <div className="text-xs text-gray-500">{me.full_name || me.username} · {me.role}</div>
        </div>
        <nav className="space-y-1">
          {visible.map((n) => (
            <NavLink key={n.key} to={n.path}
              className={({ isActive }) =>
                `block rounded-lg px-3 py-2 text-sm ${isActive ? 'bg-blue-600 text-white' : 'hover:bg-gray-200'}`}>
              {n.label}
            </NavLink>
          ))}
        </nav>
        <button onClick={logout} className="mt-6 w-full rounded-lg border px-3 py-2 text-sm hover:bg-gray-200">
          Log out
        </button>
      </aside>
      <main className="flex-1 p-6">{children ?? <Outlet />}</main>
    </div>
  )
}
