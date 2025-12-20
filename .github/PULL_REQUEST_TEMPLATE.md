## Summary
What this PR changes (2–5 bullets max).

## Trace Block

<!--
REQUIRED: This sentinel block is parsed by scripts. Keep the format exact.
See docs/delivery-path.md for format specification.
-->

```trace
Issue: #
Spec: docs/specs/FS-NNNN-slug.md
Constitution: 
ADRs: None
```

<!--
Trace Block Keys:
- Issue: GitHub issue number (e.g., #42)
- Spec: Path to spec file, or "N/A: trivial" for docs/config/<30 lines
- Constitution: Comma-separated IDs (INV-*, DM-*, AC-*, KC-*), or empty
- ADRs: Comma-separated ADR-NNNN IDs, or "None"
-->

## Files Changed
List the primary files changed (helps reviewers and enables automation).
- `path/to/file.ext`

## Determinism / Simulation Impact
- [ ] This PR changes simulation-plane logic
  - If yes: describe determinism impact and verification method.

## Verification
What you ran and what passed.
- [ ] `just ci`
- [ ] `just ids`
- [ ] `just spec-lint` (if spec exists)
- [ ] If Constitution docs changed: `just ids-gen` and committed generated outputs

## Risk / Notes
Anything reviewers should pay attention to (edge cases, follow-ups, known limitations).

## Checklist
- [ ] Trace block is complete and accurate
- [ ] This PR stays within Constitution invariants (no silent weakening)
- [ ] No new dependencies added, or licensing intake updated under `docs/licensing/`
- [ ] Spec exists for non-trivial changes, or `N/A: trivial` justified
- [ ] Gates added per spec's Gate Plan (if applicable)
