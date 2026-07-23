# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

**LibrePay** — open-source mobile money aggregator API in Rust: one HTTP/JSON API in front of many
operators (Orange Money, MTN MoMo, Moov, Wave). Built to run **today** on a mock
operator with zero setup, and go live on real operators the day a contract is signed —
by adding an adapter + env vars, with no rewrite.

## Commands

```bash
cargo run            # start API on 0.0.0.0:8080 (mock provider), configurable via env
cargo build          # debug build
cargo build --release
cargo test           # all tests
cargo test signature # run one module's tests (e.g. the HMAC tests)
cargo test tampered_body_fails   # run a single test by name
```

Runtime config is env-only (see `.env.example`): `BIND_ADDR`, `DEFAULT_PROVIDER`,
`WEBHOOK_SECRET`. Log level via `RUST_LOG` (e.g. `RUST_LOG=debug cargo run`).

## Architecture — the two abstractions that matter

The whole "build now, go live later without a rewrite" property rests on two traits.
Understand these before changing anything:

- **`PaymentProvider`** (`src/providers/mod.rs`) — the operator contract. Every
  operator is one file implementing it. `MockProvider` (`src/providers/mock.rs`) is
  the only one today; `MtnProvider`/`OrangeProvider`/`WaveProvider` will sit beside it.
  Handlers never know which concrete provider they call — they look it up by name in
  `AppState.providers`.
- **`Ledger`** (`src/ledger/mod.rs`) — transaction storage. `InMemoryLedger` today
  (data lost on restart, fine for dev/demo/tests); a `PostgresLedger` (Supabase) will
  implement the same trait later. Routes must not depend on the concrete ledger.

`src/config.rs` is the switch: the environment decides which operator is live and with
which secrets, so the public repo ships **zero** secrets (`.env` is git-ignored).

## Request flow (mobile money is asynchronous)

1. `POST /v1/payments` → validate → idempotency check → `provider.initiate()` →
   store a `Pending` transaction → return it. (`src/routes/payments.rs`)
2. Customer approves on their phone (out of band).
3. Operator confirms via `POST /v1/webhook/{provider}` — HMAC-verified over the raw
   body before the payload is trusted. (`src/routes/webhook.rs`)
4. Fallback: `GET /v1/payments/{id}` polls `provider.check_status()` while `Pending`
   and persists any change. With the mock provider this flips to `success`.

## Conventions that are load-bearing (do not break)

- **Money is integer `i64`** in the currency base unit. XOF (franc CFA) has no minor
  unit. Never introduce floats for amounts.
- **Idempotency**: `POST /v1/payments` honors the `Idempotency-Key` header; the same
  key must always return the same transaction and never charge twice.
- **Webhook signatures**: verify with the constant-time `verify_hmac_sha256`
  (`src/signature.rs`). An unverified callback must be rejected (`401`) — never trust a
  webhook body before checking `X-Signature`.
- **Errors** go through `ApiError` (`src/error.rs`), which renders a consistent
  `{ "error": { "code", "message" } }` JSON shape. Add new variants there rather than
  returning ad-hoc responses.
- Axum 0.8 path params use `{id}` syntax (not `:id`).

## Adding a new operator (the intended extension point)

1. Add `src/providers/<operator>.rs` implementing `PaymentProvider`.
2. Register it in `main.rs` (`providers.insert(...)`).
3. Add its credentials to `.env` (and document them in `.env.example`).
4. Point `DEFAULT_PROVIDER` at it. No route/ledger changes needed.
