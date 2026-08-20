#!/usr/bin/env python3
"""Audit literature citations in the source tree against src/citations.rs.

Why: 36 files carried `2026 SOTA` banners naming author-year tokens, mostly
without a DOI. Spot checks found one verified-but-misquoted value and several
tokens that could not be located in any bibliographic database. This script
makes the gap visible and keeps it visible.

A line carrying the marker `citation-audit: mention-only` is skipped, for prose
that names a removed citation in order to explain why it was removed.

Exit codes:
    0  report produced (default: informational)
    1  a gate was requested and failed

Gates:
    --strict              fail if any UNVERIFIED token is still referenced
    --max-unregistered N  fail if more than N tokens are absent from the registry

The unregistered backlog is large and is not treated as a failure by default:
each entry needs a database lookup before it can be classified. Use
--max-unregistered to ratchet it down as entries are verified.

Usage:
    python3 scripts/audit_citations.py
    python3 scripts/audit_citations.py --strict
    python3 scripts/audit_citations.py --max-unregistered 438
"""
from __future__ import annotations

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
REGISTRY = REPO / "src" / "citations.rs"

# Lines with this marker are prose about a citation, not a claim resting on it.
MENTION_ONLY_MARKER = "citation-audit: mention-only"

# "Author 2026", "Author et al. 2026". Deliberately narrow: capitalised surname
# followed by a 4-digit year, optionally with "et al.".
TOKEN_RE = re.compile(r"\b([A-Z][a-zA-Z\-']+)(?:\s+et\s+al\.?)?\s+((?:19|20)\d{2})\b")

# Tokens matching the pattern that are not literature citations.
NOT_CITATIONS = {
    "Copernicus", "Sentinel", "Landsat", "MODIS", "VIIRS", "GPM", "SRTM",
    "Otsu", "Manning", "Darcy", "Theis", "Nash", "Kappa", "Rust", "Python",
    "Indonesia", "Jakarta", "Semarang", "Bima", "Kendari", "PP", "SNI", "ISO",
    "EPA", "KLHK", "BMKG", "BIG", "BPS", "IPCC", "WHO", "UU", "Permen",
    "PermenLHK", "Jakstranas", "January", "February", "March", "April", "May",
    "June", "July", "August", "September", "October", "November", "December",
    "Table", "Figure", "Eq", "Version", "Rev", "Since", "Per", "Note", "Ref",
    "Source", "Data", "Model", "Method", "Phase", "Step", "Grid", "Zone",
}

SEARCH_DIRS = ["src", "scripts"]
SEARCH_SUFFIXES = {".rs", ".py"}
SKIP_PARTS = {"target", ".venv", "__pycache__", "node_modules", ".git"}


def parse_registry(text: str) -> tuple[set[str], set[str]]:
    """Extract VERIFIED and UNVERIFIED tokens from citations.rs."""

    def block(name: str) -> str:
        # Match `pub const NAME: &[...] = &[ ... ];`
        m = re.search(
            rf"pub const {name}:\s*&\[[^\]]*\]\s*=\s*&\[(.*?)\n\];",
            text,
            re.DOTALL,
        )
        return m.group(1) if m else ""

    def tokens(chunk: str) -> set[str]:
        return set(re.findall(r'token:\s*"([^"]+)"', chunk))

    return tokens(block("VERIFIED")), tokens(block("UNVERIFIED"))


def iter_source_files():
    for d in SEARCH_DIRS:
        root = REPO / d
        if not root.exists():
            continue
        for p in root.rglob("*"):
            if p.suffix not in SEARCH_SUFFIXES:
                continue
            if SKIP_PARTS & set(p.parts):
                continue
            if p.resolve() == REGISTRY:
                continue
            if p.name == "audit_citations.py":
                continue
            yield p


def scan() -> dict[str, list[str]]:
    """Map 'Author Year' token -> list of 'path:line' occurrences."""
    found: dict[str, list[str]] = defaultdict(list)
    for path in iter_source_files():
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        rel = path.relative_to(REPO)
        for n, line in enumerate(lines, 1):
            if MENTION_ONLY_MARKER in line:
                continue
            for surname, year in TOKEN_RE.findall(line):
                if surname in NOT_CITATIONS:
                    continue
                found[f"{surname} {year}"].append(f"{rel}:{n}")
    return found


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--strict",
        action="store_true",
        help="fail when an UNVERIFIED token is still referenced in source",
    )
    ap.add_argument(
        "--max-unregistered",
        type=int,
        default=None,
        metavar="N",
        help="fail when more than N tokens are absent from the registry",
    )
    args = ap.parse_args()

    if not REGISTRY.exists():
        print(f"FAIL: registry not found at {REGISTRY}")
        return 1

    verified, unverified = parse_registry(REGISTRY.read_text(encoding="utf-8"))
    if not verified:
        print("FAIL: could not parse VERIFIED block from citations.rs")
        return 1

    found = scan()
    unregistered = {t: locs for t, locs in found.items() if t not in verified and t not in unverified}
    flagged_in_use = {t: locs for t, locs in found.items() if t in unverified}

    print("=== Citation audit ===")
    print(f"registry: {len(verified)} verified, {len(unverified)} unverified")
    print(f"source:   {len(found)} distinct author-year tokens")
    print()

    if flagged_in_use:
        print(f"-- Referenced but UNVERIFIED ({len(flagged_in_use)}) --")
        print("   These must not be presented as established fact.")
        for token in sorted(flagged_in_use):
            locs = flagged_in_use[token]
            print(f"  {token:<28} {len(locs):>3}x  {locs[0]}")
        print()
    else:
        print("-- No UNVERIFIED citation is referenced as a claim. --")
        print()

    if unregistered:
        print(f"-- Not in registry ({len(unregistered)}) --")
        print("   Verify against Crossref/OpenAlex/arXiv, then add to VERIFIED")
        print("   with a DOI, or to UNVERIFIED with a reason.")
        for token in sorted(unregistered):
            locs = unregistered[token]
            shown = ", ".join(locs[:3]) + (f" (+{len(locs) - 3})" if len(locs) > 3 else "")
            print(f"  {token:<28} {len(locs):>3}x  {shown}")
        print()

    failures = []
    if args.strict and flagged_in_use:
        failures.append(f"{len(flagged_in_use)} UNVERIFIED token(s) still referenced")
    if args.max_unregistered is not None and len(unregistered) > args.max_unregistered:
        failures.append(
            f"{len(unregistered)} unregistered token(s) exceeds limit {args.max_unregistered}"
        )

    if failures:
        for f in failures:
            print(f"FAIL: {f}")
        return 1

    print("RESULT: OK")
    if unregistered:
        print(
            f"        {len(unregistered)} tokens still need a database lookup; "
            f"ratchet with --max-unregistered {len(unregistered)}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
