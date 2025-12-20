#!/usr/bin/env python
from __future__ import annotations

import argparse
import dataclasses
import json
import re
import sys
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
CONSTITUTION_DIR = ROOT / "docs" / "constitution"
ADR_DIR = ROOT / "docs" / "adr"

# Canonical sources (authoritative content lives here)
CANONICAL_DOCS = [
    CONSTITUTION_DIR / "invariants.md",
    CONSTITUTION_DIR / "domain-model.md",
    CONSTITUTION_DIR / "acceptance-kill.md",
]

# Non-canonical-but-authoritative rules docs
TAG_TAXONOMY = CONSTITUTION_DIR / "tag-taxonomy.md"

# Derived artifacts (generated, committed, never edited by hand)
GENERATED_MD = [
    CONSTITUTION_DIR / "id-index.md",
    CONSTITUTION_DIR / "id-index-by-tag.md",
]
GENERATED_JSON = [
    CONSTITUTION_DIR / "id-catalog.json",
]

ID_RE = re.compile(r"\b(?:INV|DM|AC|KC|ADR)-\d{4}\b")
ANCHOR_RE = re.compile(
    r'<a\s+id="(?P<id>(?:INV|DM|AC|KC)-\d{4})"\s*>\s*</a>',
    re.IGNORECASE,
)

# ADR filename pattern: NNNN-slug.md
ADR_FILENAME_RE = re.compile(r"^(?P<num>\d{4})-(?P<slug>.+)\.md$")

# We intentionally ignore code fences to reduce false positives in examples/templates.
FENCE_RE = re.compile(r"```.*?```", re.DOTALL)

STATUS_RE = re.compile(r"^\*\*Status:\*\*\s*(?P<status>.+?)\s*$", re.IGNORECASE)
TAGS_RE = re.compile(r"^\*\*Tags:\*\*\s*(?P<tags>.+?)\s*$", re.IGNORECASE)

HEADING_RE = re.compile(r"^(?P<level>#{1,6})\s+(?P<text>.+?)\s*$")


@dataclasses.dataclass(frozen=True)
class Entry:
    id: str
    title: str
    status: str
    tags: tuple[str, ...]
    file: str  # repo-relative
    anchor: str
    href: str  # repo-relative
    section: str  # nearest H2 heading (or "")

    @property
    def prefix(self) -> str:
        return self.id.split("-")[0]

    @property
    def number(self) -> int:
        return int(self.id.split("-")[1])


@dataclasses.dataclass(frozen=True)
class ADREntry:
    """Represents an Architecture Decision Record."""
    id: str  # ADR-NNNN format
    title: str
    status: str
    file: str  # repo-relative
    href: str  # repo-relative

    @property
    def prefix(self) -> str:
        return "ADR"

    @property
    def number(self) -> int:
        return int(self.id.split("-")[1])


class ErrorCollector:
    def __init__(self) -> None:
        self.errors: list[str] = []

    def add(self, msg: str) -> None:
        self.errors.append(msg)

    def has_any(self) -> bool:
        return bool(self.errors)

    def raise_if_any(self) -> None:
        if not self.errors:
            return
        print("ERROR: Constitution ID validation failed with the following issues:", file=sys.stderr)
        for msg in self.errors:
            print(f"- {msg}", file=sys.stderr)
        sys.exit(1)


def die(msg: str) -> None:
    # For true fatal errors where continuing is meaningless (missing files, unreadable taxonomy, etc.)
    print(f"ERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def strip_fences(md: str) -> str:
    return re.sub(FENCE_RE, "", md)


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def parse_allowed_tags(md: str) -> set[str]:
    # Look for the "## Allowed Tags" section and collect bullets underneath until next H2.
    lines = md.splitlines()
    allowed: set[str] = set()
    in_allowed = False
    for line in lines:
        m = HEADING_RE.match(line)
        if m and m.group("level") == "##":
            in_allowed = (m.group("text").strip().lower() == "allowed tags")
            continue
        if in_allowed:
            # stop at next H2
            if m and m.group("level") == "##":
                break
            s = line.strip()
            if s.startswith("- "):
                tag = s[2:].strip()
                # Extract tag name before colon (e.g., "determinism: description" -> "determinism")
                if ":" in tag:
                    tag = tag.split(":", 1)[0].strip()
                if tag:
                    allowed.add(tag)
    return allowed


def parse_entries_from_doc(
    path: Path,
    allowed_tags: set[str] | None,
    ec: ErrorCollector,
) -> list[Entry]:
    md = strip_fences(read_text(path))
    lines = md.splitlines()

    entries: list[Entry] = []
    section = ""
    seen_anchor_lines: dict[str, int] = {}

    i = 0
    while i < len(lines):
        line = lines[i]
        h = HEADING_RE.match(line)
        if h:
            level = len(h.group("level"))
            text = h.group("text")

            # Track H2 as current "section"
            if level == 2:
                section = text.strip()

            # Detect anchored ID headings at any level (we recommend H3)
            a = ANCHOR_RE.search(text)
            if a:
                cid = a.group("id")

                # Duplicate anchors within a file
                if cid in seen_anchor_lines:
                    ec.add(f"{rel(path)}: duplicate ID anchor found for {cid}")
                else:
                    seen_anchor_lines[cid] = i + 1

                # Title is whatever comes after "ID —"
                cleaned = ANCHOR_RE.sub("", text).strip()
                cleaned = re.sub(rf"^\s*{re.escape(cid)}\s*", "", cleaned).strip()
                cleaned = re.sub(r"^[—\-]\s*", "", cleaned).strip()
                title = cleaned or "<Untitled>"

                # Expect Status and Tags within the next ~12 lines (tolerant)
                status = ""
                tags: tuple[str, ...] = tuple()
                lookahead = 0
                j = i + 1
                while j < len(lines) and lookahead < 12:
                    s_line = lines[j].strip()
                    if s_line.startswith("#"):
                        break

                    sm = STATUS_RE.match(s_line)
                    if sm:
                        status = sm.group("status").strip()

                    tm = TAGS_RE.match(s_line)
                    if tm:
                        raw = tm.group("tags").strip()
                        if raw.lower() in {"none", "(none)", ""}:
                            tags = tuple()
                        else:
                            parsed = [t.strip() for t in raw.split(",") if t.strip()]
                            tags = tuple(parsed)

                    j += 1
                    lookahead += 1

                if not status:
                    ec.add(f"{rel(path)}: {cid} is missing **Status:**")

                # Tag allowlist checks (batch errors)
                if allowed_tags is not None:
                    for t in tags:
                        if t not in allowed_tags:
                            ec.add(
                                f"{rel(path)}: {cid} uses unknown tag '{t}' "
                                f"(not in tag-taxonomy.md allowlist)"
                            )

                href = f"{rel(path)}#{cid}"
                entries.append(
                    Entry(
                        id=cid,
                        title=title,
                        status=status,
                        tags=tags,
                        file=rel(path),
                        anchor=cid,
                        href=href,
                        section=section,
                    )
                )
        i += 1

    return entries


def collect_all_entries(ec: ErrorCollector) -> tuple[list[Entry], set[str]]:
    allowed_tags: set[str] | None = None
    if TAG_TAXONOMY.exists():
        allowed_tags = parse_allowed_tags(read_text(TAG_TAXONOMY))
        if not allowed_tags:
            die(f"{rel(TAG_TAXONOMY)}: could not find any allowlisted tags under '## Allowed Tags'")

    entries: list[Entry] = []
    for doc in CANONICAL_DOCS:
        if not doc.exists():
            die(f"missing canonical doc: {rel(doc)}")
        entries.extend(parse_entries_from_doc(doc, allowed_tags, ec))

    # Cross-file duplicate IDs
    dupes: dict[str, list[str]] = {}
    for e in entries:
        dupes.setdefault(e.id, []).append(e.file)
    collisions = {k: v for k, v in dupes.items() if len(v) > 1}
    for cid, files in sorted(collisions.items()):
        ec.add(f"duplicate ID across files: {cid} appears in {files}")

    ids = {e.id for e in entries}
    return entries, ids


def collect_adr_entries(ec: ErrorCollector) -> list[ADREntry]:
    """Collect ADR entries from docs/adr/*.md files."""
    adrs: list[ADREntry] = []

    if not ADR_DIR.exists():
        return adrs

    for adr_file in sorted(ADR_DIR.glob("*.md")):
        m = ADR_FILENAME_RE.match(adr_file.name)
        if not m:
            # Skip non-ADR files (like templates)
            continue

        num = int(m.group("num"))
        adr_id = f"ADR-{num:04d}"

        # Parse the ADR file to extract title and status
        md = read_text(adr_file)
        lines = md.splitlines()

        title = ""
        status = ""

        for line in lines:
            # Look for H1 title
            if line.startswith("# ") and not title:
                title = line[2:].strip()
                # Remove ADR number prefix if present (e.g., "# ADR-0001: Title" -> "Title")
                title = re.sub(r"^ADR-\d+[:\s—-]*", "", title, flags=re.IGNORECASE).strip()
                continue

            # Look for status in frontmatter or body
            sm = STATUS_RE.match(line.strip())
            if sm:
                status = sm.group("status").strip()
                continue

            # Also check for YAML frontmatter status
            if line.strip().startswith("status:"):
                status = line.split(":", 1)[1].strip().strip('"').strip("'")
                continue

        if not title:
            title = m.group("slug").replace("-", " ").title()

        if not status:
            status = "Unknown"

        adrs.append(
            ADREntry(
                id=adr_id,
                title=title,
                status=status,
                file=rel(adr_file),
                href=rel(adr_file),
            )
        )

    return adrs


def collect_id_references_in_docs(docs: Iterable[Path]) -> dict[str, set[str]]:
    refs: dict[str, set[str]] = {}
    for doc in docs:
        if not doc.exists():
            # Skip quietly; callers may include optional docs
            continue
        md = strip_fences(read_text(doc))
        found = set(ID_RE.findall(md))
        refs[rel(doc)] = found
    return refs


def render_id_index(entries: list[Entry], adr_entries: list[ADREntry] | None = None) -> str:
    by_prefix: dict[str, list[Entry]] = {}
    for e in entries:
        by_prefix.setdefault(e.prefix, []).append(e)
    for k in by_prefix:
        by_prefix[k].sort(key=lambda x: x.number)

    def block(title: str, prefix: str) -> str:
        lines = [f"## {title}", ""]
        for e in by_prefix.get(prefix, []):
            rel_link = "./" + Path(e.file).name
            lines.append(f"- [{e.id}]({rel_link}#{e.id}) — {e.title}")
        lines.append("")
        return "\n".join(lines)

    def adr_block(adr_list: list[ADREntry]) -> str:
        lines = ["## Architecture Decision Records", ""]
        for a in sorted(adr_list, key=lambda x: x.number):
            rel_link = "../adr/" + Path(a.file).name
            lines.append(f"- [{a.id}]({rel_link}) — {a.title}")
        lines.append("")
        return "\n".join(lines)

    out = [
        "# Constitution ID Index",
        "",
        "NOTE: This file is GENERATED. Do not edit manually.",
        "Generator: `python scripts/constitution_ids.py generate --write`",
        "",
        "This index is link-only. All substance lives in the canonical Constitution documents.",
        "",
        block("Invariants", "INV"),
        block("Domain Model", "DM"),
        block("Acceptance Criteria", "AC"),
        block("Kill Criteria", "KC"),
    ]

    if adr_entries:
        out.append(adr_block(adr_entries))

    return "\n".join(out).rstrip() + "\n"


def render_id_index_by_tag(entries: list[Entry]) -> str:
    tag_map: dict[str, list[Entry]] = {}
    for e in entries:
        for t in e.tags:
            tag_map.setdefault(t, []).append(e)

    for t in tag_map:
        tag_map[t].sort(key=lambda x: (x.prefix, x.number))

    out_lines = [
        "# Constitution ID Index by Tag",
        "",
        "NOTE: This file is GENERATED. Do not edit manually.",
        "Generator: `python scripts/constitution_ids.py generate --write`",
        "",
        "This index is link-only. All substance lives in the canonical Constitution documents.",
        "",
    ]

    for tag in sorted(tag_map.keys()):
        out_lines.append(f"## {tag}")
        out_lines.append("")
        for e in tag_map[tag]:
            rel_link = "./" + Path(e.file).name
            out_lines.append(f"- [{e.id}]({rel_link}#{e.id}) — {e.title}")
        out_lines.append("")

    return "\n".join(out_lines).rstrip() + "\n"


def render_catalog(entries: list[Entry], adr_entries: list[ADREntry] | None = None) -> str:
    payload = []
    for e in sorted(entries, key=lambda x: (x.prefix, x.number)):
        payload.append(
            {
                "id": e.id,
                "prefix": e.prefix,
                "number": e.number,
                "title": e.title,
                "status": e.status,
                "tags": list(e.tags),
                "file": e.file,
                "anchor": e.anchor,
                "href": e.href,
                "section": e.section,
            }
        )

    # Add ADR entries
    if adr_entries:
        for a in sorted(adr_entries, key=lambda x: x.number):
            payload.append(
                {
                    "id": a.id,
                    "prefix": a.prefix,
                    "number": a.number,
                    "title": a.title,
                    "status": a.status,
                    "tags": [],
                    "file": a.file,
                    "anchor": "",
                    "href": a.href,
                    "section": "",
                }
            )

    return json.dumps(payload, indent=2, ensure_ascii=False) + "\n"


def write_if_changed(path: Path, content: str) -> bool:
    existing = path.read_text(encoding="utf-8") if path.exists() else ""
    if existing == content:
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    # Use newline='\n' to ensure LF line endings on all platforms
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(content)
    return True


def check_generated(updates: dict[Path, str], ec: ErrorCollector) -> None:
    changed = []
    for path, expected in updates.items():
        current = path.read_text(encoding="utf-8") if path.exists() else ""
        if current != expected:
            changed.append(rel(path))
    if changed:
        ec.add(
            "generated files are out of date:\n"
            + "\n".join([f"  - {p}" for p in changed])
            + "\nRun: python scripts/constitution_ids.py generate --write"
        )


def main() -> None:
    ap = argparse.ArgumentParser(description="Flowstate Constitution ID tooling")
    sub = ap.add_subparsers(dest="cmd", required=True)

    gen = sub.add_parser("generate", help="Generate derived indices/catalog")
    gen.add_argument("--write", action="store_true", help="Write files in-place (otherwise print)")

    chk = sub.add_parser("check", help="Validate IDs, tags, and generated artifacts")
    chk.add_argument("--no-generated-check", action="store_true", help="Skip checking generated outputs")

    args = ap.parse_args()

    ec = ErrorCollector()
    entries, ids = collect_all_entries(ec)
    adr_entries = collect_adr_entries(ec)

    # Add ADR IDs to the known set for reference validation
    adr_ids = {a.id for a in adr_entries}
    all_ids = ids | adr_ids

    # Validate that references across canonical docs refer to known IDs.
    # Note: constitution.md is optional in early scaffolding; if present, validate it too.
    refs = collect_id_references_in_docs(CANONICAL_DOCS + [ROOT / "docs" / "constitution.md"])
    for file_rel, found in refs.items():
        for cid in sorted(found):
            if cid not in all_ids:
                ec.add(f"{file_rel}: references unknown ID {cid}")

    # Prepare generated outputs
    outputs: dict[Path, str] = {
        CONSTITUTION_DIR / "id-index.md": render_id_index(entries, adr_entries),
        CONSTITUTION_DIR / "id-index-by-tag.md": render_id_index_by_tag(entries),
        CONSTITUTION_DIR / "id-catalog.json": render_catalog(entries, adr_entries),
    }

    if args.cmd == "generate":
        if args.write:
            any_changed = False
            for path, content in outputs.items():
                any_changed = write_if_changed(path, content) or any_changed
            if any_changed:
                print("Generated files updated.")
            else:
                print("Generated files already up to date.")
        else:
            for path, content in outputs.items():
                print(f"\n# ===== {rel(path)} =====\n")
                print(content, end="")

        # Even in generate mode, still report structural errors found while parsing.
        ec.raise_if_any()
        return

    if args.cmd == "check":
        if not args.no_generated_check:
            check_generated(outputs, ec)

        ec.raise_if_any()
        print("OK: IDs, tags, references, and generated artifacts are valid.")
        return


if __name__ == "__main__":
    main()
