# System Invariants

These are non-negotiable invariants. ADRs, specs, implementation, and tests MUST comply.

Each invariant has a stable Constitution ID. Derived artifacts MUST reference invariants by ID (not by section name).

<!--
ENTRY FORMAT (use H3 with anchor):

  ### <a id="INV-NNNN"></a> INV-NNNN - Title
  **Status:** Active  
  **Tags:** tag1, tag2

  **Statement:** The system MUST...

Section groupings (H2) are optional, for organization only.
-->

## Simulation Correctness

### <a id="INV-0001"></a> INV-0001 — Deterministic Simulation
**Status:** Active  
**Tags:** determinism, simulation, replay

**Statement:** The authoritative simulation MUST produce identical outcomes given identical initial state, input sequence, seed, and tuning parameters. Randomness MUST be explicit, seeded, and recorded where it affects gameplay outcomes.

### <a id="INV-0002"></a> INV-0002 — Fixed Timestep
**Status:** Active  
**Tags:** determinism, simulation, physics

**Statement:** The simulation MUST advance in fixed-size Ticks. Game logic and physics MUST NOT depend on frame rate, wall-clock time, or variable delta time.

### <a id="INV-0003"></a> INV-0003 — Authoritative Simulation
**Status:** Active  
**Tags:** authority, networking, security

**Statement:** The authoritative simulation instance MUST be the single source of truth for all game-outcome-affecting state. Clients are untrusted. Client messages MUST be treated as intent and MUST NOT directly author authoritative state transitions. Inputs MUST be validated before affecting authoritative state.

*Non-normative note: Game-outcome-affecting state includes but is not limited to: hits, damage, status effects, resource changes, entity spawns/despawns, ability cooldowns. This rule applies regardless of whether the authoritative instance runs on a dedicated server, listen-server, or relay architecture.*

### <a id="INV-0004"></a> INV-0004 — Simulation Core Isolation
**Status:** Active  
**Tags:** architecture, determinism, testability

**Statement:** The Simulation Core (DM-0014) MUST NOT perform I/O operations, networking, rendering, wall-clock time reads, or system calls. All external communication MUST occur through explicit, serializable message boundaries owned by the Server Edge (DM-0011). Explicit seeded randomness consistent with INV-0001 is permitted.

*Non-normative note: This enables determinism (INV-0001), testability, and replay (INV-0006). The simulation may use seeded RNG as long as the seed is recorded. "Serializable message boundaries" means no function pointers, closures, or ambient state in the interface.*

### <a id="INV-0005"></a> INV-0005 — Tick-Indexed I/O Contract
**Status:** Active  
**Tags:** replay, traceability, networking

**Statement:** All inputs delivered to the Simulation Core (DM-0014) and all outputs emitted from it MUST carry an explicit Tick (DM-0001) identifier. This identifier MUST represent the simulation tick at which the input is applied or the output is generated, not network timestamps or client-local time. For any given session/channel stream, Tick identifiers MUST be monotonic non-decreasing.

*Non-normative note: This boundary contract enables replay (INV-0006) and allows future delay compensation schemes without changing the simulation interface. "Applied at tick T" means the input affects the state transition from T to T+1. Monotonicity prevents pathological replay streams.*

### <a id="INV-0006"></a> INV-0006 — Replay Verifiability
**Status:** Active  
**Tags:** determinism, replay, verification

**Statement:** The system MUST support match reproduction from a replay artifact. A replay artifact MUST include: initial state snapshot, random seed(s), ruleset/tuning parameters, and chronologically ordered input stream with tick associations. Replay validation MUST verify equivalence of authoritative outcome at a defined checkpoint (e.g., match end tick or final state hash).

*Non-normative note: Equivalence verification may use state hashing, periodic snapshots, or final outcome comparison. The key requirement is that the system can prove determinism in practice, not just assert it in theory. "Authoritative outcome" is robust to early termination or disconnect scenarios. For v0, replay verification is scoped to same build + same platform (cross-platform determinism deferred to post-v0; see ADR-0005 Determinism Scope).*

## Invariant Change Policy

- Adding an invariant requires a maintainer decision and usually an ADR.
- Weakening/removing an invariant is an exceptional event and requires explicit maintainer approval plus an ADR that explains why the invariant is no longer valid.
