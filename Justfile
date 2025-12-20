set shell := ["sh", "-eu", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# Cross-platform Python (Windows has 'python', Linux/macOS have 'python3')
python := if os() == "windows" { "python" } else { "python3" }

default: help
help:
	@just --list

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace --all-targets

check-licenses:
	@echo "NOTE: License scanning not yet automated (planned: cargo-deny)."
	@echo "Manual review required. See: docs/licensing/third-party.md"

ids:
	{{python}} scripts/constitution_ids.py check

ids-gen:
	{{python}} scripts/constitution_ids.py generate --write

trace: ids

# Lint all specs
spec-lint:
	{{python}} scripts/spec_lint.py

# Lint a single spec file (for development iteration)
# Usage: just spec-lint-file docs/specs/FS-0001-foo.md
spec-lint-file file:
	{{python}} scripts/spec_lint.py {{file}}

# Lint only changed specs (for PR CI)
spec-lint-changed:
	{{python}} scripts/spec_lint.py --changed

# Validate PR trace block (reads from stdin)
# Usage: cat pr_body.txt | just pr-trace
pr-trace:
	{{python}} scripts/pr_trace.py

# Validate PR trace block from file
# Usage: just pr-trace-file path/to/pr_body.txt
pr-trace-file file:
	{{python}} scripts/pr_trace.py --file {{file}}

# Full CI check (used on main branch and for local validation)
ci: fmt lint test ids spec-lint

# PR-specific CI check (uses changed-only spec lint)
ci-pr: fmt lint test ids spec-lint-changed

check: ci
verify: ci
