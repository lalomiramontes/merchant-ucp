# merchant-ucp

A protocol-agnostic merchant server implementing the Universal Commerce Protocol (UCP),
built with Rust and Axum. Designed for the MX/US/CA commerce corridor with support for
both fiat and stablecoin payment methods.

## Overview

This project explores agentic commerce: AI agents autonomously discovering merchants,
negotiating payment methods, and completing purchases — without human intervention at
checkout time.

The merchant server is intentionally payment-method agnostic. Stripe, PayPal, and
x402/USDC on Solana are all first-class payment handlers registered in the UCP profile.
The agent negotiates which one to use per transaction.

## Architecture

```
merchant-ucp/
├── docs/
│   ├── proyecto-merchant-ucp.md   # Full architecture and project context (ES)
│   ├── decisions.md               # Technical decision log
│   └── skills/
│       └── ucp-buyer.md           # Hermes agent skill (Phase 2)
├── merchant-server/               # Rust + Axum UCP server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── models/                # UCP data types
│       ├── routes/                # HTTP handlers
│       └── store/                 # State (in-memory → PostgreSQL)
├── docker-compose.yml             # Service orchestration (Phase 2+)
└── README.md
```

## Protocols

| Protocol | Role | Status |
|----------|------|--------|
| UCP (Google + Shopify) | Commerce layer: discovery, catalog, checkout | Phase 1 |
| ACP (OpenAI + Stripe) | Conversational checkout | Phase 4 |

## Payment Handlers

| Handler | Method | Phase |
|---------|--------|-------|
| Stripe | Fiat, card | 1 |
| PayPal | Fiat | 1 |
| x402 / USDC (Solana) | Stablecoin | 2 |
| MPP (Stripe + Tempo) | Fiat + stablecoin | 3 |

## Tech Stack

- **Server**: Rust + Axum
- **Database**: PostgreSQL (via sqlx)
- **Agent**: Hermes (Nous Research) with Gemini Flash free tier
- **Infrastructure**: Oracle Cloud Always Free — VM.Standard.A1.Flex (4 OCPU ARM, 24 GB RAM)
- **OS**: Ubuntu 24.04 Minimal aarch64

## Development

### Prerequisites

- Rust (stable)
- Docker (for service isolation on VPS)
- Gemini API key (Google AI Studio — free tier)

### Run locally

```bash
cd merchant-server
cargo run
```

### Deploy to VPS

```bash
# On the VPS (native ARM compilation)
git pull
cargo build --release
```

## Roadmap

- [x] Project architecture and documentation
- [ ] **Phase 1** — UCP merchant server (in-memory state)
  - [ ] `/.well-known/ucp` profile endpoint
  - [ ] Checkout session CRUD
  - [ ] Complete / cancel operations
  - [ ] curl-based integration tests
- [ ] **Phase 2** — Persistence + Agent
  - [ ] PostgreSQL with sqlx
  - [ ] Hermes install and configuration
  - [ ] `ucp-buyer.md` skill
  - [ ] End-to-end: Hermes buys from the merchant
- [ ] **Phase 3** — Real payment handlers
  - [ ] Stripe
  - [ ] x402 / USDC on Solana
  - [ ] MPP
- [ ] **Phase 4** — VPS deploy + ACP
  - [ ] Oracle Cloud instance setup
  - [ ] Docker service orchestration
  - [ ] ACP protocol implementation

## References

- [UCP Specification](https://ucp.dev/2026-04-08/specification/overview/)
- [Hermes Agent](https://hermes-agent.nousresearch.com/docs/)
- [x402 Protocol](https://x402.org)
- [Kora (Solana x402 facilitator)](https://github.com/solana-foundation/kora)
