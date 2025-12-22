# Constitution ID Index

NOTE: This file is GENERATED. Do not edit manually.
Generator: `python scripts/constitution_ids.py generate --write`

This index is link-only. All substance lives in the canonical Constitution documents.

## Invariants

- [INV-0001](./invariants.md#INV-0001) — Deterministic Simulation
- [INV-0002](./invariants.md#INV-0002) — Fixed Timestep
- [INV-0003](./invariants.md#INV-0003) — Authoritative Simulation
- [INV-0004](./invariants.md#INV-0004) — Simulation Core Isolation
- [INV-0005](./invariants.md#INV-0005) — Tick-Indexed I/O Contract
- [INV-0006](./invariants.md#INV-0006) — Replay Verifiability
- [INV-0007](./invariants.md#INV-0007) — Deterministic Ordering & Canonicalization

## Domain Model

- [DM-0001](./domain-model.md#DM-0001) — Tick
- [DM-0002](./domain-model.md#DM-0002) — World
- [DM-0003](./domain-model.md#DM-0003) — Character
- [DM-0004](./domain-model.md#DM-0004) — LocomotionMode
- [DM-0005](./domain-model.md#DM-0005) — Entity
- [DM-0006](./domain-model.md#DM-0006) — InputCmd
- [DM-0007](./domain-model.md#DM-0007) — Snapshot
- [DM-0008](./domain-model.md#DM-0008) — Session
- [DM-0009](./domain-model.md#DM-0009) — Channel
- [DM-0010](./domain-model.md#DM-0010) — Match
- [DM-0011](./domain-model.md#DM-0011) — Server Edge
- [DM-0012](./domain-model.md#DM-0012) — Matchmaker
- [DM-0013](./domain-model.md#DM-0013) — Game Server Instance
- [DM-0014](./domain-model.md#DM-0014) — Simulation Core
- [DM-0015](./domain-model.md#DM-0015) — Game Client
- [DM-0016](./domain-model.md#DM-0016) — Baseline
- [DM-0017](./domain-model.md#DM-0017) — ReplayArtifact
- [DM-0018](./domain-model.md#DM-0018) — StateDigest
- [DM-0019](./domain-model.md#DM-0019) — PlayerId
- [DM-0020](./domain-model.md#DM-0020) — EntityId
- [DM-0021](./domain-model.md#DM-0021) — MatchId
- [DM-0022](./domain-model.md#DM-0022) — InputTickWindow
- [DM-0023](./domain-model.md#DM-0023) — LastKnownIntent (LKI)
- [DM-0024](./domain-model.md#DM-0024) — AppliedInput
- [DM-0025](./domain-model.md#DM-0025) — TargetTickFloor
- [DM-0026](./domain-model.md#DM-0026) — InputSeq
- [DM-0027](./domain-model.md#DM-0027) — StepInput

## Acceptance Criteria

- [AC-0001](./acceptance-kill.md#AC-0001) — v0 Two-Client Multiplayer Slice

## Kill Criteria

- [KC-0001](./acceptance-kill.md#KC-0001) — Simulation Core Boundary Violation
- [KC-0002](./acceptance-kill.md#KC-0002) — Replay Verification Blocker

## Architecture Decision Records

- [ADR-0000](../adr/0000-adr-template.md) — ADR XXXX: <Title>
- [ADR-0001](../adr/0001-authoritative-multiplayer-architecture.md) — ADR 0001: Authoritative Multiplayer Architecture
- [ADR-0002](../adr/0002-deterministic-simulation.md) — ADR 0002: Deterministic Simulation
- [ADR-0003](../adr/0003-fixed-timestep-simulation.md) — ADR 0003: Fixed Timestep Simulation Model (Tick-Driven)
- [ADR-0004](../adr/0004-server-authoritative-architecture.md) — ADR 0004: Server-Authoritative Architecture
- [ADR-0005](../adr/0005-v0-networking-architecture.md) — ADR 0005: v0 Networking Architecture
- [ADR-0006](../adr/0006-input-tick-targeting.md) — ADR 0006: Input Tick Targeting & Server Tick Guidance
- [ADR-0007](../adr/0007-state-digest-algorithm-canonical-serialization.md) — ADR 0007: StateDigest Algorithm (v0)
