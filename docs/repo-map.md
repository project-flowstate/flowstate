# Repository Map

This document is the routing table for Flowstate’s repository: what each major area is for, where new information should go, and how to avoid duplication.

This file is intentionally short. It is a map, not a second handbook.

## Authority

The canonical authority / precedence definition lives in one place:
- `docs/constitution.md#authority-map`

## Start Here

- `README.md`: front door (high-level intro + start links)
- `docs/vision.md`: game design intent (non-authoritative; informs Constitution)
- `docs/handbook.md`: human-readable operating model
- `docs/constitution.md` + `docs/constitution/*`: definitions, invariants, domain model, acceptance/kill criteria
- `docs/adr/*`: binding decisions (within Constitution constraints)
- `docs/licensing/*`: operational policies (e.g., third-party intake)
- `AGENTS.md`: operational rules for agentic work

## Where information should go

- Product thesis, invariants, domain model, acceptance/kill criteria:
  - `docs/constitution.md`
  - `docs/constitution/*`

- Decisions that should not be relitigated:
  - `docs/adr/*`

- Operational policies and repeatable enforcement rules:
  - `docs/licensing/*` (and other policy folders as they emerge)

- Derived, module-scoped implementable clarity (optional):
  - `docs/specs/*`
  Specs are derived artifacts and should trace back to Constitution IDs (INV-*, DM-*).

- Agent operational constraints:
  - `AGENTS.md`

- Human onboarding and repo orientation:
  - `docs/handbook.md`
  - `README.md`

- Game vision and design intent (non-authoritative; informs but does not override Constitution):
  - `docs/vision.md`

## Repo topology (what lives where)

- `.github/`: GitHub automation, templates, workflows, contribution scaffolding
- `docs/`: governance, policies, and orientation docs
- `crates/`: Rust crates (Simulation Core and shared libraries)
- `client/`: client implementation(s) (presentation plane; engine-facing)
- `protocol/`: engine-agnostic schemas/messages

## Anti-duplication rule (critical)

Prefer linking over repeating.

- If content is authoritative somewhere (Constitution/ADR/policy), other docs should link to it.
- If two documents must be edited together to stay consistent, the structure is wrong.

## Naming conventions

- ADRs: `docs/adr/NNNN-title-in-kebab-case.md`
- Constitution annexes: `docs/constitution/<topic>.md`
- Policies: `docs/<area>/<topic>.md` (e.g., `docs/licensing/third-party.md`)
- Human docs: `docs/<topic>.md` (e.g., `docs/handbook.md`, `docs/repo-map.md`)
