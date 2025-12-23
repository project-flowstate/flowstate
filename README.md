# Flowstate

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![CI](https://github.com/project-flowstate/flowstate/actions/workflows/ci.yml/badge.svg)](https://github.com/project-flowstate/flowstate/actions/workflows/ci.yml)

Flowstate is an MIT-licensed, open-source competitive multiplayer game inspired by SUPERVIVE.

It is built architecture-first: a deterministic, server-authoritative simulation core with clean boundaries comes before content, art, or polish. The goal is long-term leverage, correctness, and preservability.

## Status

**v0 Multiplayer Slice Complete** (December 2025)

The authoritative server foundation is implemented and validated:

- **Simulation Core** (`crates/sim`) — Deterministic fixed-timestep world with FNV-1a state digest, WASD movement (5.0 units/sec)
- **Wire Protocol** (`crates/wire`) — Protobuf message types (ClientHello, ServerWelcome, InputCmdProto, SnapshotProto, ReplayArtifact)
- **Replay System** (`crates/replay`) — Full verification pipeline with initialization and outcome anchors
- **Server Edge** (`crates/server`) — Input validation, buffer management, LastKnownIntent fallback, session lifecycle

**Test Coverage:** 58 tests passing (sim: 16, wire: 6, replay: 8, server: 28)

**What's Missing:** Networking transport layer (ENet integration) and game client. Server operates in "manual step mode" for now.

## What we’re optimizing for

- Deterministic simulation and replay verification
- Clean authority boundaries (Simulation Core isolated from Server Edge and Game Client)
- Transport-independent protocol semantics
- Testability as a first-class feature
- Human + agent collaboration within explicit constraints

## Quickstart

Run the full local validation surface:

```
just ci
```

Constitution ID tooling (when editing canonical Constitution docs):

```
just ids
just ids-gen
```

## Read next

- Game vision and design intent: [`docs/vision.md`](./docs/vision.md)
- Constitution (kernel): [`docs/constitution.md`](./docs/constitution.md)
- Constitution annexes (invariants, domain model, criteria): [`docs/constitution/`](./docs/constitution/)
- Architecture decisions: [`docs/adr/`](./docs/adr/)
- How the project operates (human): [`docs/handbook.md`](./docs/handbook.md)
- Repo routing / where things go: [`docs/repo-map.md`](./docs/repo-map.md)
- Agent operating rules: [`AGENTS.md`](./AGENTS.md)
- Licensing policy and third-party intake: [`docs/licensing/`](./docs/licensing/)

## Contributing

Contributions are welcome, but must align with the Constitution and applicable ADRs.

Start with:
- [Contributing Guide](./.github/CONTRIBUTING.md)
- [`docs/handbook.md`](./docs/handbook.md)

## License

MIT. See [`LICENSE`](./LICENSE).
