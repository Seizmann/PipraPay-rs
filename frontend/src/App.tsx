import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { useEffect, useState } from 'react'
import Setup from './pages/Setup'
import Login from './pages/Login'
import Dashboard from './pages/Dashboard'
import StaffPage from './pages/Staff'

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/setup" element={<Setup />} />
        <Route path="/admin/login" element={<Login />} />
        <Route path="/admin/dashboard" element={<Dashboard />} />
        <Route path="/admin/staff" element={<StaffPage />} />
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
