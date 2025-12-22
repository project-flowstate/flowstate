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

## Domain Model

- [DM-0001](./domain-model.md#DM-0001) — Tick
- [DM-0002](./domain-model.md#DM-0002) — World
- [DM-0003](./domain-model.md#DM-0003) — Character
- [DM-0004](./domain-model.md#DM-0004) — Locomotion Mode
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

## Acceptance Criteria

- [AC-0001](./acceptance-kill.md#AC-0001) — v0 Two-Client Multiplayer Slice

## Kill Criteria

- [KC-0001](./acceptance-kill.md#KC-0001) — Simulation Core Boundary Violation
- [KC-0002](./acceptance-kill.md#KC-0002) — Replay Verification Blocker

## Architecture Decision Records

- [ADR-0000](../adr/0000-adr-template.md) — ADR XXXX: <Title>
- [ADR-0001](../adr/0001-authoritative-multiplayer-architecture.md) — ADR 0001: Authoritative Multiplayer Architecture
- [ADR-0002](../adr/0002-deterministic-simulation.md) — ADR 0002: Deterministic Simulation
- [ADR-0003](../adr/0003-fixed-timestep-simulation.md) — ADR 0003: Fixed Timestep Simulation Model
- [ADR-0004](../adr/0004-server-authoritative-architecture.md) — ADR 0004: Server-Authoritative Architecture
- [ADR-0005](../adr/0005-v0-networking-architecture.md) — ADR 0005: v0 Networking Architecture
