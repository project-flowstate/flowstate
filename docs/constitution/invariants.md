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

## Invariant Change Policy

- Adding an invariant requires a maintainer decision and usually an ADR.
- Weakening/removing an invariant is an exceptional event and requires explicit maintainer approval plus an ADR that explains why the invariant is no longer valid.
