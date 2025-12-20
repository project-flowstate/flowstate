# AGENTS.md

This repository is designed to be operated by humans and AI agents.

## Authority (Canonical)

The single canonical authority / precedence definition is the Constitution’s Authority Map:
- See [`docs/constitution.md#authority-map`](./docs/constitution.md#authority-map)

If instructions conflict or intent is ambiguous: do not guess. Propose options with tradeoffs and request a maintainer decision.

## Golden Delivery Path

All features and changes follow the delivery path defined in [`docs/delivery-path.md`](./docs/delivery-path.md):

1. **Issue** — Use YAML issue templates; select governance classification
2. **Governance Fit Check** — Determine if Constitution/ADR changes are needed
3. **Spec** — Create `docs/specs/FS-NNNN-slug.md` with required sections
4. **PR** — Include sentinel trace block; link to spec and issue
5. **Merge** — All gates pass (`just ci`)

**Key commands:**
- `just spec-lint` — Validate spec structure and ID references
- `just pr-trace` — Validate PR trace block (used in CI)
- `just ci-pr` — PR-specific CI (changed-only spec lint)

## Agent Operating Model

- Treat the Constitution as the definition layer (thesis, invariants, domain model, acceptance/kill criteria).
- Treat ADRs as the decision layer (why a path was chosen).
- Treat tests as the truth layer (what the system actually guarantees).
- Treat specs as derived artifacts that clarify a module or slice and may be regenerated when definitions change.

## Creating a Spec from an Issue

When asked to draft a spec from an issue or feature request:

1. **Read the issue** to understand the problem and acceptance criteria
2. **Verify governance fit** — Does it trace to existing Constitution IDs? If it requires new invariants or domain concepts, those must be added first via a governance PR.
3. **Copy the template**: `docs/specs/_TEMPLATE.md` → `docs/specs/FS-NNNN-slug.md` (use the issue number for NNNN)
4. **Fill all required sections** (see template comments for guidance):
   - Frontmatter: `status: Draft`, `issue: N`, `title: ...`
   - Problem, Issue, Trace Map, Domain Concepts, Interfaces, Determinism Notes, Gate Plan, Acceptance Criteria, Non-Goals
5. **Run validation**: `just spec-lint` to validate all specs
6. **If Draft status and NEW: concepts are referenced**, note them clearly in Domain Concepts table and flag for governance review
7. **Submit for approval** — Maintainer reviews and sets status to `Approved` before implementation begins

**Spec Lifecycle**: Specs remain in `docs/specs/` indefinitely. Status progression: `Draft → Approved → Implemented`. Implemented specs serve as historical design documentation.

## Implementing an Approved Spec

When asked to implement a spec (status: `Approved`):

1. **Read the spec thoroughly** — Understand Problem, Trace Map, Interfaces, Determinism Notes, and Gate Plan
2. **Identify affected files** — Usually in `crates/sim/src/` for simulation features
3. **Implement interfaces** — Add/modify types and methods as defined in the "Interfaces" section
4. **Write Tier 0 tests** — All items in "Gate Plan → Tier 0" must be implemented as tests in the same PR
5. **Run `just ci`** — Ensure all checks pass (fmt, lint, test, ids, spec-lint)
6. **Update spec status** — Change `status: Approved` → `status: Implemented` in the frontmatter
7. **Generate PR body** — Use `.github/PULL_REQUEST_TEMPLATE.md` to create the PR description, filling the `trace` block with Issue, Spec, Constitution IDs, and ADRs referenced in the spec's Trace Map. Save to `.github/prs/NNN-slug.md` (use issue number for NNN).
8. **Present summary** — Brief summary of changes made, test results, and location of PR body file

**Invocation:** When maintainer says "Implement the approved spec: docs/specs/FS-NNNN-slug.md", follow these steps without requiring additional instructions.

**Key principles:**
- Tier 0 tests are **mandatory** — they define "done"
- Determinism Notes guide implementation choices
- If implementation reveals spec issues, discuss with maintainer before proceeding

**Code documentation guidelines (optimize for agent context windows):**
- Always reference Constitution IDs in module/struct docs (e.g., `/// Ref: INV-0001, DM-0003`)
- Always add test comments linking to spec Gate Plan items (e.g., `// Tier 0 Gate: ...`)
- Add test module headers with spec reference (e.g., `//! Tests for FS-NNNN: Title` + `//! Spec: docs/specs/FS-NNNN-slug.md`)
- Keep doc comments concise; avoid examples for trivial methods (getters, constructors)
- Document complex logic and invariants, not obvious behavior
- Prefer clear code over explanatory comments
- For non-obvious constraint enforcement, add inline comments with Constitution IDs (e.g., `// INV-0001: Why this matters`)

## Spec Derivation Rules (Derived Specs)

When creating or updating derived specs (e.g., `docs/specs/*`), specs MUST:
- Trace key requirements to invariant IDs (INV-*) and domain IDs (DM-*)
- Avoid introducing new domain concepts without first updating the domain model (`docs/constitution/domain-model.md`)
- Prefer executable truth (tests, replays, golden trajectories) over prose
- State assumptions explicitly and make them testable when feasible

Specs MUST NOT:
- Change or weaken invariants without a maintainer-approved Constitution/ADR update
- Add dependencies without following `docs/licensing/*` policy and updating the third-party ledger

## Determinism (Non-Negotiable)

- The authoritative simulation must be deterministic and replay-verifiable.
- Randomness must be explicit, seeded, and recorded where it affects outcomes.
- If determinism is uncertain, strengthen tests before adding features.

## Command Contract (Local == CI)

`just` is the canonical task runner. Before opening or updating a PR, run the CI-equivalent checks locally.

Expected targets (names may evolve, but the contract must remain stable):
- `just fmt`
- `just lint`
- `just test`
- `just ci` (or `just check`) as the CI-equivalent meta target

If a required command is missing:
- Add the smallest stable target.
- Update CI in lockstep.
## Git Workflow (Critical)

**Branch protection is ACTIVE on `main`.** Direct pushes are blocked. All changes require PRs.

### Required Workflow

1. **Create feature branch**: `git checkout -b type/description`
   - Types: `feat/`, `fix/`, `chore/`, `docs/`
2. **Make changes and stage**: `git add <files>`
3. **Commit** (standalone command): `git commit -m "message"`
4. **Push branch**: `git push -u origin branch-name`
5. **Open PR on GitHub** with trace block
6. **Wait for CI to pass**
7. **Merge via GitHub UI**
8. **Switch back to main**: `git checkout main && git pull`

### Git Command Rules

- **NEVER chain git commands** with `;` or `&&` (e.g., `git add . && git commit && git push`)
- **Each git operation is a separate command** that requires maintainer approval
- **NEVER push directly to main** — it will be rejected by branch protection
- **Always use feature branches** for changes
- If you forget and try to push to main, CREATE A BRANCH from the current state

### Command Separation Examples

❌ Wrong:
```bash
git add . && git commit -m "message" && git push
```

✅ Correct:
```bash
git add .
# Wait for approval
git commit -m "message"
# Wait for approval  
git push
```
## PR Hygiene

- Keep PRs small and reviewable.
- Link the Issue and cite relevant Constitution/ADR sections (IDs preferred).
- Include a short verification note (commands run, results).
- Do not “fix” failing tests by deleting/weakening them unless explicitly instructed.
