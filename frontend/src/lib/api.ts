// Minimal fetch wrapper: JSON in/out, throws on !ok with server message.
export async function api<T = any>(path: string, opts: RequestInit = {}): Promise<T> {
  const res = await fetch(path, {
    headers: { 'Content-Type': 'application/json' },
    credentials: 'same-origin',
    ...opts,
  })
  const body = await res.json().catch(() => ({}))
  if (!res.ok) throw new Error((body as any).message ?? `HTTP ${res.status}`)
  return body as T
}
