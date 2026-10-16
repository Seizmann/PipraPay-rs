# PipraPay-rs — Requirements (PRD)

Version: 1.0 (draft)
Date: 2026-10-16
Status: Discussion complete — implementation reference

PipraPay-rs is a from-scratch rewrite of PipraPay (PHP) in Rust, developed as a
new parallel project. The existing PHP PipraPay keeps running; this does not
replace it at launch and no data migration is needed (fresh start).

---

## 1. Goals

1. A personal payment-gateway platform (a & b): self-hosted invoice/payment-link
   system for real use, eventually replacing the PHP deployment.
2. Extremely low idle resource usage: runs on the developer's local machine,
   near-zero RAM/CPU when idle.
3. Compatible with the existing PipraPay V3 Android companion app (SMS gateway)
   **without modifying the app** — the backend must implement the app's exact
   API contract.
4. Modern stack: Rust backend + React/TypeScript SPA frontend.

## 2. Non-goals (V1)

- Official merchant APIs (bKash tokenized, Nagad merchant API) — V1 is SMS-based
  verification only, including for merchant accounts.
- Data migration from PHP PipraPay.
- Third-party gateway plugins/addon marketplace.
- Multi-currency (BDT only in V1).
- Separate mobile app (reuse existing V3 app).

## 3. Tech stack

| Layer      | Choice                                                          |
|------------|-----------------------------------------------------------------|
| Backend    | Rust + **Axum** + **SQLx** (Postgres) + Tokio                   |
| Frontend   | React + TypeScript + Vite + Tailwind CSS (SPA)                  |
| Database   | **Supabase Cloud (Postgres)** — credentials in `SECRETS.md`     |
| Serving    | Single Axum binary serves both `/api/*` and built `frontend/dist` (SPA fallback to `index.html`). No CORS needed (same origin). |
| Exposure   | Cloudflare Tunnel (named tunnel) for public URL (webhooks, customer payment pages, V3 app SMS push) |
| Secrets    | All credentials in `SECRETS.md` at project root, gitignored. Coding agents run all DB migrations from it. |

### Repository layout

```
PipraPay-rs/
├── backend/
│   ├── src/
│   └── migrations/        # SQLx migrations
├── frontend/              # React+TS+Vite+Tailwind
│   └── dist/              # build output, served by Axum
├── REQUIREMENTS.md
├── SECRETS.md             # gitignored — DB url, SMTP, Telegram, tunnel
└── .gitignore
```

## 4. Domain model (feature parity with PHP PipraPay)

### 4.1 Brands
- Multiple brands/businesses per account, each with own settings: name, logo,
  currency (BDT fixed in V1), brand-specific gateway assignments, notification
  settings.
- Brand settings pages mirror PHP PipraPay: general, payment-link settings,
  currency (BDT only), FAQ, themes.

### 4.2 Staff & permissions
- Multi-staff system with role + permission matrix, same granularity as PHP
  PipraPay: page-level permissions (`view`, `create`, `edit`, `delete`,
  `connect` …) per module (invoices, transactions, devices, gateways, staff,
  settings, brands...).
- Staff belong to roles; roles hold permission sets. Admin role is superuser.
- 2FA (TOTP) for login, same as PHP version.

### 4.3 Customers
- Customer records with name/email/phone; invoice & payment-link history per
  customer.

### 4.4 Invoices & payment links
- Create invoice → send public link → customer opens payment page → pays via
  configured gateway → SMS verification matches payment → invoice marked paid.
- Payment links (fixed-amount and pay-what-you-want style), same behavior as
  PHP PipraPay.
- Public customer-facing payment page is **in V1 scope**: gateway selection,
  amount, instructions (send money number, TrxID entry), QR code, theme.
- Theme system: V1 ships with one default theme (theme engine structured so
  more can be added later).

### 4.5 Gateways (V1: SMS-based only)
- Supported: **bKash** (personal / merchant / agent), **Nagad** (personal /
  merchant / agent).
- Per-brand gateway configuration: receive number, gateway type, instructions,
  enabled/disabled.
- Verification: incoming payment SMS from the V3 app is parsed, matched against
  expected amount + receive number + TrxID → transaction/invoice marked paid.
- All parsing is SMS-pattern based; no official APIs.

### 4.6 SMS companion app integration (hard requirement)
The existing PipraPay V3 Android app must work unchanged. Its API contract
(observed in app source, `PipraPayApi.java`): all calls are
`POST {base_url}` with form-encoded bodies, keyed by `action-companion`:

| `action-companion`        | Payload                                                       | Response                              |
|---------------------------|---------------------------------------------------------------|---------------------------------------|
| `login`                   | `onetimepassword`, `name`, `model`, `android_level`, `app_version` | `{"status": true, "token": "..."}` or `{"message": "..."}` |
| `account-information`     | token header/param (see implementation during build)          | account info JSON                     |
| `sms-transmit-sender`     | whitelisted sender numbers / device info                      | sender list ack                       |
| `sms-transmit-bulk`       | `sms_list` (JSON array of intercepted SMS)                    | matching result ack                   |

- Device pairing via one-time password (OTP) shown in the admin panel → app
  stores a token.
- Devices page in admin: list paired devices, connect info, bulk delete —
  same as PHP PipraPay.
- Historical SMS import (24h/3d/7d batch) from the app must be handled by
  `sms-transmit-bulk` with dedupe by TrxID.
- Exact request/response shapes must be verified against the app source code
  (`/home/sijan/SijansP/pirapay-full/PipraPay-V3-App/`) during implementation
  of each endpoint. The PHP handler for these actions is the reference for
  semantics.

### 4.7 Transactions & SMS data
- Transactions list per brand (status, gateway, amount, TrxID, customer).
- Raw received SMS log (`sms-data` equivalent) with matching status (matched /
  unmatched / duplicate) for debugging.

### 4.8 Notifications
- **Email (SMTP)** and **Telegram** — both, configurable per brand (mirrors
  PHP brand-setting which has Telegram + email).
- Events: invoice paid, invoice created (to customer), payment received via
  SMS, device pairing event.

### 4.9 Dashboard & reports
- Dashboard: totals (paid invoices, revenue, per-gateway breakdown), recent
  transactions, device status chips.
- Reports page with date-range filters.

### 4.10 System settings & misc
- General settings, cron-job equivalents (invoice reminders, cleanup — as
  Tokio background tasks), import (customers/invoices CSV), domains page
  (tunnel/public URL config), activities/audit log.

## 5. Auth & API surface

- Admin/staff SPA session auth (session cookie or JWT — decided at
  implementation; cookie preferred for same-origin simplicity).
- Public API for payment pages (no auth, per-link token).
- Companion app API (contract in 4.6, token-authed after OTP pairing).
- REST-ish JSON API under `/api/*` for the SPA.

## 6. Non-functional requirements

- Idle footprint target: < 50 MB RSS, ~0% CPU when idle (the reason this
  rewrite exists).
- Single binary + `frontend/dist` + `SECRETS.md` = full deployment.
- Money paths (SMS matching, TrxID dedupe, amount comparison) need exact
  decimal handling — use integer-minor-units or Rust `rust_decimal`, never
  floats. Every matching rule gets one runnable test.
- All payment/SMS parsing rules table-driven (regex/pattern config), so new
  MFS formats are added without restructuring.
- Audit log for staff actions (who did what — activities page parity).

## 7. Acceptance criteria (V1 done means)

1. V3 Android app pairs via OTP, pushes live + bulk SMS, payments verify.
2. Invoice + payment-link created, customer pays via bKash/Nagad SMS flow,
   invoice auto-marks paid, email + Telegram notification fires.
3. Multi-brand with per-brand gateway/notification settings works.
4. Staff accounts with permission matrix + 2FA work.
5. SPA served by the same Axum binary, usable behind Cloudflare Tunnel.
6. `docker stats`-grade idleness: near-zero CPU, < 50 MB RAM for the backend
   process.

## 8. Build order (suggested milestones)

1. Skeleton: Axum + SQLx + Supabase connection, migrations, SPA serve, tunnel.
2. Auth: login, 2FA, roles/permissions, staff management.
3. Brands + gateways (config only) + customers.
4. Companion app contract: login/OTP pairing, account-information,
   sms-transmit-* with SMS parsing (bKash first, then Nagad).
5. Invoices + payment links + public payment page + verification matching.
6. Notifications (Telegram + SMTP), dashboard, reports, activities.
7. Settings, import, cron tasks, polish.
