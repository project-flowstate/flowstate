#!/usr/bin/env python
"""
Spec linter for FlowState delivery path.

Validates specs against required sections, Constitution ID references,
and gate plan requirements.

Usage:
    python scripts/spec_lint.py                    # Lint all specs
    python scripts/spec_lint.py docs/specs/FS-0001-example.md  # Lint specific spec
    python scripts/spec_lint.py --changed          # Lint only changed specs (git diff)

Exit codes:
    0 = success (may have warnings)
    1 = errors found (blocks CI)
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple

ROOT = Path(__file__).resolve().parents[1]
SPECS_DIR = ROOT / "docs" / "specs"
CATALOG_PATH = ROOT / "docs" / "constitution" / "id-catalog.json"

# Required sections (must exist and have content)
REQUIRED_SECTIONS = [
    "Problem",
    "Trace Map",
    "Domain Concepts",
    "Interfaces",
    "Determinism Notes",
    "Gate Plan",
    "Acceptance Criteria",
]

# Optional sections (presence checked but not required)
OPTIONAL_SECTIONS = [
    "Non-Goals",
    "Risks",
    "Alternatives",
]

# Frontmatter fields
REQUIRED_FRONTMATTER = ["status", "issue", "title"]

# Valid status values
VALID_STATUSES = {"Draft", "Approved", "Implemented"}

# ID patterns
ID_RE = re.compile(r"\b(?:INV|DM|AC|KC|ADR)-\d{4}\b")
NEW_CONCEPT_RE = re.compile(r"\bNEW:\s*(\w+)")

# Section heading pattern
SECTION_RE = re.compile(r"^##\s+(.+?)\s*$", re.MULTILINE)

# Frontmatter pattern
FRONTMATTER_RE = re.compile(r"^---\s*\n(.+?)\n---", re.DOTALL)

# Tier 0 section pattern (must have at least one bullet)
TIER0_RE = re.compile(r"###\s+Tier\s+0[^\n]*\n(.*?)(?=###|\Z)", re.DOTALL | re.IGNORECASE)
BULLET_RE = re.compile(r"^\s*[-*]\s+\[.\]", re.MULTILINE)


class LintResult(NamedTuple):
    errors: list[str]
    warnings: list[str]


def load_catalog() -> set[str]:
    """Load known IDs from the constitution catalog."""
    if not CATALOG_PATH.exists():
        return set()

    try:
        with open(CATALOG_PATH, encoding="utf-8") as f:
            catalog = json.load(f)
        return {entry["id"] for entry in catalog}
    except (json.JSONDecodeError, KeyError):
        return set()


def parse_frontmatter(content: str) -> dict[str, str]:
    """Extract YAML frontmatter from spec content."""
    match = FRONTMATTER_RE.match(content)
    if not match:
        return {}

    frontmatter = {}
    for line in match.group(1).splitlines():
        if ":" in line:
            key, value = line.split(":", 1)
            frontmatter[key.strip().lower()] = value.strip().strip('"').strip("'")
    return frontmatter


def find_sections(content: str) -> dict[str, str]:
    """Find all H2 sections and their content."""
    sections: dict[str, str] = {}
    matches = list(SECTION_RE.finditer(content))

    for i, match in enumerate(matches):
        section_name = match.group(1).strip()
        start = match.end()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(content)
        section_content = content[start:end].strip()
        sections[section_name] = section_content

    return sections


def lint_spec(path: Path, catalog_ids: set[str]) -> LintResult:
    """Lint a single spec file."""
    errors: list[str] = []
    warnings: list[str] = []

    try:
        content = path.read_text(encoding="utf-8")
    except Exception as e:
        return LintResult([f"Cannot read file: {e}"], [])

    # Get relative path for error messages (handle both absolute and relative paths)
    try:
        rel_path = path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        rel_path = str(path)

    # Check frontmatter
    frontmatter = parse_frontmatter(content)
    if not frontmatter:
        errors.append(f"{rel_path}: Missing YAML frontmatter")
    else:
        for field in REQUIRED_FRONTMATTER:
            if field not in frontmatter or not frontmatter[field]:
                errors.append(f"{rel_path}: Missing frontmatter field: {field}")

        # Validate status
        status = frontmatter.get("status", "")
        if status and status not in VALID_STATUSES:
            errors.append(
                f"{rel_path}: Invalid status '{status}' (must be one of: {', '.join(VALID_STATUSES)})"
            )

    # Find sections
    sections = find_sections(content)

    # Check required sections
    for section in REQUIRED_SECTIONS:
        if section not in sections:
            errors.append(f"{rel_path}: Missing required section: ## {section}")
        elif not sections[section].strip():
            errors.append(f"{rel_path}: Section '## {section}' is empty")

    # Check Tier 0 gate plan has at least one bullet
    tier0_match = TIER0_RE.search(content)
    if tier0_match:
        tier0_content = tier0_match.group(1)
        if not BULLET_RE.search(tier0_content):
            errors.append(f"{rel_path}: Tier 0 gate plan must have at least one bullet item")
    elif "Gate Plan" in sections:
        # Gate Plan exists but no Tier 0 subsection
        errors.append(f"{rel_path}: Gate Plan must include '### Tier 0' subsection")

    # Validate referenced IDs exist in catalog
    if catalog_ids:
        referenced_ids = set(ID_RE.findall(content))
        for ref_id in sorted(referenced_ids):
            if ref_id not in catalog_ids:
                errors.append(f"{rel_path}: References unknown ID: {ref_id}")

    # Check NEW: concepts based on status
    status = frontmatter.get("status", "Draft")
    new_concepts = NEW_CONCEPT_RE.findall(content)
    if new_concepts:
        if status in {"Approved", "Implemented"}:
            for concept in new_concepts:
                errors.append(
                    f"{rel_path}: Status is '{status}' but contains unresolved 'NEW: {concept}' "
                    f"(must resolve to DM-* before approval)"
                )
        else:
            for concept in new_concepts:
                warnings.append(f"{rel_path}: Draft spec contains 'NEW: {concept}' (resolve before approval)")

    return LintResult(errors, warnings)


def get_changed_specs() -> list[Path]:
    """Get list of specs changed in current git diff."""
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", "HEAD"],
            capture_output=True,
            text=True,
            cwd=ROOT,
            check=True,
        )
        changed = result.stdout.strip().splitlines()

        # Also check staged files
        result_staged = subprocess.run(
            ["git", "diff", "--name-only", "--cached"],
            capture_output=True,
            text=True,
            cwd=ROOT,
            check=True,
        )
        changed.extend(result_staged.stdout.strip().splitlines())

        specs = []
        for file in set(changed):
            if file.startswith("docs/specs/") and file.endswith(".md"):
                path = ROOT / file
                if path.exists() and path.name != "_TEMPLATE.md":
                    specs.append(path)
        return specs
    except subprocess.CalledProcessError:
        return []


def get_all_specs() -> list[Path]:
    """Get all spec files."""
    if not SPECS_DIR.exists():
        return []
    return [
        p for p in SPECS_DIR.glob("*.md")
        if p.name != "_TEMPLATE.md" and p.name != "README.md"
    ]


def main() -> None:
    parser = argparse.ArgumentParser(description="Lint spec files")
    parser.add_argument("files", nargs="*", help="Specific files to lint")
    parser.add_argument("--changed", action="store_true", help="Only lint changed files")
    parser.add_argument("--warnings-as-errors", action="store_true", help="Treat warnings as errors")
    args = parser.parse_args()

    # Determine which files to lint
    if args.files:
        specs = [Path(f) for f in args.files]
    elif args.changed:
        specs = get_changed_specs()
        if not specs:
            print("No changed specs found.")
            sys.exit(0)
    else:
        specs = get_all_specs()

    if not specs:
        print("No specs found to lint.")
        sys.exit(0)

    # Load catalog
    catalog_ids = load_catalog()
    if not catalog_ids:
        print("WARNING: Could not load ID catalog; skipping ID validation", file=sys.stderr)

    # Lint each spec
    all_errors: list[str] = []
    all_warnings: list[str] = []

    for spec in sorted(specs):
        result = lint_spec(spec, catalog_ids)
        all_errors.extend(result.errors)
        all_warnings.extend(result.warnings)

    # Report results
    if all_warnings:
        print("WARNINGS:", file=sys.stderr)
        for w in all_warnings:
            print(f"  - {w}", file=sys.stderr)
        print()

    if all_errors:
        print("ERRORS:", file=sys.stderr)
        for e in all_errors:
            print(f"  - {e}", file=sys.stderr)
        print()
        print(f"Spec lint failed: {len(all_errors)} error(s), {len(all_warnings)} warning(s)")
        sys.exit(1)

    if args.warnings_as_errors and all_warnings:
        print(f"Spec lint failed (warnings as errors): {len(all_warnings)} warning(s)")
        sys.exit(1)

    print(f"Spec lint passed: {len(specs)} spec(s) checked, {len(all_warnings)} warning(s)")
    sys.exit(0)


if __name__ == "__main__":
    main()
