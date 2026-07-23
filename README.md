# LibrePay

Open-source **mobile money aggregator API** in Rust. One HTTP/JSON API in front
of many operators (Orange Money, MTN MoMo, Moov, Wave…). Any client speaks to it
the same way — web (React, Vue, Laravel…) and mobile (Flutter, Kotlin, Swift,
React Native).

**Ready-to-use idea:** build and test the full flow *today* on a mock operator,
with no contract and no server credentials. The day you sign with a real
operator, you add an adapter + fill env vars — **no rewrite**.

## Why a language you can integrate everywhere

The server is Rust, but clients don't care: the interface is **REST + JSON over
HTTP**. Every web and mobile stack can call it. Rust is chosen for the *server's*
safety (money = zero tolerance for crashes/nulls), not for the clients.

## What works without any contract

| Works today (no contract) | Needs a signed operator contract |
|---|---|
| All the code (routes, adapters, ledger, webhooks) | Moving **real** money |
| Mock operator end-to-end | Prod credentials (`401/403` without them) |
| Operator **sandboxes** (e.g. MTN MoMo dev portal) | Holding customer funds (payment licence) |

## Run

```bash
cp .env.example .env          # optional; sane defaults exist
cargo run                     # starts on 0.0.0.0:8080 with the mock operator
```

## Endpoints

| Method | Path | Purpose |
|---|---|---|
| GET  | `/health` | liveness |
| POST | `/v1/payments` | start a collection (charge a customer) |
| GET  | `/v1/payments` | list transactions (`?status=&provider=&limit=`) |
| GET  | `/v1/payments/{id}` | poll status (falls back to operator) |
| GET  | `/v1/stats` | dashboard KPIs (counts, success rate, volume, per-operator) |
| POST | `/v1/webhook/{provider}` | operator callback (HMAC-verified) |

CORS is open in dev so the dashboard (different port) can call the API. Tighten
to the dashboard origin in production.

## Dashboard

A **Flutter Web** dashboard lives in `dashboard/` — KPIs, per-operator breakdown,
and a live transactions table with status filtering. It consumes this API over
REST; set the API base URL from the top bar (default `http://localhost:8080`).

```bash
cd dashboard
flutter pub get
flutter run -d chrome      # dev
flutter build web --release  # production build -> build/web
```

### Create a payment

```bash
curl -s localhost:8080/v1/payments \
  -H 'Content-Type: application/json' \
  -H 'Idempotency-Key: 11111111-1111-1111-1111-111111111111' \
  -d '{
    "amount": 5000,
    "currency": "XOF",
    "provider": "mock",
    "phone": "+22670000000",
    "reference": "CMD-1234",
    "callback_url": "https://merchant.example/webhook"
  }'
```

Returns a `pending` transaction with an `id`. Re-sending the same
`Idempotency-Key` returns the same transaction — **never a double charge**.

### Poll status

```bash
curl -s localhost:8080/v1/payments/<id>
```

With the mock operator this flips to `success`.

## Architecture (the reusable core)

```
Client (web/mobile) ──HTTP/JSON──► API ──► PaymentProvider (trait)
                                              ├─ MockProvider   (today)
                                              ├─ MtnProvider    (contract day)
                                              ├─ OrangeProvider
                                              └─ WaveProvider
                                     Ledger (trait)
                                              ├─ InMemoryLedger (today)
                                              └─ PostgresLedger (Supabase, later)
```

- `src/providers/` — one file per operator, all implementing the same trait.
- `src/ledger/` — swap in-memory for Postgres without touching routes.
- `src/config.rs` — env decides which operator is live and with which secrets.

## Roadmap

- **Phase 1 (done):** API + mock + in-memory ledger + idempotency + signed webhooks.
- **Phase 2:** wire `MtnProvider` to the free MTN MoMo **sandbox**; Postgres ledger.
- **Phase 3 (contract):** flip `DEFAULT_PROVIDER`, fill real credentials, go live.

## Security notes

- Secrets live server-side only. Never ship operator keys in a web/mobile app.
- Webhooks are HMAC-SHA256 verified (`X-Signature`). Unverified callbacks are
  rejected — otherwise anyone could fake a "success".
- Amounts are integer `i64` (XOF has no minor unit). No floats for money.

## License

MIT.
