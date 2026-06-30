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

## [2026-06] Hermes runs locally, not on the VPS

**Decision**: Hermes Agent runs on the developer's laptop with a Docker-sandboxed
terminal backend. It is not deployed on the Oracle VPS.

**Reasoning**: The original plan assumed both the merchant server and Hermes
would run on the same VPS instance. After provisioning, the actual free-tier
instance available was smaller than initially planned (2 OCPU / 12 GB RAM,
not the originally expected 4 OCPU / 24 GB). Running an LLM-driven agent
alongside the merchant server on a constrained instance was deprioritized —
the merchant server is the production-facing component and gets the VPS
resources; Hermes, as a development/testing tool, runs locally where resources
are not a concern.

**Trade-off**: Hermes must reach the merchant server over the public network
(VPS public address) rather than over localhost. This required opening the
relevant port both in OCI's Security List (network-level firewall) and in the
VM's local `iptables` rules (OS-level firewall) — two independent layers that
both needed explicit allow rules.

---

## [2026-06] Docker as Hermes' sandboxed backend

**Decision**: Run Hermes with `terminal.backend: docker` rather than `local`,
after first validating the agent works correctly in local mode.

**Reasoning**: Hermes (the LLM, via Gemini) decides what commands to run;
Hermes-the-program (the agent framework) is what actually executes them. The
risk is not that Hermes misbehaves on its own, but that the underlying LLM
makes a poor decision (ambiguous instruction, prompt injection from a fetched
web page, etc.) and the framework faithfully executes it. Docker contains
that risk by isolating where commands actually run, without requiring
separate hardware (the originally considered alternative: a dedicated Mac
Mini or Raspberry Pi for agent isolation).

Validated first in `local` mode (per official quickstart guidance — confirm
the agent works before adding isolation, to avoid debugging both at once),
then switched to `docker` with no behavior change and no measurable latency
difference.

---

## [2026-06] Native ARM compilation on VPS (no cross-compilation)

**Decision**: Compile Rust binaries directly on the Oracle ARM VPS rather than
cross-compiling from the local x86_64 laptop.

**Reasoning**: Cross-compilation adds toolchain complexity (linkers, sysroots)
with no practical benefit for a single-developer project. Rust on ARM is
mature enough that `cargo build` works without extra configuration. Workflow:
develop locally → git push → git pull on VPS → cargo build --release.

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

## [2026-06] Hermes + Gemini for the buyer agent

**Decision**: Use Hermes (Nous Research) as the autonomous buyer agent,
backed by a Gemini model via Google AI Studio free tier.

**Reasoning**: Hermes is skill-driven — the buyer behavior is defined in a
Markdown file, not code. This keeps the agent layer configuration rather than
programming, and provides hands-on experience with the agent framework itself.

**Lesson learned**: free-tier rate limits vary significantly *per model*, not
uniformly across "the free tier." A model release (e.g. a newer Flash variant)
can carry a much lower free quota than an adjacent one (e.g. a Lite variant).
The practical model choice was driven by checking actual per-model quotas in
the provider dashboard rather than assuming newer/default means better quota.

**Lesson learned**: without a skill, the agent reliably guesses REST endpoint
paths by trial and error (`/checkout`, `/checkout/create`, `/sessions`, etc.)
rather than failing fast — burning many requests and tokens per attempt. This
both wastes quota and pollutes context size, accelerating rate-limit errors.
A skill with explicit, fixed paths eliminates this entirely — confirmed with
a side-by-side test (no skill vs. with skill) against the same merchant
server.

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

## [2026-06] Oracle Cloud as infrastructure — chosen with eyes open

**Decision**: Use Oracle Cloud Infrastructure for VPS hosting at the PoC stage.

**Reasoning**: OCI's Always Free tier remains the most generous permanent free
compute offer in the market (ARM compute + generous storage, no expiration),
clearly ahead of alternatives — AWS dropped its 12-month free tier for new
accounts in 2025 in favor of shorter-lived credits, GCP's free tier is more
limited (1 GB RAM instance), and paid alternatives (Hetzner, Vultr) are not
free even if cheap.

**Known risk, confirmed firsthand**: the account upgrade process (Always Free
→ Pay-As-You-Go) took over two weeks and required escalating through multiple
support tickets and tax-document verification before resolving. Independent
reviews corroborate that OCI support for small/individual accounts is
inconsistent — in contrast to reportedly strong support for enterprise
accounts with dedicated solutions architects. This is treated as a real,
documented signal, not just a one-off bad experience.

**Implication for production**: this project remains a learning lab / PoC,
not a production deployment, and the infrastructure choices reflect that
(no auth, no HTTPS, no monitoring — see `architecture.md` Phase 5). When this
project moves toward any real exposure, the OCI support experience is a
genuine input into re-evaluating the hosting provider — likely toward a paid
plan (OCI or otherwise) where support quality is less of an open question,
rather than assuming free tier scales smoothly into production.

---

## [2026-06] Operational details kept out of architecture.md

**Decision**: Instance-specific details (exact CPU/RAM allocation, public IP,
exact region, account upgrade status) are not recorded in `architecture.md`.

**Reasoning**: These details are either ephemeral (an IP or instance spec can
change on recreation) or not useful to a reader trying to understand the
system's design. `architecture.md` is treated as a living description of the
system aimed at being useful to any reader, including a future external one;
operational/account-specific detail lives here in `decisions.md` (as
reasoning/context) or in untracked local notes, not in the public-facing
architecture description.
