# CLAUDE.md — PipraPay-rs

Instructions for coding agents working in this repo. Read alongside
[REQUIREMENTS.md](REQUIREMENTS.md) (the PRD — source of truth for scope).

## What this is

From-scratch Rust rewrite of PipraPay (PHP). The PHP app keeps running in
parallel; this repo is a fresh start (no data migration). Runs on the owner's
local machine, exposed via Cloudflare Tunnel. Goal: near-zero idle RAM/CPU.

## Stack (fixed — do not swap)

- Backend: Rust + Axum + SQLx + Tokio, Postgres (Supabase cloud)
- Frontend: React + TypeScript + Vite + Tailwind CSS, served by the same Axum
  binary (`frontend/dist` + SPA fallback to index.html). No CORS — same origin.
- One binary + `frontend/dist` + secrets = full deployment.

## Secrets

- All credentials live in `SECRETS.md` (gitignored). Read them from there —
  never hardcode into source, migrations, or examples.
- Agents run all DB migrations against `DATABASE_URL` from `SECRETS.md`.
- Never log secrets or echo them into errors.

## Layout

```
backend/
  src/            # Axum app
  migrations/     # SQLx migrations (ordered, never edit an applied one)
frontend/
  src/            # React SPA
  dist/           # build output (gitignored)
PipraPay-V3-App/  # NOT in this repo — see path below
```

Reference codebases (read-only, outside this repo):
- PHP original: `/home/sijan/SijansP/pirapay-full/PipraPay/`
- Android SMS gateway app: `/home/sijan/SijansP/pirapay-full/PipraPay-V3-App/`

## Hard rules

1. **Money paths**: amounts use `rust_decimal` (or integer minor units), never
   floats. Every SMS-matching / TrxID-dedupe / amount-compare rule ships with
   at least one runnable test that fails if the rule breaks.
2. **V3 app compatibility**: the existing Android app must work unchanged. Its
   API contract: `POST {base_url}` form-encoded, keyed by `action-companion`
   field (`login`, `account-information`, `sms-transmit-sender`,
   `sms-transmit-bulk`). Verify exact request/response shapes against
   `PipraPayApi.java` in the app repo and the PHP handlers in
   `PipraPay/pp-content/pp-admin/pp-root/devices/` before implementing.
3. **SMS parsing is table-driven** (pattern/regex config), not hardcoded
   per-gateway branches — new MFS formats must not require restructuring.
4. **Parity over invention**: features mirror PHP PipraPay (brands, staff +
   permission matrix + 2FA, invoices, payment links, public payment page,
   Telegram + SMTP notifications, dashboard/reports, activities log). Don't
   redesign domain concepts while porting.
5. **V1 scope only** (see REQUIREMENTS.md §2 non-goals): bKash + Nagad
   SMS-based only, BDT only, no merchant APIs, no addon marketplace. Suggest
   bigger ideas, but don't build them unasked.

## Commands

```bash
# backend
cd backend && cargo run                 # dev server
cargo test                              # includes money-path tests
sqlx migrate run                        # against DATABASE_URL from SECRETS.md

# frontend
cd frontend && npm run dev              # Vite dev server (proxies /api to backend)
cd frontend && npm run build            # outputs frontend/dist, Axum serves it
```

## Conventions

- Keep diffs small and boring; reuse before adding.
- Mark deliberate simplifications with `// ponytail:` comments naming the
  ceiling and upgrade path.
- No new dependencies for what stdlib/axum/sqlx already cover — check
  `Cargo.toml`/`package.json` first.
- Commit messages end with: `Co-Authored-By: Claude Code <noreply@anthropic.com>`

## Build order (milestones)

1. Skeleton: Axum + SQLx + Supabase, migrations, SPA serve, tunnel config
2. Auth: login, 2FA, roles/permissions, staff
3. Brands + gateway config + customers
4. V3 app contract + SMS parsing (bKash, then Nagad)
5. Invoices + payment links + public payment page + matching
6. Notifications (Telegram + SMTP), dashboard, reports, activities
7. Settings, import, cron tasks, polish
