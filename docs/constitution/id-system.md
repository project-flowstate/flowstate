# Constitution ID System

This document defines the canonical identifier system (“Constitution IDs”) used to link requirements, decisions, specs, and code to the Constitution.

## Purpose

Constitution IDs exist to:
- create stable, unambiguous references for humans and agents
- enable reliable linking (docs → ADRs → specs → code/tests)
- reduce duplication by referencing authoritative statements instead of rewriting them
- keep long-term coherence as titles and wording evolve

## ID Taxonomy

Constitution IDs are grouped into four categories:

- **INV-####** — **Invariants**
  Non-negotiable, must-always-hold properties of the system and repository.

- **DM-####** — **Domain Model**
  Canonical vocabulary: entities, concepts, states, events, commands, and relationships.

- **AC-####** — **Acceptance Criteria**
  Testable statements defining “done.” Used to evaluate changes and specs.

- **KC-####** — **Kill Criteria**
  Explicit rejection/rollback conditions. Used to prevent “bad wins.”

## ID Format

IDs use a strict, stable format:

- `PREFIX-NNNN` where:
  - `PREFIX` ∈ `{INV, DM, AC, KC}`
  - `NNNN` is a 4-digit, zero-padded integer

Examples:
- `INV-0001`
- `DM-0042`
- `AC-0100`
- `KC-0003`

Titles are optional and may change without affecting the ID.

## Stability Rules

### Never renumber
Once assigned, an ID never changes.

### Never reuse
An ID is never reassigned to a different concept, even if deprecated.

### Deprecation and supersession
If an entry becomes obsolete:
- mark it as **Deprecated**
- optionally indicate a successor via **Superseded by <ID>**
- keep the original entry in place to prevent link rot

Recommended status values:
- `Active`
- `Deprecated`
- `Superseded by <ID>`

## Tags

Each Constitution entry SHOULD include `**Tags:**` as controlled metadata.

- Tags are used to generate navigation views and to improve discoverability.
- Tags MUST come from the allowlist in `docs/constitution/tag-taxonomy.md`.
- Tags are NOT identity and may change without breaking links.

Example:
- `**Tags:** determinism, replay`

## Linkability Requirements

### Why explicit anchors
GitHub’s automatic heading anchors are derived from heading text and can change when titles change.
Agents benefit from stable anchors because they can reliably:
- fetch the authoritative statement
- link to it from derived artifacts
- validate references

### Anchor convention
Every ID entry MUST define an explicit anchor matching the ID.

Use this pattern on the entry heading:

```md
### <a id="INV-0001"></a> INV-0001 — Deterministic simulation
```

This makes the stable link:

* `docs/constitution/invariants.md#INV-0001`

### Entry structure (recommended)

Each ID entry SHOULD include:

* **Status**
* **Tags**
* **Statement** (the authoritative clause)
* Optional: **Notes**, **Rationale**, **Examples**, **Non-examples**
* Optional: **Supersession** or **Related IDs**

Recommended template:

```md
### <a id="INV-0001"></a> INV-0001 — <Short title>
**Status:** Active | Deprecated | Superseded by INV-####
**Tags:** <comma-separated allowlisted tags>

**Statement:** <Single clear sentence defining the clause.>

**Notes:**
- <Optional constraints or interpretation guidance>

**Related:**
- <Optional list of related IDs>
```

## Referencing Policy

### Shorthand vs. real references

Shorthand forms like `(INV/DM/AC/KC)` are allowed only in instructional contexts.

They are not valid references.

### Normative references must be explicit

Any normative or traceability context MUST cite explicit IDs.

Valid examples:

* `Constraints: INV-0001, DM-0006`
* `Satisfies: AC-0104`
* `Kill criteria: KC-0002`
* `Depends on: DM-0017`

### Links in references (recommended)

Where feasible, references should link directly to the authoritative source.

Example:

* `Constraints: [INV-0001](./invariants.md#INV-0001)`

## Generated Indices and Catalog

The following files are derived artifacts generated from canonical Constitution docs:

* `docs/constitution/id-index.md` (sorted by ID, link-only)
* `docs/constitution/id-index-by-tag.md` (grouped by tag, link-only)
* `docs/constitution/id-catalog.json` (machine-readable catalog)

Generated artifacts are committed and must be regenerated via `just ids-gen` when canonical docs change.

## Amendment Process

The ID system (including tags allowlist and generators) is part of the Constitution’s operating rules.
Changes require:

* an ADR (recommended for non-trivial changes)
* updates to the relevant Constitution docs
* preservation of existing IDs, anchors, and links