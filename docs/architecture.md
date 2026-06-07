# Project: UCP Merchant Server

Architecture and project context document. Last updated: June 2026.

---

## Goal

Build a merchant server implementing the Universal Commerce Protocol (UCP) with support
for multiple payment methods (fiat and crypto), deployable on an ARM VPS, with an
autonomous agent (Hermes) acting as buyer for end-to-end testing.

The approach is **not crypto-centric** — x402/USDC is one payment option among others
(Stripe, PayPal, etc.). The merchant is payment-method agnostic.

---

## Tech Stack

| Component        | Technology                        | Notes                                    |
|------------------|-----------------------------------|------------------------------------------|
| Merchant server  | Rust + Axum                       | HTTP server, UCP business logic          |
| Database         | PostgreSQL                        | Checkout and order persistence           |
| Buyer agent      | Hermes (Nous Research)            | Autonomous UCP client via Markdown skills|
| Agent LLM        | Gemini Flash (Google AI Studio)   | Free tier, 1500 req/day, no credit card  |
| Infrastructure   | Oracle Cloud Always Free (PAYG)   | VM.Standard.A1.Flex, ARM aarch64         |
| Server OS        | Ubuntu 24.04 Minimal aarch64      | Home region: US West (San Jose)          |
| Isolation        | Docker (between services only)    | Not used for compilation                 |
| Version control  | Git                               | Local dev → VPS deploy                  |

---

## Protocols

### Phase 1 — UCP (Universal Commerce Protocol)
- Spec: `ucp.dev`, version `2026-04-08`
- Developed by Google + Shopify
- Covers the full commerce lifecycle: discovery, catalog, checkout, payment, fulfillment
- Transport: REST first, MCP later
- Platform-agnostic — any agent can consume it without registration

### Phase 4 — ACP (Agentic Commerce Protocol)
- Developed by OpenAI + Stripe
- Conversational checkout, complementary to UCP
- Added once UCP is fully understood and working

---

## Payment Handlers

Payment handlers are advertised in `/.well-known/ucp` and negotiated per transaction
by the agent. The merchant does not need to know which method the agent will use upfront.

| Handler               | Method          | Phase |
|-----------------------|-----------------|-------|
| Stripe                | Fiat, card      | 1     |
| PayPal                | Fiat            | 1     |
| x402 / USDC (Solana)  | Stablecoin      | 2     |
| MPP (Stripe + Tempo)  | Fiat + crypto   | 3     |

---

## Project Structure (Rust)

```
merchant-server/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── checkout.rs      # Checkout, CheckoutStatus, LineItem, Buyer, Message...
│   │   └── profile.rs       # UcpProfile, PaymentHandler, Service...
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── well_known.rs    # GET /.well-known/ucp
│   │   └── checkout.rs      # POST/GET/PUT/.../complete/.../cancel
│   └── store/
│       └── mod.rs           # In-memory state (phase 1), PostgreSQL (phase 2)
```

---

## UCP Flow (official spec 2026-04-08)

### Merchant endpoints

| Method | Endpoint                                    | Description            |
|--------|---------------------------------------------|------------------------|
| GET    | `/.well-known/ucp`                          | Merchant profile       |
| POST   | `/ucp/v1/checkout-sessions`                 | Create checkout        |
| GET    | `/ucp/v1/checkout-sessions/:id`             | Get checkout           |
| PUT    | `/ucp/v1/checkout-sessions/:id`             | Update checkout        |
| POST   | `/ucp/v1/checkout-sessions/:id/complete`    | Complete checkout      |
| POST   | `/ucp/v1/checkout-sessions/:id/cancel`      | Cancel checkout        |

### Checkout status lifecycle

```
incomplete ←→ requires_escalation (buyer handoff via continue_url)
    ↓
ready_for_complete
    ↓
complete_in_progress
    ↓
completed

canceled (can occur from any state — session expired)
```

### Error severities

| Severity               | Agent action                              |
|------------------------|-------------------------------------------|
| `recoverable`          | Fix via Update Checkout and retry         |
| `requires_buyer_input` | Hand off to user via continue_url         |
| `requires_buyer_review`| Hand off to user via continue_url         |
| `unrecoverable`        | Retry with a new session                  |

### Minimal `/.well-known/ucp` profile

```json
{
  "ucp": {
    "version": "2026-04-08",
    "services": {
      "dev.ucp.shopping": [{
        "version": "2026-04-08",
        "transport": "rest",
        "endpoint": "https://merchant.example.com/ucp/v1"
      }]
    },
    "capabilities": {
      "dev.ucp.shopping.checkout": [{ "version": "2026-04-08" }]
    },
    "payment_handlers": {
      "com.stripe": [{ "id": "stripe_1", "version": "2026-04-08" }]
    }
  }
}
```

---

## Infrastructure (Oracle Cloud)

### Account
- Type: Pay As You Go (PAYG) — Always Free resources, no charges while
  within the free tier limits
- Tenancy: `cuadrolabs`
- Home region: US West (San Jose)
- Auth: FIDO2 passkey (Google Password Manager) + user/password

### Instance (pending creation — upgrade in progress)
- Shape: `VM.Standard.A1.Flex`
- OCPUs: 4
- RAM: 24 GB
- Boot volume: 50 GB
- OS: Canonical Ubuntu 24.04 Minimal aarch64
- Architecture: ARM (aarch64)

### Always Free resources included
- 200 GB Block Volume total
- 20 GB Object Storage
- 1 Load Balancer (10 Mbps)
- 10 TB outbound data transfer/month
- 2 Autonomous Database instances (20 GB each)

### Known risk
- Home region San Jose is a popular US region — possible "out of capacity"
  errors when creating ARM instances. If it occurs, retry periodically.
- Mitigated by PAYG account (priority over pure free tier accounts)

---

## Local Development → VPS

### Workflow
1. Develop and test on laptop (Ubuntu x86_64)
2. When working: `git push`
3. On the VPS: `git pull && cargo build --release`
4. Native ARM compilation on the VPS — no cross-compilation or Docker buildx

### Why compile directly on the VPS
- Rust on ARM is mature — `cargo build` works without extra configuration
- Simpler than cross-compilation or Docker multi-arch builds
- The resulting binary is correct for the hardware

### Docker — service isolation only
```
Oracle VPS
└── Ubuntu 24.04 ARM
      ├── container: merchant-server
      ├── container: hermes-agent
      └── container: postgresql
```
Docker is not used for compilation, only for isolating services in production.

---

## Hermes Agent (buyer agent)

### What it is
- Open-source framework by Nous Research (MIT license)
- Autonomous agent with persistent memory and a skills system
- Installed as a CLI tool, not a library — opaque to the user
- The user only writes skills in Markdown

### Installation
```bash
curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh | bash
hermes config set model google/gemini-flash-2.5
```

### LLM: Gemini Flash (free tier)
- 1,500 requests/day, 15 RPM
- No credit card, no expiration
- Sufficient for simulating UCP purchases in development
- Fallback: Groq (14,400 req/day, Llama/DeepSeek models, also free)

### UCP buyer skill (to be written in Phase 2)
File `~/.hermes/skills/ucp-buyer.md` — describes the purchase flow in natural
language. Hermes executes the steps using its built-in HTTP tools.
No Python, TypeScript, or Rust code required for the agent layer.

---

## Project Phases

### Phase 1 — UCP merchant server (in progress)
- [ ] Base Rust + Axum structure
- [ ] Data models (Checkout, LineItem, Buyer, Message, etc.)
- [ ] GET `/.well-known/ucp`
- [ ] POST/GET/PUT/complete/cancel checkout-sessions
- [ ] In-memory state
- [ ] curl-based integration tests

### Phase 2 — Persistence + Agent
- [ ] PostgreSQL with sqlx
- [ ] Install and configure Hermes
- [ ] ucp-buyer.md skill
- [ ] End-to-end flow: Hermes buys from the merchant

### Phase 3 — Real payment handlers
- [ ] Stripe (fiat)
- [ ] x402 / USDC on Solana (stablecoin)
- [ ] MPP

### Phase 4 — VPS deploy + ACP
- [ ] Configure Oracle VPS instance (pending upgrade)
- [ ] Deploy merchant server + Hermes + PostgreSQL via Docker
- [ ] Implement ACP as a second commerce protocol

---

## Background Context

- Target market corridor: Mexico / USA / Canada
- Focus: enabling everyday commerce with stablecoins as one option,
  not the only method
- Rust/Axum style reference: StateSet repo `agentic_server/` (~1700 lines, Axum)
- Relevant prior experience: x402 with Kora on Solana devnet (full flow),
  Anchor/Solana (vault, escrow, AMM patterns)
