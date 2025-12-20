# Flowstate Constitution

This Constitution is the repository’s highest-level source of truth.

It defines:
- Product thesis and experience intent
- The authoritative precedence/authority map for resolving conflicts
- The contract by which humans and agents derive specs and code

Detailed definitions and long-form “law” live in the Constitution annexes under [`docs/constitution/`](./constitution/). Annexes are part of the Constitution and carry equal authority.

## Authority Map

Order of precedence (highest first):

1) Maintainer instructions in the current Issue/PR context  
2) Constitution (this kernel) + Constitution annexes ([docs/constitution/*](./constitution/))  
3) ADRs ([docs/adr/*](./adr/))  
4) Policies (e.g., [docs/licensing/*](./licensing/))  
5) Agent operating rules ([AGENTS.md](../AGENTS.md))  
6) Human orientation docs ([docs/handbook.md](./handbook.md), [docs/repo-map.md](./repo-map.md))  
7) [README.md](../README.md), [docs/vision.md](./vision.md) (descriptive, non-authoritative)

If two artifacts conflict, defer to the higher authority. If intent is ambiguous, propose options with tradeoffs and request a maintainer decision rather than guessing.

## Constitution Annexes

All files listed below are authoritative and part of the Constitution:

- Tag taxonomy (allowlist): [docs/constitution/tag-taxonomy.md](./constitution/tag-taxonomy.md)
- Scope & non-goals: [docs/constitution/scope-non-goals.md](./constitution/scope-non-goals.md)
- System invariants (INV-*): [docs/constitution/invariants.md](./constitution/invariants.md)
- Domain model & glossary (DM-*): [docs/constitution/domain-model.md](./constitution/domain-model.md)
- Acceptance & kill criteria (AC-*, KC-*): [docs/constitution/acceptance-kill.md](./constitution/acceptance-kill.md)

## Product Thesis

Flowstate is an MIT-licensed, open-source competitive multiplayer game inspired by SUPERVIVE.

It is built architecture-first: a deterministic, server-authoritative simulation core with clean boundaries comes before content, art, or polish. The goal is long-term leverage, correctness, and preservability.

The project prioritizes:
- Correctness over convenience
- Architecture over content
- Determinism over ad-hoc behavior
- Testability and replayability as core features
- Human + agent collaboration within explicit constraints

## Spec Derivation Contract

Specs are derived artifacts. The Constitution and ADRs are authoritative.

Derived specs (e.g., `docs/specs/*`) MUST:
- Trace key requirements to invariant IDs (INV-*) and domain IDs (DM-*)
- Avoid introducing new domain concepts without first updating the domain model ([docs/constitution/domain-model.md](./constitution/domain-model.md))
- Use Constitution tags from the allowlist ([docs/constitution/tag-taxonomy.md](./constitution/tag-taxonomy.md)) to improve discoverability
- Prefer executable truth (tests, replay/golden cases) over prose
- State assumptions explicitly and make them testable when feasible

Derived specs MUST NOT:
- Change or weaken invariants without a maintainer-approved Constitution/ADR update
- Add dependencies without following [docs/licensing/*](./licensing/) policy and updating the third-party ledger

## Governance

- The maintainer is the final arbiter of architecture, scope, and design.
- Decisions that should not be relitigated are recorded as ADRs under [docs/adr/](./adr/).
- Contributions are welcome, but must align with this Constitution and applicable ADRs.
