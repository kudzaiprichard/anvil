#!/usr/bin/env python3
"""Combined progress view for the multi-session parallel authoring effort.

Reads the authoritative manifest (`tools/packs/index.json`) for verified counts
per batch, and prints each session's self-reported note
(`tools/packs/.progress/session-*.md`). Read-only — safe to run any time from any
session.

Usage: python tools/progress_summary.py
"""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "tools" / "packs" / "index.json"
SCRAPE = ROOT / "src-tauri" / "resources" / "catalog" / "catalog_leetcode.json"
PROGRESS_DIR = ROOT / "tools" / "packs" / ".progress"

# Lane ownership from PARALLEL_PLAN.md (batch -> session).
OWNER = {2: 1, 7: 1, 12: 1, 3: 2, 8: 2, 13: 2, 4: 3, 9: 3, 14: 3,
         5: 4, 10: 4, 15: 4, 6: 5, 11: 5, 16: 5, 1: "-", 0: "-"}


def batch_of_row(i: int) -> int:
    return (i // 200) + 1


def main() -> None:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8")) if MANIFEST.exists() else {}
    verified = Counter(v.get("batch", 0) for v in manifest.values())

    # Authorable-denominator per batch is unknown without categorizing, so show
    # the raw row count (200, last batch 187) as the ceiling.
    rows = []
    if SCRAPE.exists():
        scrape = json.loads(SCRAPE.read_text(encoding="utf-8"))
        rows = scrape.get("questions", scrape if isinstance(scrape, list) else [])
    total_rows = Counter(batch_of_row(i) for i, _ in enumerate(rows))

    print(f"=== verified packs per batch (manifest: {len(manifest)} total) ===")
    print(f"{'batch':>5} {'owner':>6} {'verified':>9} {'rows':>6}")
    for b in sorted(set(list(total_rows) + list(verified))):
        if b == 0:
            continue
        print(f"{b:>5} {str(OWNER.get(b, '?')):>6} {verified.get(b, 0):>9} {total_rows.get(b, 0):>6}")
    done = sum(verified.values())
    print(f"\nfoundation (batch 0): {verified.get(0, 0)} | grand total verified: {done}")

    print("\n=== per-session progress notes ===")
    notes = sorted(PROGRESS_DIR.glob("session-*.md")) if PROGRESS_DIR.exists() else []
    if not notes:
        print("(none yet — sessions write tools/packs/.progress/session-<N>.md)")
    for f in notes:
        print(f"\n--- {f.name} ---")
        print(f.read_text(encoding="utf-8").strip() or "(empty)")


if __name__ == "__main__":
    main()
