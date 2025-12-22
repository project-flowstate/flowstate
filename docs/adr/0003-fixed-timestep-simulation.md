# ADR 0003: Fixed Timestep Simulation Model (Tick-Driven)

## Status
Accepted

## Type
Technical

## Context
Variable delta time (frame-rate-dependent simulation) causes non-determinism and makes replay verification and testing unreliable. The Simulation Core must advance independently of frame rate and wall-clock time.

A per-step `dt_seconds` argument is a determinism footgun: even if intended constant, it invites accidental variation and weakens the Simulation Core boundary.

## Decision
The Simulation Core (DM-0014) MUST advance in **fixed discrete ticks**. Tick duration is a match-level constant derived from match configuration and MUST NOT vary within a match.

The Simulation Core stepping API MUST be tick-driven and MUST NOT accept per-call delta time.

### Normative Requirements
- Match start configuration MUST declare `tick_rate_hz`; `dt_seconds = 1.0 / tick_rate_hz` is implied and constant for the match.
- Tick rate MUST be stored on the Simulation Core instance (e.g., World) for the match; it MUST NOT be sourced from global/process state.
- Simulation stepping MUST be `advance(tick: Tick, step_inputs: &[StepInput]) -> Snapshot` (or equivalent). The tick parameter is the explicit boundary tick per INV-0005; tick duration is implicit to the Simulation Core instance (configured at construction).
- The Simulation Core MUST assert that the provided tick matches its internal state (e.g., `tick == world.tick()`) to prevent misalignment.
- StepInput (DM-0027) is the simulation-plane input type; the Server Edge MUST convert AppliedInput (DM-0024) to StepInput before invoking `advance()`.
- State transitions MUST occur only on tick boundaries.
- Simulation logic MUST NOT depend on:
  - frame rate / render cadence
  - wall-clock time
  - variable delta time
- Time-based gameplay rules SHOULD be authored in seconds (cooldowns, durations, speeds). Implementations MAY convert these to tick-domain constants at match initialization, provided the conversion is deterministic and derived solely from match configuration.

### Initial Supported Tick Rate
- v0: 60 Hz (see [docs/networking/v0-parameters.md](../networking/v0-parameters.md))
- Additional discrete tick rates (e.g., 30/120 Hz) require validation before adoption.

## Rationale
**Why fixed timestep:**
- **Prerequisite for determinism:** Same inputs + same tick rate → same outputs (ADR-0002)
- **Consistent gameplay:** Movement/physics behave identically on slow and fast hardware
- **Replayability:** Recorded inputs can be replayed at any speed without changing outcomes
- **Networking:** Clients and server can synchronize on tick numbers

**Why tick-driven (no per-call dt):**
- Eliminates a major determinism footgun (accidental dt variation)
- Clarifies Simulation Core boundary (tick rate is configuration, not runtime parameter)
- "Author in seconds" still works; deterministic init-time conversion enables multi-rate support

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
  - DM-0024 (AppliedInput) — Server Edge input selection
  - DM-0027 (StepInput) — Simulation Core input type
- Canonical Constitution docs:
  - [docs/constitution/invariants.md](../constitution/invariants.md)
  - [docs/constitution/domain-model.md](../constitution/domain-model.md)
- Related ADRs:
  - ADR-0001 (Authoritative Multiplayer Architecture) — defines Simulation Core where fixed timestep applies
  - ADR-0002 (Deterministic Simulation) — fixed timestep enables determinism

## Alternatives Considered
- **Variable delta time** — `advance(state, inputs, dt_variable)` where dt changes per frame. Rejected: Non-deterministic; same inputs produce different outcomes at different frame rates.
- **Per-call `dt_seconds` constant-in-practice** — `advance(inputs, dt_seconds)` where dt is "supposed to be" constant. Rejected: Footgun; boundary leak; invites accidental variation.
- **Frame-locked simulation** — Couple simulation to rendering frame rate. Rejected: Determinism requires independence from frame rate.
- **Hardcoded tick rate (no configurability)** — Single tick rate, no match-level configuration. Rejected: Less flexible for future game modes; testing at different rates is useful.
- **Fully variable tick rate** — Allow tick rate to change mid-match. Rejected: Numerical integration differences cause subtle gameplay changes; validation surface area explodes.

## Implications
- **Enables:** Determinism (ADR-0002), replay verification, tick-synchronized networking, consistent gameplay across hardware
- **Constrains:** Simulation cannot adapt to low frame rates; must use fixed-timestep integration techniques; tick rate is chosen at match start
- **Migration costs:** None (greenfield project)
- **Contributor impact:** Contributors MUST keep wall-clock time out of the Simulation Core; any time progression is tick-indexed and configuration-derived. Replay artifacts MUST record tick configuration sufficient to reproduce the match's tick duration.

## Follow-ups
- Define concrete `World::new(seed, tick_rate_hz)` and `World::advance(tick, step_inputs)` API in simulation crate
- Document integration patterns for movement/physics at fixed timestep
- Validate behavior at 30/60/120 Hz before expanding supported tick rates
- Add CI gate: verify same outcomes at same tick rate across multiple runs
