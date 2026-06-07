# Technical Decisions

A running log of architectural and technical decisions made during development,
including context and reasoning. Useful for resuming work after a break or
providing context to AI assistants.

---

## [2026-06] In-memory state before PostgreSQL

**Decision**: Start with in-memory checkout state, add PostgreSQL later.

**Reasoning**: The UCP checkout flow is the core learning objective of Phase 1.
Adding a database layer before the flow works end-to-end adds unnecessary
complexity and friction. PostgreSQL will be introduced once the merchant server
handles the full checkout lifecycle correctly via curl tests.

**Trade-off**: Server restarts lose all active checkout sessions. Acceptable
for development and PoC; not acceptable for production.

---

## [2026-06] Single VM, Docker for service isolation

**Decision**: Deploy all services (merchant server, Hermes, PostgreSQL) on a
single Oracle Cloud VM, isolated via Docker containers.

**Reasoning**: Oracle Cloud Always Free (PAYG) provides 4 OCPUs and 24 GB RAM
on a single VM.Standard.A1.Flex instance. This is more than sufficient for
a PoC. Running multiple VMs adds networking complexity with no benefit at
this stage.

Docker provides process isolation (blast radius containment if Hermes behaves
unexpectedly) without the overhead of full VM separation.

**Future**: When production load justifies it, split DB to a separate VM and
add a Load Balancer in front of multiple merchant server instances. Oracle
free tier supports this within the same resource pool.

---

## [2026-06] Native ARM compilation on VPS (no cross-compilation)

**Decision**: Compile Rust binaries directly on the Oracle ARM VPS rather than
cross-compiling from the local x86_64 laptop.

**Reasoning**: Cross-compilation adds toolchain complexity (linkers, sysroots)
with no practical benefit for a single-developer project. The VPS has ample
CPU and RAM for native compilation. Workflow: develop locally → git push →
git pull on VPS → cargo build --release.

Docker buildx multi-arch would be the right choice if deploying to multiple
instances simultaneously, but that is not the current requirement.

---

## [2026-06] UCP before ACP

**Decision**: Implement UCP (Google + Shopify) first, add ACP (OpenAI + Stripe)
in Phase 4.

**Reasoning**: UCP has a more complete and better-documented public spec
(ucp.dev). It is surface-agnostic by design — any agent, including Hermes,
can consume it without platform registration. Both protocols cover the full
commerce lifecycle; UCP knowledge transfers directly to ACP implementation.

---

## [2026-06] Hermes + Gemini Flash for the buyer agent

**Decision**: Use Hermes (Nous Research) as the autonomous buyer agent,
backed by Gemini Flash via Google AI Studio free tier.

**Reasoning**: Hermes is skill-driven — the buyer behavior is defined in a
Markdown file, not code. This keeps the agent layer configuration rather than
programming, and provides hands-on experience with the agent framework itself.
Gemini Flash free tier (1,500 req/day, no credit card) is sufficient for
simulating UCP purchases in development.

Fallback: Groq free tier (14,400 req/day, Llama/DeepSeek models) if Gemini
rate limits become an issue.

---

## [2026-06] Payment-method agnostic design

**Decision**: The merchant server does not assume any specific payment method.
Payment handlers (Stripe, PayPal, x402/USDC, MPP) are registered in the UCP
profile and negotiated per transaction by the agent.

**Reasoning**: The project target is the MX/US/CA commerce corridor where both
fiat and stablecoin payments are relevant. Crypto (x402/USDC) is one option,
not the default. A payment-agnostic design reflects real merchant needs and
makes the codebase more generally useful.

---

## [2026-06] Oracle Cloud Always Free (PAYG) as infrastructure

**Decision**: Use Oracle Cloud PAYG account for VPS hosting.

**Reasoning**: Oracle Always Free provides 4 OCPUs ARM + 24 GB RAM + 200 GB
storage permanently at no cost — the most generous free tier in the market.
PAYG account eliminates idle instance reclamation risk.

**Known risks**:
- Home region US West (San Jose) is a popular region — possible "out of
  capacity" errors when creating ARM instances. Mitigation: retry periodically.
- No SLA on free tier. Acceptable for PoC; not for production.
- If instance creation repeatedly fails, fallback is Google Cloud free tier
  (e2-micro, 1 GB RAM) — sufficient for the merchant server binary alone.
