import { useState, type FormEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '../lib/api'

export default function Login() {
  const [msg, setMsg] = useState('')
  const [twoFa, setTwoFa] = useState(false)
  const nav = useNavigate()

  async function submit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    setMsg('')
    try {
      const j = await api('/api/auth/login', {
        method: 'POST',
        body: JSON.stringify({ username: f.get('username'), password: f.get('password') }),
      })
      if (j.target === '2fa') {
        setTwoFa(true)
      } else {
        nav('/admin/dashboard')
      }
    } catch (err: any) {
      setMsg(err.message)
    }
  }

  async function submit2fa(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    try {
      await api('/api/auth/2fa', { method: 'POST', body: JSON.stringify({ code: f.get('code') }) })
      nav('/admin/dashboard')
    } catch (err: any) {
      setMsg(err.message)
    }
  }

  return (
    <div className="mx-auto mt-32 max-w-md rounded-xl border p-6">
      <h1 className="text-xl font-semibold">PipraPay</h1>
      {!twoFa ? (
        <form onSubmit={submit} className="mt-4 space-y-3">
          <input name="username" placeholder="Email or username" required
            className="w-full rounded-lg border px-3 py-2 text-sm" />
          <input name="password" type="password" placeholder="Password" required
            className="w-full rounded-lg border px-3 py-2 text-sm" />
          <button className="w-full rounded-lg bg-blue-600 py-2 text-white">Sign in</button>
        </form>
      ) : (
        <form onSubmit={submit2fa} className="mt-4 space-y-3">
          <p className="text-sm text-gray-500">Enter the 6-digit code from your authenticator.</p>
          <input name="code" placeholder="123456" required maxLength={6}
            className="w-full rounded-lg border px-3 py-2 text-sm tracking-widest" />
          <button className="w-full rounded-lg bg-blue-600 py-2 text-white">Verify</button>
        </form>
      )}
      {msg && <p className="mt-3 text-sm text-red-600">{msg}</p>}
    </div>
  )
}
