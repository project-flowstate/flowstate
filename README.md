# Flowstate

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![CI](https://github.com/project-flowstate/flowstate/actions/workflows/ci.yml/badge.svg)](https://github.com/project-flowstate/flowstate/actions/workflows/ci.yml)

Flowstate is an MIT-licensed, open-source competitive multiplayer game inspired by SUPERVIVE.

It is built architecture-first: a deterministic, server-authoritative simulation core with clean boundaries comes before content, art, or polish. The goal is long-term leverage, correctness, and preservability.

## Status

Foundation phase. Initial workspace + simulation scaffolding is in place. No gameplay slice yet.

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
