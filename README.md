# PipraPay-rs

From-scratch Rust rewrite of [PipraPay](https://github.com/piprapay/piprapay) (PHP) — a self-hosted invoice & payment-link platform for Bangladeshi MFS gateways (bKash, Nagad), verified via SMS.

The PHP original keeps running in parallel; this repo is a fresh start.

## Stack

| Layer    | Tech                                                        |
|----------|-------------------------------------------------------------|
| Backend  | Rust (Axum + SQLx + Tokio), Postgres (Supabase)             |
| Frontend | React + TypeScript + Vite + Tailwind CSS                    |
| Serving  | One Axum binary serves the API and the built SPA            |
| Access   | Cloudflare Tunnel (runs on a local machine, near-zero idle resources) |

## SMS verification

An Android companion app ([PipraPay V3 app](https://github.com/piprapay/piprapay-v3-app)) on a payment-receiving phone intercepts bKash/Nagad payment SMS and pushes them to this backend, which parses and matches them against open invoices (amount + receive number + TrxID). No official merchant APIs in v1 — SMS-based verification for personal, merchant, and agent accounts alike.

## Docs

- [REQUIREMENTS.md](REQUIREMENTS.md) — PRD: scope, domain model, acceptance criteria, milestones
- [CLAUDE.md](CLAUDE.md) — conventions and rules for coding agents
- `SECRETS.md` — local credentials (gitignored; see its template for required values)

## Status

v1 in development — milestone 1 (skeleton) pending. See REQUIREMENTS.md §8 for the build order.
