#!/usr/bin/env python
"""
PR trace block validator for FlowState delivery path.

Validates the sentinel trace block in PR descriptions, ensuring:
- Trace block is present and parseable
- Referenced spec exists
- Bidirectional link: spec's issue matches PR's issue
- All Constitution/ADR IDs are valid

Usage:
    python scripts/pr_trace.py < pr_body.txt
    python scripts/pr_trace.py --file pr_body.txt
    echo "$PR_BODY" | python scripts/pr_trace.py

Exit codes:
    0 = success
    1 = validation errors
"""
from __future__ import annotations

import argparse
import glob
import json
import re
import sys
from pathlib import Path
from typing import NamedTuple

ROOT = Path(__file__).resolve().parents[1]
SPECS_DIR = ROOT / "docs" / "specs"
CATALOG_PATH = ROOT / "docs" / "constitution" / "id-catalog.json"

# Trace block pattern: ```trace ... ```
TRACE_BLOCK_RE = re.compile(r"```trace\s*\n(.+?)\n```", re.DOTALL)

# Field patterns within trace block
ISSUE_RE = re.compile(r"^Issue:\s*#?(\d+)\s*$", re.MULTILINE)
SPEC_RE = re.compile(r"^Spec:\s*(.+?)\s*$", re.MULTILINE)
CONSTITUTION_RE = re.compile(r"^Constitution:\s*(.+?)\s*$", re.MULTILINE)
ADRS_RE = re.compile(r"^ADRs:\s*(.+?)\s*$", re.MULTILINE)

# ID pattern
ID_RE = re.compile(r"\b(?:INV|DM|AC|KC|ADR)-\d{4}\b")

# Spec frontmatter issue pattern
SPEC_ISSUE_RE = re.compile(r"^issue:\s*#?(\d+)\s*$", re.MULTILINE | re.IGNORECASE)

# Trivial escape hatch
TRIVIAL_PATTERN = re.compile(r"^N/A:\s*trivial", re.IGNORECASE)


class TraceBlock(NamedTuple):
    issue: str | None
    spec: str | None
    constitution: list[str]
    adrs: list[str]
    raw: str


class ValidationResult(NamedTuple):
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


def parse_trace_block(pr_body: str) -> TraceBlock | None:
    """Extract and parse the trace block from PR body."""
    match = TRACE_BLOCK_RE.search(pr_body)
    if not match:
        return None

    raw = match.group(1)

    # Extract issue
    issue_match = ISSUE_RE.search(raw)
    issue = issue_match.group(1) if issue_match else None

    # Extract spec
    spec_match = SPEC_RE.search(raw)
    spec = spec_match.group(1).strip() if spec_match else None

    # Extract constitution IDs
    constitution: list[str] = []
    const_match = CONSTITUTION_RE.search(raw)
    if const_match:
        const_value = const_match.group(1).strip()
        if const_value:
            constitution = [id_.strip() for id_ in ID_RE.findall(const_value)]

    # Extract ADR IDs
    adrs: list[str] = []
    adrs_match = ADRS_RE.search(raw)
    if adrs_match:
        adrs_value = adrs_match.group(1).strip()
        if adrs_value.lower() != "none" and adrs_value:
            adrs = [id_.strip() for id_ in ID_RE.findall(adrs_value)]

    return TraceBlock(issue=issue, spec=spec, constitution=constitution, adrs=adrs, raw=raw)


def find_spec_for_issue(issue_num: str) -> Path | None:
    """Find a spec file for the given issue number."""
    # Try exact match first: FS-NNNN-*.md
    padded = issue_num.zfill(4)
    patterns = [
        f"FS-{padded}-*.md",
        f"FS-{issue_num}-*.md",
    ]

    for pattern in patterns:
        matches = list(SPECS_DIR.glob(pattern))
        if matches:
            return matches[0]

    return None


def get_spec_issue(spec_path: Path) -> str | None:
    """Extract the issue number from a spec's frontmatter."""
    try:
        content = spec_path.read_text(encoding="utf-8")
    except Exception:
        return None

    match = SPEC_ISSUE_RE.search(content[:1000])  # Only search frontmatter area
    if match:
        return match.group(1)
    return None


def validate_trace_block(pr_body: str, catalog_ids: set[str]) -> ValidationResult:
    """Validate the trace block in PR body."""
    errors: list[str] = []
    warnings: list[str] = []

    # Check trace block exists
    trace = parse_trace_block(pr_body)
    if trace is None:
        errors.append("Missing trace block. PR must contain ```trace ... ``` sentinel block.")
        return ValidationResult(errors, warnings)

    # Check required fields
    if not trace.issue:
        errors.append("Trace block missing 'Issue: #NNNN' field")

    if not trace.spec:
        errors.append("Trace block missing 'Spec:' field")

    # If we have both issue and spec, do validation
    if trace.issue and trace.spec:
        # Check for trivial escape hatch
        if TRIVIAL_PATTERN.match(trace.spec):
            # Trivial change - check if there's actually a spec for this issue
            existing_spec = find_spec_for_issue(trace.issue)
            if existing_spec:
                warnings.append(
                    f"Spec marked as 'N/A: trivial' but spec exists for issue #{trace.issue}: "
                    f"{existing_spec.relative_to(ROOT).as_posix()}"
                )
        else:
            # Non-trivial - validate spec exists and links match
            spec_path = ROOT / trace.spec
            if not spec_path.exists():
                errors.append(f"Spec file does not exist: {trace.spec}")
            else:
                # Bidirectional check: spec's issue must match PR's issue
                spec_issue = get_spec_issue(spec_path)
                if spec_issue is None:
                    errors.append(f"Spec {trace.spec} is missing 'issue:' in frontmatter")
                elif spec_issue != trace.issue:
                    errors.append(
                        f"Issue mismatch: PR declares Issue: #{trace.issue}, "
                        f"but spec {trace.spec} declares issue: {spec_issue}"
                    )

            # Check if there's a spec for this issue that wasn't referenced
            expected_spec = find_spec_for_issue(trace.issue)
            if expected_spec:
                expected_path = expected_spec.relative_to(ROOT).as_posix()
                declared_path = trace.spec.replace("\\", "/")
                if expected_path != declared_path:
                    warnings.append(
                        f"Spec for issue #{trace.issue} exists at {expected_path}, "
                        f"but PR references {trace.spec}"
                    )

    # If issue exists but spec is missing/trivial, check if spec exists
    if trace.issue and (not trace.spec or TRIVIAL_PATTERN.match(trace.spec or "")):
        existing_spec = find_spec_for_issue(trace.issue)
        if existing_spec and not (trace.spec and TRIVIAL_PATTERN.match(trace.spec)):
            errors.append(
                f"Spec exists for issue #{trace.issue} at "
                f"{existing_spec.relative_to(ROOT).as_posix()}, but Spec field is missing"
            )

    # Validate Constitution IDs
    if catalog_ids:
        for cid in trace.constitution:
            if cid not in catalog_ids:
                errors.append(f"Unknown Constitution ID: {cid}")

        for adr_id in trace.adrs:
            if adr_id not in catalog_ids:
                errors.append(f"Unknown ADR ID: {adr_id}")

    return ValidationResult(errors, warnings)


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate PR trace block")
    parser.add_argument("--file", "-f", help="Read PR body from file instead of stdin")
    parser.add_argument("--warnings-as-errors", action="store_true", help="Treat warnings as errors")
    args = parser.parse_args()

    # Read PR body
    if args.file:
        try:
            with open(args.file, encoding="utf-8") as f:
                pr_body = f.read()
        except Exception as e:
            print(f"ERROR: Cannot read file: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        pr_body = sys.stdin.read()

    if not pr_body.strip():
        print("ERROR: Empty PR body", file=sys.stderr)
        sys.exit(1)

    # Load catalog
    catalog_ids = load_catalog()
    if not catalog_ids:
        print("WARNING: Could not load ID catalog; skipping ID validation", file=sys.stderr)

    # Validate
    result = validate_trace_block(pr_body, catalog_ids)

    # Report results
    if result.warnings:
        print("WARNINGS:", file=sys.stderr)
        for w in result.warnings:
            print(f"  - {w}", file=sys.stderr)
        print()

    if result.errors:
        print("ERRORS:", file=sys.stderr)
        for e in result.errors:
            print(f"  - {e}", file=sys.stderr)
        print()
        print(f"PR trace validation failed: {len(result.errors)} error(s)")
        sys.exit(1)

    if args.warnings_as_errors and result.warnings:
        print(f"PR trace validation failed (warnings as errors): {len(result.warnings)} warning(s)")
        sys.exit(1)

    print("PR trace validation passed")
    sys.exit(0)


if __name__ == "__main__":
    main()
