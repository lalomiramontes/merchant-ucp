# Project: UCP Merchant Server

Architecture and project context document.

---

## Goal

Build a merchant server implementing the Universal Commerce Protocol (UCP) with support
for multiple payment methods (fiat and crypto), with an autonomous agent (Hermes) acting
as buyer for end-to-end testing.

The approach is **not crypto-centric** — x402/USDC is one payment option among others
(Stripe, PayPal, etc.). The merchant is payment-method agnostic.

This is currently a learning lab / PoC, not a production service. See
`decisions.md` for the reasoning behind each architectural choice.

---

## Tech Stack

| Component        | Technology                        |
|------------------|------------------------------------|
| Merchant server  | Rust + Axum                       |
| Database         | PostgreSQL (Phase 2)              |
| Buyer agent      | Hermes (Nous Research)            |
| Agent LLM        | Gemini (Google AI Studio)         |
| Infrastructure   | Oracle Cloud Infrastructure (ARM VPS) |
| Isolation        | Docker (service isolation; Hermes runs sandboxed locally) |
| Version control  | Git                                |

---

## Protocols

### Phase 1 — UCP (Universal Commerce Protocol)
- Spec: `ucp.dev`, version `2026-04-08`
- Developed by Google + Shopify
- Covers the full commerce lifecycle: discovery, catalog, checkout, payment, fulfillment
- Transport: REST (this project); UCP also defines MCP, A2A, and Embedded transports
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
│       └── mod.rs           # In-memory state (Phase 1), PostgreSQL (Phase 2)
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

## Infrastructure

The merchant server runs on an ARM-based VPS (Oracle Cloud Infrastructure).
See `decisions.md` for why OCI was chosen and its known limitations.
Instance-specific details (exact specs, IP, region) are intentionally not
documented here since they can change; see local/private notes if needed.

### Local development → VPS workflow
1. Develop and test on laptop (x86_64)
2. `git push` when working
3. On the VPS: `git pull && cargo build --release`
4. Native ARM compilation on the VPS — no cross-compilation or Docker buildx
   (Rust on ARM is mature enough that this adds no friction)

### Docker — service isolation only
Docker is not used for compilation. It is used to isolate:
- The merchant server process (optional, low priority — it's a compiled
  Rust binary, not arbitrary code)
- Hermes Agent (high priority — runs an LLM-driven agent with terminal/file
  access; isolating it limits the blast radius of any unexpected action)

Hermes runs locally on the developer's laptop with a Docker-sandboxed
terminal backend. It is not deployed on the VPS (see `decisions.md`).

---

## Hermes Agent (buyer agent)

### What it is
- Open-source framework by Nous Research (MIT license)
- Autonomous agent with persistent memory and a skills system
- Installed as a CLI tool, not a library — opaque to the user
- The user only writes skills in Markdown; Hermes executes the steps using
  its built-in tools (terminal, HTTP requests, etc.)
- Skill format follows the open `agentskills.io` standard, portable across
  Hermes, Claude Code, Cursor, and other compatible agents

### UCP buyer skill
File: `docs/skills/ucp-buyer.md` (also copied to `~/.hermes/skills/` locally).
Describes the UCP purchase flow step by step, including the exact fixed
endpoint paths — this avoids the agent guessing REST route names, which it
does unreliably on its own (e.g. trying `/checkout`, `/checkout/create`,
`/sessions`, etc. before succeeding).

A more complete community/expert skill set for UCP, ACP, and AP2 exists at
[OrcaQubits/agentic-commerce-skills-plugins](https://github.com/OrcaQubits/agentic-commerce-skills-plugins),
packaged for Claude Code but portable in principle. Useful as a reference
for more advanced patterns (AP2 mandates, conformance testing) once this
project's own skill is outgrown.

---

## Project Phases

### Phase 1 — UCP merchant server
- [x] Base Rust + Axum structure
- [x] Data models (Checkout, LineItem, Buyer, Message, etc.)
- [x] GET `/.well-known/ucp`
- [x] POST/GET/PUT/complete/cancel checkout-sessions
- [x] In-memory state
- [x] curl-based integration tests (happy path, cancel, error/409 case)

### Phase 2 — Persistence + Agent
- [ ] PostgreSQL with sqlx
- [x] Install and configure Hermes (local, Docker-sandboxed backend)
- [x] `ucp-buyer.md` skill
- [x] End-to-end flow: Hermes buys from the merchant (validated locally
      and against the VPS deployment)

### Phase 3 — Real payment handlers
- [ ] Stripe (fiat)
- [ ] x402 / USDC on Solana (stablecoin)
- [ ] MPP

### Phase 4 — VPS deploy + ACP
- [x] Configure Oracle VPS instance
- [x] Deploy merchant server on VPS
- [ ] Deploy PostgreSQL on VPS (via Docker)
- [ ] Implement ACP as a second commerce protocol

### Phase 5 — Security (currently missing)
The server has no authentication today. Anyone who knows the VPS address can
create, read, or complete checkouts with no identity verification. Acceptable
for a learning PoC; must be addressed before any real exposure.

- [ ] OAuth 2.0 authorization server (`/.well-known/oauth-authorization-server`)
      so platforms/agents obtain a token before acting on a buyer's behalf
      (UCP "identity linking")
- [ ] AP2 Mandates — cryptographically signed proof of buyer consent,
      required for fully-autonomous checkout without UI handoff. Conceptually
      similar to signing a Solana transaction, but for purchase authorization
      instead of fund transfer
- [ ] Rate limiting / basic abuse protection on public endpoints
- [ ] HTTPS

---

## Background Context

- Target market corridor: Mexico / USA / Canada
- Focus: enabling everyday commerce with stablecoins as one option,
  not the only method
- Rust/Axum style reference: StateSet repo `agentic_server/` (~1700 lines, Axum)
- Relevant prior experience: x402 with Kora on Solana devnet (full flow),
  Anchor/Solana (vault, escrow, AMM patterns)
