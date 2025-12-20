---
status: Draft
issue: NNNN
title: Short descriptive title
---

# FS-NNNN: Short Descriptive Title

> **Status:** Draft | Approved | Implemented  
> **Issue:** [#NNNN](https://github.com/ORG/REPO/issues/NNNN)  
> **Author:** @username  
> **Date:** YYYY-MM-DD

---

## Problem

<!-- REQUIRED (linter-enforced) -->
<!-- What problem does this feature solve? Be specific. -->

## Issue

<!-- REQUIRED (linter-enforced) -->
<!-- Link to the GitHub issue. Must match frontmatter `issue` field. -->

- Issue: [#NNNN](https://github.com/ORG/REPO/issues/NNNN)

## Trace Map

<!-- REQUIRED (linter-enforced) -->
<!-- List Constitution and ADR IDs this spec implements or is constrained by. -->

| ID | Relationship | Notes |
|----|--------------|-------|
| INV-0001 | Constrains | Must maintain determinism |
| DM-0005 | Implements | Uses Tick concept |
| ADR-0003 | Implements | Follows decision X |

## Domain Concepts

<!-- REQUIRED (linter-enforced) -->
<!-- List domain concepts used. Mark new concepts with "NEW:" prefix. -->
<!-- NEW: concepts allowed in Draft status; must resolve to DM-* before Approved. -->

| Concept | ID | Notes |
|---------|-----|-------|
| Tick | DM-0005 | Existing |
| NEW: InputFrame | — | Proposed; needs DM entry |

## Interfaces

<!-- REQUIRED (linter-enforced) -->
<!-- What types, resources, or messages are added or changed? -->

### New Types

```rust
pub struct ExampleType {
    field: u32,
}
```

### Changed Types

- `ExistingType`: Added `new_field: Option<u32>`

### New Messages / Events

- None

## Determinism Notes

<!-- REQUIRED (linter-enforced) -->
<!-- How does this feature impact simulation correctness? -->
<!-- If no sim impact, state "No simulation impact." -->

- This feature affects simulation state by: ...
- Determinism preserved because: ...
- Replay verification: ...

## Gate Plan

<!-- REQUIRED (linter-enforced) -->
<!-- Tier 0 must have at least one bullet item. -->

### Tier 0 (Must pass before merge)

- [ ] Unit tests for `ExampleType`
- [ ] `just ci` passes
- [ ] Determinism check: replay test with fixed seed

### Tier 1 (Tracked follow-up)

- [ ] Performance benchmark (Issue: #TBD)
- [ ] Extended replay test (100k ticks)

### Tier 2 (Aspirational)

- [ ] "Feel" metrics (not yet formalized)

## Acceptance Criteria

<!-- REQUIRED (linter-enforced) -->
<!-- Observable outcomes that define "done". Must be objectively verifiable. -->

- [ ] Criterion 1: When X happens, Y is observable
- [ ] Criterion 2: System state satisfies Z after operation
- [ ] Criterion 3: `just ci` passes with new tests

## Non-Goals

<!-- OPTIONAL -->
<!-- What is explicitly out of scope? -->

- This spec does NOT address: ...
- Deferred to future work: ...

## Risks

<!-- OPTIONAL -->
<!-- Known risks and mitigations. -->

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Risk 1 | Low | Medium | Mitigation strategy |

## Alternatives

<!-- OPTIONAL -->
<!-- Other approaches considered and why they weren't chosen. -->

### Alternative A: ...

- Pros: ...
- Cons: ...
- Rejected because: ...

---

## Changelog

<!-- Update as the spec evolves. -->

| Date | Author | Change |
|------|--------|--------|
| YYYY-MM-DD | @username | Initial draft |
