# Copilot Instructions

> Repository-wide instructions for GitHub Copilot.
> For the full operating model, see [`AGENTS.md`](../AGENTS.md).

## Quick Reference

```bash
just ci          # Full validation (fmt + lint + test + ids + spec-lint)
just ci-pr       # PR-specific CI (changed-only spec lint)
just fmt         # Check formatting
just lint        # Clippy lints
just test        # Run tests
just ids         # Validate Constitution IDs, tags, references
just ids-gen     # Regenerate derived index files (commit the output)
just spec-lint   # Validate all specs (structure, IDs, gate plans)
just spec-lint-file FILE  # Validate a single spec file
just pr-trace    # Validate PR trace block (stdin)
just pr-trace-file FILE   # Validate PR trace block from file
```

## Repository Structure

| Path | Purpose |
|------|---------|
| `crates/sim/` | Authoritative simulation kernel (Rust) |
| `client/` | Presentation plane (future) |
| `protocol/` | Engine-agnostic schemas (future) |
| `docs/constitution.md` | Authoritative definitions and invariants |
| `docs/constitution/` | Constitution annexes (invariants, domain model, criteria) || `docs/delivery-path.md` | Golden Delivery Path protocol |
| `docs/specs/` | Feature specs (named `FS-NNNN-slug.md`) || `docs/adr/` | Architecture Decision Records |
| `AGENTS.md` | Agent operating rules and workflow |

## Authority Hierarchy

When instructions conflict, follow this precedence (highest first):

1. Maintainer instructions in the current Issue/PR
2. Constitution (`docs/constitution.md` + `docs/constitution/*`)
3. ADRs (`docs/adr/*`)
4. `AGENTS.md`
5. This file

Full authority map: [`docs/constitution.md#authority-map`](../docs/constitution.md#authority-map)

## Build & Validation

### Prerequisites
- Rust toolchain: pinned in `rust-toolchain.toml` (auto-installed by rustup)
- Python: required for `just ids` / `just ids-gen`
- Just: task runner (`cargo install just` or platform package manager)

### Validation Steps

Always run before committing:
```bash
just ci
```

If you edit files under `docs/constitution/`:
```bash
just ids-gen   # Regenerate indices
git add docs/constitution/id-index.md docs/constitution/id-index-by-tag.md docs/constitution/id-catalog.json
```

## Constitution IDs

This repository uses stable Constitution IDs for traceability:

| Prefix | Meaning | File |
|--------|---------|------|
| `INV-####` | Invariants (must always hold) | `docs/constitution/invariants.md` |
| `DM-####` | Domain Model (canonical vocabulary) | `docs/constitution/domain-model.md` |
| `AC-####` | Acceptance Criteria (testable "done") | `docs/constitution/acceptance-kill.md` |
| `KC-####` | Kill Criteria (stop/re-scope triggers) | `docs/constitution/acceptance-kill.md` |
| `ADR-####` | Architecture Decision Records | `docs/adr/NNNN-*.md` |

**In commits, issues, PRs, and tests:** reference explicit IDs (e.g., `INV-0001`, `DM-0005`).

**Machine-readable catalog:** `docs/constitution/id-catalog.json`

## Code Patterns

### Rust
- 4-space indent (rustfmt authoritative)
- Prefer `cargo clippy` with `-D warnings`
- Tests should reference Constitution IDs in comments when verifying invariants

### Simulation Plane
- **Determinism is non-negotiable** — see `INV-0001`, `INV-0002`
- Fixed timestep; no frame-rate-dependent logic
- Authoritative state transitions only in `crates/sim`

## PR Checklist

Before opening a PR:
- [ ] `just ci` passes
- [ ] `just ids` passes
- [ ] `just spec-lint` passes (if spec exists)
- [ ] If Constitution docs changed: `just ids-gen` run and outputs committed
- [ ] PR contains valid `trace` block (Issue, Spec, Constitution, ADRs)
- [ ] PR references relevant Constitution IDs (INV/DM/AC/KC/ADR)
- [ ] Spec exists for non-trivial changes (`docs/specs/FS-NNNN-slug.md`)
- [ ] No new dependencies added without updating `docs/licensing/third-party.md`
- [ ] **Changes committed to feature branch (NOT main) - see AGENTS.md for Git workflow**

## Key Invariants (Reference)

For quick context, the most critical invariants are:

- **INV-0001**: Deterministic stepping

Full list: [`docs/constitution/invariants.md`](../docs/constitution/invariants.md)

## When Uncertain

If intent is ambiguous or instructions conflict:
1. Do not guess
2. Propose options with tradeoffs
3. Request maintainer decision

See: [`AGENTS.md`](../AGENTS.md) for detailed agent operating rules.
