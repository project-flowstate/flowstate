# ADR 0003: Fixed Timestep Simulation Model

## Status
Accepted

## Type
Technical

## Context
Variable delta time (frame-rate-dependent simulation) causes non-determinism: the same inputs produce different outcomes depending on how fast the frame loop runs. This makes replay verification impossible, testing unreliable, and gameplay inconsistent across hardware.

To achieve determinism (ADR-0002), the simulation must advance in predictable, fixed-size increments independent of frame rate or wall-clock time.

## Decision
The Simulation Plane (ADR-0001) MUST advance in **fixed discrete ticks**. Each tick represents a fixed duration of simulated time.

**Requirements:**
- Simulation stepping function: `advance(state, inputs, dt_seconds)` where `dt_seconds` is constant for a match
- Tick rate is a **match-level constant** (declared in protocol handshake, e.g., 60 Hz → dt = 16.67ms)
- All time-based rules MUST be expressed in seconds (cooldowns, durations, velocities), not tick counts
- Game logic and physics MUST NOT depend on:
  - Frame rate or frames-per-second
  - Wall-clock time or variable delta time
  - Rendering vsync or monitor refresh rate
- Simulation state transitions occur ONLY on tick boundaries

**Initial supported tick rate:** 60 Hz (may expand to 30/120 Hz in future)

## Rationale
**Why fixed timestep:**
- **Prerequisite for determinism:** Same inputs + same dt → same outputs (ADR-0002)
- **Consistent gameplay:** Movement/physics behave identically on slow and fast hardware
- **Replayability:** Recorded inputs can be replayed at any speed without changing outcomes
- **Networking:** Clients and server can synchronize on tick numbers

**Why dt-parameterized (not hardcoded tick count):**
- Time-based rules (e.g., "3 second cooldown") are intuitive and tunable
- Allows testing at different tick rates without rewriting logic
- Scales to variable tick rates (30/60/120 Hz) if needed in future

**Why match-level constant (not global):**
- Different game modes may benefit from different tick rates
- Flexibility without sacrificing determinism within a match
- Client can adapt prediction based on declared tick rate

**Tradeoffs:**
- Simulation cannot respond to frame-rate changes (intentional: consistency > responsiveness)
- Tick rate must be chosen carefully (too low = choppy, too high = CPU cost)
- Numerical integration differences at different dt values mean tick rate cannot vary freely without re-validation

## Constraints & References (no prose duplication)
- Constitution IDs:
  - INV-0002 (Fixed Timestep) — canonical definition
  - DM-0001 (Tick) — atomic unit of game time
- Canonical Constitution docs:
  - [docs/constitution/invariants.md](../constitution/invariants.md)
  - [docs/constitution/domain-model.md](../constitution/domain-model.md)
- Related ADRs:
  - ADR-0001 (Three-Plane Architecture) — defines Simulation Plane where fixed timestep applies
  - ADR-0002 (Deterministic Simulation) — fixed timestep enables determinism

## Alternatives Considered
- **Variable delta time** — `advance(state, inputs, dt_variable)` where dt changes per frame. Rejected: Non-deterministic; same inputs produce different outcomes at different frame rates.
- **Frame-locked simulation** — Couple simulation to rendering frame rate. Rejected: Determinism requires independence from frame rate.
- **Hardcoded tick rate (no dt parameter)** — Simulation logic uses tick counts instead of seconds. Rejected: Less intuitive; harder to tune; cannot test at different tick rates.
- **Fully variable tick rate** — Allow tick rate to change mid-match. Rejected: Numerical integration differences cause subtle gameplay changes; validation surface area explodes.

## Implications
- **Enables:** Determinism (ADR-0002), replay verification, tick-synchronized networking, consistent gameplay across hardware
- **Constrains:** Simulation cannot adapt to low frame rates; must use fixed-timestep integration techniques
- **Migration costs:** None (greenfield project)
- **Contributor impact:** All simulation logic must be written in terms of `dt_seconds`, not frame count or wall-clock time

## Follow-ups
- Define simulation stepping API: `advance(state, inputs, dt_seconds)`
- Document integration patterns for movement/physics at fixed timestep
- Validate behavior at 30/60/120 Hz before expanding supported tick rates
- Add CI gate: verify same outcomes at same tick rate across multiple runs
