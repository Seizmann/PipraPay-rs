import { useState, type FormEvent } from 'react'

export default function Setup() {
  const [msg, setMsg] = useState('')
  const [done, setDone] = useState(false)

  async function submit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const f = new FormData(e.currentTarget)
    const res = await fetch('/api/setup', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        setup_token: f.get('token'),
        username: f.get('username'),
        email: f.get('email'),
        password: f.get('password'),
        full_name: f.get('full_name'),
      }),
    })
    const j = await res.json().catch(() => ({}))
    if (res.ok) {
      setDone(true)
    } else {
      setMsg(j.message ?? 'Setup failed')
    }
  }

  if (done) {
    return (
      <div className="mx-auto mt-32 max-w-md rounded-xl border p-6 text-center">
        <h1 className="text-lg font-semibold">Setup complete</h1>
        <p className="mt-2 text-sm text-gray-500">You can now sign in.</p>
        <a href="/admin/login" className="mt-4 inline-block rounded-lg bg-blue-600 px-4 py-2 text-white">Go to login</a>
      </div>
    )
  }

  return (
    <div className="mx-auto mt-16 max-w-md rounded-xl border p-6">
      <h1 className="text-xl font-semibold">Initial setup</h1>
      <p className="mt-1 text-sm text-gray-500">Create the first admin account.</p>
      <form onSubmit={submit} className="mt-4 space-y-3">
        <input name="token" placeholder="Setup token (from SECRETS.md)" required
          className="w-full rounded-lg border px-3 py-2 text-sm" />
        <input name="full_name" placeholder="Full name" required
          className="w-full rounded-lg border px-3 py-2 text-sm" />
        <input name="username" placeholder="Username" required
          className="w-full rounded-lg border px-3 py-2 text-sm" />
        <input name="email" type="email" placeholder="Email" required
          className="w-full rounded-lg border px-3 py-2 text-sm" />
        <input name="password" type="password" placeholder="Password (min 8 chars)" required
          className="w-full rounded-lg border px-3 py-2 text-sm" />
        <button className="w-full rounded-lg bg-blue-600 py-2 text-white">Create admin</button>
        {msg && <p className="text-sm text-red-600">{msg}</p>}
      </form>
    </div>
  )
}
