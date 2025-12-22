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

**Statement:** The authoritative simulation instance MUST be the single source of truth for all game-outcome-affecting state. Clients are untrusted. Client messages MUST be treated as intent and MUST NOT directly author authoritative state transitions.

Inputs MUST be validated and sanitized by the Server Edge (DM-0011) before they affect authoritative state or are delivered to the Simulation Core (DM-0014). The Simulation Core MUST NOT perform client-facing validation, authentication, or identity binding; it operates only on StepInput (DM-0027) values derived from AppliedInput (DM-0024) by the Server Edge.

Player identity binding is a trust boundary: PlayerId (DM-0019) assignment and binding to Session (DM-0008) is exclusively owned by the Server Edge; the Simulation Core treats PlayerId only as an indexing/ordering key. Client-provided PlayerId values (explicit or implicit) MUST be ignored and overwritten by the Server Edge with the session-assigned PlayerId before inputs reach the Simulation Core.

*Non-normative note: Game-outcome-affecting state includes but is not limited to: hits, damage, status effects, resource changes, entity spawns/despawns, ability cooldowns. This rule applies regardless of whether the authoritative instance runs on a dedicated server, listen-server, or relay architecture.*

### <a id="INV-0004"></a> INV-0004 — Simulation Core Isolation
**Status:** Active  
**Tags:** architecture, determinism, testability

**Statement:** The Simulation Core (DM-0014) MUST NOT perform I/O operations, networking, rendering, wall-clock time reads, or system calls. All external communication MUST occur through explicit, serializable message boundaries owned by the Server Edge (DM-0011). Explicit seeded randomness consistent with INV-0001 is permitted.

Debug/telemetry mechanisms (e.g., time synchronization probes, instrumentation) MUST NOT influence authoritative outcomes; they may observe and report, but MUST NOT change simulation decisions or state transitions.

**Enforcement (normative):** This isolation MUST be mechanically enforced in CI for the Simulation Core crate/module. At minimum:
- Dependencies for the Simulation Core MUST be allowlisted (or equivalently, non-allowlisted dependencies MUST be denied) to prevent accidental introduction of I/O/network/time capabilities.
- CI MUST run a forbidden-API/source scan over the Simulation Core to reject disallowed surfaces, including (at minimum): file and socket I/O, system calls, wall-clock time reads, thread sleeps, environment access, and unseeded/ambient randomness.
- Conditional-compilation or build-flag “escape hatches” (e.g., `cfg(...)` paths) MUST NOT be used to introduce prohibited capabilities into the Simulation Core in any build mode.

*Non-normative note: This enables determinism (INV-0001), testability, and replay (INV-0006). The simulation may use seeded RNG as long as the seed is recorded. "Serializable message boundaries" means no function pointers, closures, or ambient state in the interface.*

### <a id="INV-0005"></a> INV-0005 — Tick-Indexed I/O Contract
**Status:** Active  
**Tags:** replay, traceability, networking

**Statement:** All inputs delivered to the Simulation Core (DM-0014) and all outputs emitted from it MUST carry an explicit Tick (DM-0001) identifier. This identifier MUST represent the simulation tick at which the input is applied or the output is generated, not network timestamps or client-local time. For any given session/channel stream, Tick identifiers MUST be monotonic non-decreasing.

*Non-normative note: This boundary contract enables replay (INV-0006) and allows future delay compensation schemes without changing the simulation interface. "Applied at tick T" means the input affects the state transition from T to T+1. Monotonicity prevents pathological replay streams.*

### <a id="INV-0006"></a> INV-0006 — Replay Verifiability
**Status:** Active  
**Tags:** determinism, replay, verification

**Statement:** The system MUST support match reproduction from a ReplayArtifact (DM-0017).

**v0 Replay Scope:** For v0, the declared replay scope MUST be "same produced binary artifact on the same target triple/profile." Replay verification MUST execute the exact produced artifact that generated the ReplayArtifact. Rebuilding, relinking, or substituting dependencies or resources between the authoritative run and verification is forbidden under v0 scope.

A ReplayArtifact MUST be versioned and self-describing enough to reproduce the authoritative outcome under the declared replay scope. At minimum, it MUST include:
- a format/version identifier,
- sufficient initialization data to reconstruct the authoritative starting state for replay (e.g., an initial Baseline (DM-0016) or equivalent canonical state seed + deterministic initialization parameters),
- all determinism-relevant seed(s)/parameters and any required algorithm identifiers (e.g., RNG algorithm ID; StateDigest algorithm ID),
- a chronologically ordered stream of AppliedInput (DM-0024) values with tick associations (i.e., the per-tick inputs that were actually applied by the authoritative simulation, not raw client messages),
- a defined verification anchor: checkpoint_tick and associated StateDigest(s) (DM-0018) required by the replay procedure (e.g., checkpoint digest and/or final digest).

**Tick-Boundary Checkpoint Semantics:** Any recorded checkpoint_tick MUST denote the world tick value immediately after completing the last applied tick step (post-step). Replay anchors (including checkpoint ticks and digests) MUST NOT be recorded mid-tick. Authoritative termination with respect to replay anchoring MUST occur only at tick boundaries.

**Initialization Verification Anchor:** Replay validation MUST verify at least one initialization anchor before applying any replayed inputs or steps (e.g., a StateDigest of the initial Baseline or pre-step starting state) in addition to at least one outcome anchor (e.g., final or checkpoint digest). If the initialization anchor fails, replay verification MUST fail immediately.

Replay validation MUST verify outcome equivalence by comparing the computed StateDigest at the declared anchor tick(s) to the digest recorded in the ReplayArtifact.

*Non-normative note: Equivalence verification may use state hashing, periodic snapshots, or final outcome comparison; the key requirement is that the system can prove determinism in practice, not just assert it in theory. "Authoritative outcome" is robust to early termination or disconnect scenarios. For v0, replay verification is scoped to same build + same platform (cross-platform determinism deferred to post-v0; see ADR-0005 Determinism Scope).*

### <a id="INV-0007"></a> INV-0007 — Deterministic Ordering & Canonicalization
**Status:** Active  
**Tags:** determinism, verification, traceability

**Statement:** Whenever the Simulation Core processes a set or collection that can affect outcomes (e.g., multiple inputs, entities, events, collisions) within a tick, it MUST use a stable, deterministic ordering that is independent of runtime artifacts (hash iteration order, pointer/address order, thread scheduling, or platform-specific container behavior).

When a canonical byte representation is required for verification (e.g., StateDigest (DM-0018) computation), the state-to-bytes process MUST be explicitly canonicalized such that equivalent authoritative states serialize identically. The digest/canonicalization procedure MUST be versionable and unambiguous (e.g., via algorithm identifiers recorded in the ReplayArtifact (DM-0017)).

*Non-normative note: Deterministic ordering is typically achieved by sorting on stable keys such as PlayerId (DM-0019) for StepInput (DM-0027) values and EntityId (DM-0020) for entity iteration. Canonicalization must eliminate representational ambiguity (e.g., stable iteration order; stable float handling; stable serialization layout) so replay verification is meaningful.*

## Invariant Change Policy

- Adding an invariant requires a maintainer decision and usually an ADR.
- Weakening/removing an invariant is an exceptional event and requires explicit maintainer approval plus an ADR that explains why the invariant is no longer valid.
