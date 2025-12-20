# Golden Delivery Path

> Protocol for constitution-driven development. Machine-parseable; agent-friendly.
>
> **Authority:** This document defines the delivery workflow. For invariants and domain model, see [`constitution.md`](constitution.md).

---

## Checkpoints

```
Issue → Governance Fit Check → Spec → PR → Merge
```

| Checkpoint | Gate | Artifact |
|------------|------|----------|
| Issue | Governance classification selected | Issue (feature_request.yml or governance-change.yml) |
| Governance Fit | Classification determines required artifacts | See Governance Classification Table |
| Spec | Spec linter passes (`just spec-lint`) | `docs/specs/FS-NNNN-slug.md` |
| PR | Trace block valid (`just pr-trace`) | PR with sentinel trace block |
| Merge | `just ci` passes | Merged commit |

---

## Governance Classification Table

| Classification | Definition | Required Artifacts | Escalation |
|---------------|------------|-------------------|------------|
| **Fits existing** | All IDs exist; expressible within current invariants/domain | Spec → Implementation PR | None |
| **Clarify wording** | Ambiguity fix; no semantic change, no new IDs | Constitution prose-only PR | Reclassify if semantic change |
| **New domain concept** | New DM-* entry needed | DM entry PR → Spec → Implementation PR | DM lands before/with impl |
| **New invariant** | New INV-* entry | Governance issue → Constitution PR + ADR + Spec w/ Tier 0 gate | Maintainer approval required |
| **Change invariant** | Modify/weaken/remove INV-* | Governance issue → Constitution PR + ADR + migration + rollback | Maintainer approval; highest bar |
| **Architectural decision** | Cross-cutting choice; no INV change | ADR PR → Spec → Implementation PR | ADR lands before impl |

### Classification Rules

1. **Fits existing** is the default. Use it unless one of the other conditions applies.
2. **Clarify wording** requires a credible "no semantic change" argument. If disputed, reclassify.
3. **New domain concept** triggers when a spec introduces a term not in `DM-*` entries.
4. **New invariant** and **Change invariant** require explicit maintainer approval before merge.
5. **Architectural decision** applies to cross-cutting choices with plausible alternatives (e.g., "ECS vs OOP").

---

## Spec Naming Convention

```
docs/specs/FS-NNNN-slug.md
```

- `FS-` = FlowState prefix (stable)
- `NNNN` = GitHub issue number (zero-padded to 4 digits)
- `slug` = lowercase-hyphenated short name

**Examples:**
- `docs/specs/FS-0042-player-movement.md`
- `docs/specs/FS-0123-input-replay.md`

**Lookup rule:** Given issue `#N`, search for `docs/specs/FS-N-*.md` (with or without zero-padding).

---

## Trace Block Format

PRs MUST contain a sentinel-fenced trace block for deterministic parsing.

### Format

~~~markdown
```trace
Issue: #NNNN
Spec: docs/specs/FS-NNNN-slug.md
Constitution: INV-0001, DM-0005
ADRs: ADR-0003
```
~~~

### Keys (stable, case-sensitive)

| Key | Format | Required | Notes |
|-----|--------|----------|-------|
| `Issue` | `#NNNN` | Yes | GitHub issue number |
| `Spec` | Path or `N/A: trivial` | Yes | Relative path from repo root |
| `Constitution` | Comma-separated IDs | Yes | INV-*, DM-*, AC-*, KC-*; can be empty |
| `ADRs` | Comma-separated IDs or `None` | Yes | ADR-NNNN format |

### Escape Hatch: Trivial Changes

Use `Spec: N/A: trivial` for changes that do not require a spec.

**Trivial criteria:**
- Documentation-only changes
- Configuration-only changes
- Code changes <30 lines with no simulation impact

Reviewers may reject `trivial` classification if criteria are not met.

---

## Spec Metadata Format

Specs MUST begin with a YAML frontmatter block:

```yaml
---
status: Draft | Approved | Implemented
issue: NNNN
title: Short descriptive title
---
```

### Status Definitions

| Status | Meaning | Linter Behavior |
|--------|---------|-----------------|
| `Draft` | Work in progress; not yet reviewed | `NEW:` domain concepts allowed (warning) |
| `Approved` | Reviewed and approved; ready for implementation | `NEW:` concepts must resolve to DM-* (error) |
| `Implemented` | Implementation merged | `NEW:` concepts must resolve to DM-* (error) |

---

## Spec Required Sections

The spec linter enforces presence of these sections:

| Section | Enforcement | Description |
|---------|-------------|-------------|
| `## Problem` | Required | What problem this solves |
| `## Issue` | Required | Link to GitHub issue |
| `## Trace Map` | Required | Constitution and ADR IDs this spec implements/constrains |
| `## Domain Concepts` | Required | DM-* IDs used; `NEW: ConceptName` for draft concepts |
| `## Interfaces` | Required | Types, resources, messages added/changed |
| `## Determinism Notes` | Required | Impact on simulation correctness |
| `## Gate Plan` | Required | Tier 0/1/2 gates (Tier 0 must have ≥1 bullet) |
| `## Acceptance Criteria` | Required | Observable outcomes that define "done" |
| `## Non-Goals` | Optional | Explicitly out of scope |
| `## Risks` | Optional | Known risks and mitigations |
| `## Alternatives` | Optional | Other approaches considered |

---

## Gate Tiers

| Tier | Requirement | Examples |
|------|-------------|----------|
| **Tier 0** | Must pass before merge | Format, lint, build, unit tests, determinism checks |
| **Tier 1** | Must be tracked as follow-up issue with owner | Performance budgets, soak tests, extended replays |
| **Tier 2** | Aspirational; not yet formalized | "Movement feel" metrics, UX polish |

**Enforcement:** Tier 0 section must contain at least one bullet item.

---

## PR Validation Rules

The `pr_trace.py` script enforces:

1. **Trace block present:** PR body contains ` ```trace ` sentinel block
2. **Issue declared:** `Issue: #N` is present
3. **Spec linkage:** If `docs/specs/FS-N-*.md` exists for the issue, `Spec:` must reference it
4. **Spec exists:** If `Spec: <path>` declared, file must exist
5. **Bidirectional match:** If `Spec: <path>` declared, spec's `issue:` metadata must match PR's `Issue:`
6. **Constitution IDs valid:** All IDs in `Constitution:` exist in catalog
7. **ADR IDs valid:** All IDs in `ADRs:` exist in catalog (or `None`)

---

## ID Formats

| Prefix | Source | Example |
|--------|--------|---------|
| `INV-NNNN` | `docs/constitution/invariants.md` | `INV-0001` |
| `DM-NNNN` | `docs/constitution/domain-model.md` | `DM-0005` |
| `AC-NNNN` | `docs/constitution/acceptance-kill.md` | `AC-0001` |
| `KC-NNNN` | `docs/constitution/acceptance-kill.md` | `KC-0001` |
| `ADR-NNNN` | `docs/adr/NNNN-*.md` | `ADR-0003` |

All IDs are validated against `docs/constitution/id-catalog.json`.

---

## Workflow Examples

### Example 1: Feature that fits existing constitution

```
1. Create issue using feature_request.yml
   → Select "Fits existing Constitution + Domain Model"
2. Create spec: docs/specs/FS-0042-player-jump.md
   → Run: just spec-lint
3. Create implementation PR with trace block
   → Run: just ci-pr
4. Review and merge
```

### Example 2: Feature requiring new domain concept

```
1. Create issue using feature_request.yml
   → Select "Requires new domain concept (DM-*)"
2. Create governance PR adding DM-NNNN to domain-model.md
   → Run: just ids-gen && just ci
3. Create spec referencing new DM-NNNN
4. Create implementation PR
5. Review and merge (DM PR first or together)
```

### Example 3: Invariant change

```
1. Create issue using governance-change.yml
   → Select "Change existing invariant"
   → Include: justification, semantic impact, rollback criteria
2. Get maintainer approval on issue
3. Create ADR documenting decision
4. Create Constitution PR modifying INV-*
5. Create spec with Tier 0 gate for new invariant
6. Create implementation PR
7. Review chain and merge in order
```
