# ADR 0002: Deterministic Simulation

## Status
Accepted

## Type
Technical

## Context
Competitive multiplayer games require verifiable correctness. Players expect consistent outcomes: the same inputs should produce the same results every time. Without determinism, replay verification is impossible, testing becomes unreliable, and subtle bugs can manifest as desync or inconsistent match outcomes across clients.

The Simulation Core (DM-0014, defined in ADR-0001) is the authoritative source of truth for game state. If the simulation is non-deterministic, there is no "truth"—only probabilistic approximations.

## Decision
The authoritative simulation MUST be **deterministic**: identical initial state, input sequence, seed, and tuning parameters MUST produce identical outcomes across all runs, platforms, and compiler configurations.

**Requirements:**
- All randomness MUST be explicit, seeded, and recorded
- Simulation logic MUST NOT depend on:
  - Wall-clock time or timestamps
  - Variable delta time or frame rate
  - Floating-point operations with platform-specific behavior
  - Pointer addresses or memory layout
  - Thread scheduling or async execution order
  - External I/O or network state
- Simulation state transitions MUST be reproducible from recorded inputs

**Permitted:**
- Fixed-point arithmetic or carefully-controlled float operations with defined rounding
- Explicit random number generators with recorded seeds
- Deterministic collision resolution with stable ordering

## Rationale
**Why determinism:**
- Enables replay verification (record inputs → replay → verify identical outcomes)
- Makes testing reliable (same test input → same result, always)
- Simplifies debugging (reproduce exact game state from inputs)
- Enables rollback netcode (if needed in future)
- Prevents subtle desyncs in distributed simulation

**Why strict enforcement:**
- "Mostly deterministic" is indistinguishable from "non-deterministic" in production
- Hidden non-determinism (e.g., float rounding differences) causes rare, unreproducible bugs
- Replay verification only works if determinism is guaranteed, not probabilistic

**Tradeoffs:**
- Cannot use platform-provided random() without seeding
- Cannot use arbitrary floating-point math (must quantize or use fixed-point)
- Cannot read system time during simulation
- May sacrifice some "natural randomness" for reproducibility

## Constraints & References (no prose duplication)
- Constitution IDs:
  - INV-0001 (Deterministic Simulation) — canonical definition
- Canonical Constitution docs:
  - [docs/constitution/invariants.md](../constitution/invariants.md)
- Related ADRs:
  - ADR-0001 (Authoritative Multiplayer Architecture) — defines Simulation Core where determinism applies
  - ADR-0003 (Fixed Timestep) — fixed timestep is a prerequisite for determinism

## Alternatives Considered
- **Probabilistic simulation** — Allow non-deterministic behavior, accept occasional desyncs. Rejected: Violates correctness goals; makes replay and testing unreliable.
- **Deterministic only on single platform** — Guarantee determinism on Linux x64 only. Rejected: Clients run on multiple platforms; desyncs would occur across platforms.
- **Epsilon-tolerance determinism** — Allow small float differences. Rejected: Epsilon errors accumulate; "close enough" is not verifiable.

## Implications
- **Enables:** Replay verification, reliable testing, rollback netcode (future), simplified debugging
- **Constrains:** No use of `rand()`, `time()`, or platform-specific float behavior in simulation
- **Migration costs:** None (greenfield project)
- **Contributor impact:** Simulation code must avoid common non-deterministic patterns (unseeded RNG, system time, unordered maps)

## Follow-ups
- Establish replay recording/playback infrastructure in simulation crate
- Add CI gate: run determinism tests (same inputs → same outputs)
- Document deterministic coding patterns in handbook
- Define acceptable RNG/float/collision resolution strategies
