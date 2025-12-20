# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Versioning Policy

- **`0.0.0`** (current): Foundation phase, no releases
- **`0.x.y`**: Pre-1.0 development; breaking changes permitted between minors
- **`1.0.0+`**: Stable API; strict SemVer (MAJOR.MINOR.PATCH)

Versions are updated manually when cutting releases (milestone-based, not per-PR).
Changes accumulate in `[Unreleased]` until a version is tagged.

## [Unreleased]

### Added

**Governance & Constitution:**
- Constitution with authority map, product thesis, and derivation contract (`docs/constitution.md`)
- Constitution annexes: invariants (`INV-0001`, `INV-0002`), domain model (`DM-0001..0004`), acceptance/kill criteria, tag taxonomy, ID system, scope/non-goals
- Constitution ID system with automated validation (`just ids`) and generation (`just ids-gen`)
- ADR template for future architecture decisions (`docs/adr/0000-adr-template.md`)
- Golden Delivery Path protocol with governance classification and trace blocks (`docs/delivery-path.md`)

**Infrastructure & Automation:**
- Justfile with `ci`, `fmt`, `lint`, `test`, `ids`, `spec-lint`, `pr-trace` targets
- GitHub Actions CI workflow with Rust caching, Python setup, and PR trace validation
- Python validation scripts: Constitution IDs, spec linting, PR trace parsing
- Dependabot configuration for Rust and GitHub Actions

**GitHub Governance:**
- Issue templates: feature requests, governance changes, bug reports, agent tasks
- PR template with required trace block
- Contributing guide, Code of Conduct, Security policy, CODEOWNERS

**Documentation:**
- Game vision and design intent (`docs/vision.md`)
- Human handbook and operating model (`docs/handbook.md`)
- Repository routing table (`docs/repo-map.md`)
- Agent operating rules (`AGENTS.md`)
- Licensing policy and third-party intake process (`docs/licensing/third-party.md`)
- Spec template and structure (`docs/specs/`)

**Code:**
- Minimal simulation crate (`crates/sim`) with smoke test
- Rust toolchain pinned to 1.92.0 with edition 2024
- Workspace structure with placeholders for client and protocol crates

### Changed

- (none yet)

### Deprecated

- (none yet)

### Removed

- (none yet)

### Fixed

- (none yet)

### Security

- (none yet)
