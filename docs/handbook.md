# Flowstate Handbook

This handbook is the human-readable operating model for Flowstate: how the repo is structured and how work is performed. It is orientation, not authority.

## Authority

The canonical authority / precedence definition lives in one place:
- `docs/constitution.md#authority-map`

This handbook should link to canonical documents rather than restate them.

## What this project is

Flowstate is an MIT-licensed, open-source competitive multiplayer game inspired by SUPERVIVE.
The project prioritizes deterministic simulation correctness, clean architectural boundaries, and long-term maintainability.
Content and polish come later, after the simulation contract is proven.

## What to read (recommended order)

1. `README.md` (front door)
2. `docs/constitution.md` and `docs/constitution/*` (definitions, invariants, domain model, criteria)
3. `docs/adr/*` (binding decisions)
4. `AGENTS.md` (agent operating rules)
5. `docs/repo-map.md` (routing table)

## Architecture overview (conceptual)

Flowstate uses a three-plane conceptual model with strict boundaries:

- Simulation Plane (authoritative)
  Canonical game rules and state transitions. Deterministic and replay-verifiable. Engine-agnostic.

- Client Plane (presentation)
  Rendering, input capture, UI, interpolation, and eventually prediction/reconciliation. Downstream of authoritative state.

- Control Plane (orchestration; optional)
  Match lifecycle, hosting, telemetry, and dev tooling. Must not contain game rules.

The binding details belong in ADRs.

## How work enters the system

Preferred:
- A GitHub Issue describing intent, constraints, inputs/outputs, acceptance criteria, and non-goals.

Acceptable:
- A small PR that clearly states intent and references relevant authority (Constitution/ADR).

Principle:
- Avoid “wandering PRs.” Every change should have a reason, a boundary, and a validation path.

## Development contract (Local == CI)

- `just` is the canonical task runner.
- CI must run the same checks a contributor can run locally.
- CI failures must be reproducible locally.

Expected shape:
- `just fmt`
- `just lint`
- `just test`
- `just ci` (or `just check`)

When changing canonical Constitution docs, run `just ids-gen` to update committed generated artifacts.

## Determinism and testing philosophy

Determinism is foundational for the simulation plane:
- Identical inputs and seed must produce identical outcomes.
- Replay and golden tests are preferred mechanisms for correctness.
- If determinism is uncertain, strengthen tests before adding features.

## Licensing posture (high-level)

- Repo code is MIT licensed.
- Dependencies are permissive-only by default.
- Third-party intake requires provenance and documentation under `docs/licensing/`.

## Where should this change go?

- Changes to meaning, constraints, or definitions:
  Update the Constitution (and possibly add/update an ADR).

- Decisions that should not be relitigated:
  Add an ADR.

- Module clarification derived from existing definitions:
  Add/update a derived spec under `docs/specs/` that traces to Constitution IDs.

- Onboarding, contributor guidance, and operational notes:
  Update this handbook and/or `CONTRIBUTING.md` and/or `AGENTS.md`.
