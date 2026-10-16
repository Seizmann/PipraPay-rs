import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { useEffect, useState } from 'react'
import Setup from './pages/Setup'
import Login from './pages/Login'
import Dashboard from './pages/Dashboard'
import StaffPage from './pages/Staff'
import Brands from './pages/Brands'
import Gateways from './pages/Gateways'
import Customers from './pages/Customers'
import Domains from './pages/Domains'
import BrandSettings from './pages/BrandSettings'
import SystemSettings from './pages/SystemSettings'

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/setup" element={<Setup />} />
        <Route path="/admin/login" element={<Login />} />
        <Route path="/admin/dashboard" element={<Dashboard />} />
        <Route path="/admin/staff" element={<StaffPage />} />
        <Route path="/admin/brands" element={<Brands />} />
        <Route path="/admin/gateways" element={<Gateways />} />
        <Route path="/admin/customers" element={<Customers />} />
        <Route path="/admin/domains" element={<Domains />} />
        <Route path="/admin/brand-settings" element={<BrandSettings />} />
        <Route path="/admin/settings" element={<SystemSettings />} />
        <Route path="*" element={<Root />} />
      </Routes>
    </BrowserRouter>
  )
}

function Root() {
  const [status, setStatus] = useState<'checking' | 'setup' | 'login'>('checking')
  useEffect(() => {
    fetch('/api/setup/status')
      .then((r) => r.json())
      .then((j) => setStatus(j.setup_needed ? 'setup' : 'login'))
      .catch(() => setStatus('login'))
  }, [])
  if (status === 'checking') return null
  if (status === 'setup') return <Navigate to="/setup" replace />
  return <Navigate to="/admin/login" replace />
}

export default App
