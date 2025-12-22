# Constitution Tag Taxonomy

Tags are controlled metadata used to generate navigation views (e.g., by-tag indices) and to help humans/agents discover related constraints.

Tags are NOT identity:
- Identity is the Constitution ID (e.g., INV-0007).
- Tags may change over time without breaking links, but churn should be rare and intentional.

## Rules

- Tags MUST be chosen from the allowlist below.
- Tags MUST be lowercase kebab-case (preferred). snake_case is allowed only if unavoidable.
- Prefer 1–3 tags per entry. Avoid tag sprawl.
- Each tag has a single, stable meaning (defined below). Do not use synonyms.
- Adding/removing/renaming a tag in the allowlist is a Constitution change.

## How to choose tags

- Choose 1 PRIMARY tag: what you would search for a year from now.
- Add up to 2 SUPPORT tags only if they materially improve retrieval/discovery.
- Do not encode priority, status, sprint scope, or “this week’s initiative” in tags.
- Avoid micro-tags for specific mechanics (e.g., “mantle”, “glide”). Put that specificity in the entry title and the domain model (DM-*).

## Allowed Tags

### Simulation correctness (cross-cutting)

- determinism: Fixed-step simulation; deterministic outcomes given identical inputs/seed/tuning hash.
- replay: Record/replay, verification via replay hashes, golden trajectories, and determinism validation.
- authority: Server-authoritative outcomes; trust boundaries; convergence; engine-agnostic simulation law.
- entity: Simulation objects (characters, projectiles, spawned gameplay objects) and their identity/lifecycle.
- security: Input validation, anti-cheat foundations, trusted/untrusted boundaries.
- testability: Isolation of Simulation Core to enable pure unit testing without I/O mocking.
- verification: Mechanisms for proving correctness (state hashing, replay equivalence, checkpoint comparison).

### Architecture

- architecture: Component separation, boundary contracts, module layering.
- traceability: Tick-indexing, provenance of state transitions, replay artifact structure.
- identity: Unique identifiers for entities, sessions, and other tracked objects.
- presentation: Game Client concerns: rendering, UI, input capture, interpolation. Non-authoritative.
- infrastructure: Deployment, orchestration, matchmaking. Outside the Game Server Instance boundary.

### Netcode / protocol semantics

- simulation: Simulation-plane architecture (ticks, state/events) independent of transport concerns.
- networking: Transport/session concerns; packet flow; connection lifecycle.
- schema: Versioned data formats (inputs/snapshots/events) and compatibility guarantees.
- prediction: Client prediction/rollback/reconciliation/interpolation policies.
- protocol: Message formats, wire protocols, serialization schemes.
- transport: Underlying delivery mechanisms (ENet, WebTransport, channels).
- input: Client input capture, encoding, and transmission.
- state-sync: Snapshot generation, packing, and client state reconciliation.
- connection: Session establishment, handshake, disconnection, reconnection.

### Gameplay pillars

- controls: Input semantics and guarantees (e.g., WASD locomotion + mouse aiming; decoupled move/aim contract).
- movement: Locomotion verbs/modes/state machines (dash/jump/glide, recovery moves, traversal rules).
- physics: Momentum/impulses/collision response/kinematics that define “feel” and movement tech headroom.
- combat: Ability rules, damage/CC systems, combat state constraints, combat-movement coupling.
- hazard: Lethal edges/voids/hazard volumes; recovery tools; risk economics at boundaries.
- learnability: Drills, terminology, teachable tech, onboarding-to-mastery scaffolding.

### Governance / sustainability

- operations: Canonical commands, CI parity, validation workflows, contributor ergonomics.
- orchestration: Match lifecycle/control-plane services that must not contain game rules.
- preservability: Self-hostability; no required proprietary centralized services.
- licensing: Permissive posture; dependency policy.
- provenance: Third-party source tracking for code/assets.

### Delivery phases (reserved)

- phase0
- phase1
- phase2
- phase3

Phase tags SHOULD be used only on AC-* entries (and any single document that defines phase semantics), to prevent phase tagging from polluting invariant/domain discovery.

### Documentation meta (documentation-only)

- glossary: Terminology definitions, naming, and canonical vocabulary.
- tests: Test-only law (test strategy requirements, golden-case obligations, determinism harness rules).
