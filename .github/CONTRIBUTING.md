# Contributing

Thanks for your interest in contributing to this project.

Contributions of all kinds are welcome, including bug fixes, improvements, refactors, tests, and documentation.

## Authority / Source of Truth

The canonical authority and precedence rules for this repository are defined in one place:
- `docs/constitution.md#authority-map`

If you are unsure where something belongs or which document wins in a conflict, consult the Authority Map and link to it rather than duplicating it elsewhere.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Run the repository validation surface locally:

```
just ci
```

## Constitution IDs and generated artifacts

This repository uses Constitution IDs (`INV-####`, `DM-####`, `AC-####`, `KC-####`) to keep traceability stable across ADRs, specs, and code.

If you edit canonical Constitution documents under `docs/constitution/`:

1. Validate ID/tag/reference integrity:

```
just ids
```

2. Regenerate committed navigation artifacts:

```
just ids-gen
```

Generated artifacts are committed and MUST be regenerated via `just ids-gen` whenever canonical Constitution documents change.

## Workflow

- Create a feature branch from `main`
- Make focused, incremental changes
- Open a Pull Request referencing a related Issue where applicable

## Pull Requests

- Keep PRs small and scoped
- Ensure all CI checks pass
- Add or update tests when appropriate
- Clearly describe what changed and why
- For non-trivial changes, reference relevant Constitution IDs and/or ADRs

## Issues

- Use the provided issue templates
- For substantial or architectural changes, open an Issue before starting work

## Code Style and Quality

- Code is automatically formatted, linted, and tested
- CI is authoritative and must pass before a PR can be merged

## Code of Conduct

This project follows the Contributor Covenant.
All participants are expected to uphold this standard.
