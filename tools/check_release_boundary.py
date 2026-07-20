#!/usr/bin/env python3
"""Installer boundary check — the shipping safety gate (Phase 8).

Anvil's one inviolable shipping rule: **public installers ship no third-party
problem statements, and specifically never any `*leetcode*` catalog.** The dev
scrape (`resources/catalog/catalog_leetcode.json`) is gitignored so a clean
checkout — CI, or a fresh clone — never has it. But the Tauri bundler copies the
whole `resources/catalog/` directory, so a release built on a developer machine
that still holds the scrape *would* sweep it into the installer. This script is
the fail-closed gate that makes that impossible to do by accident.

It answers two questions, from the same source of truth the bundler uses
(`src-tauri/tauri.conf.json` → `bundle.resources`):

  1. **Would any `*leetcode*` file be bundled?**  Every resource entry is
     resolved to the concrete file set the bundler would copy; if any resolved
     file's name contains `leetcode` (case-insensitive), the gate fails.
  2. **Is the payload at the no-scrape baseline?**  The total size of the
     bundled resource set must stay under `MAX_BUNDLED_RESOURCE_BYTES`. A clean
     tree is ~4.3 MB; the LeetCode scrape alone is ~20 MB, so a differently
     *named* scrape (or any other bulk statement dump) trips the size guard even
     if it dodges the name guard.

Belt-and-suspenders, it also rejects a `bundle.resources` entry that literally
references `leetcode`, and — given ``--bundle-dir DIR`` — scans an already-built
bundle tree for the same violations.

Usage:
  python tools/check_release_boundary.py                 # gate the source tree (pre-bundle)
  python tools/check_release_boundary.py --bundle-dir X  # also scan a built bundle's resources
  python tools/check_release_boundary.py --json          # machine-readable summary

Exit code is non-zero on any violation — wire it into the release/build
workflows *before* the bundle step so a tainted tree can never be packaged.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC_TAURI = ROOT / "src-tauri"
TAURI_CONF = SRC_TAURI / "tauri.conf.json"

# The forbidden pattern. Matches the `.gitignore` rule (`*leetcode*`) and the
# DISCLAIMER: whatever is committed/shipped must be original, never a scrape.
FORBIDDEN_GLOB = "*leetcode*"

# No-scrape baseline. A clean bundled resource set is ~4.3 MB (test packs +
# lessons + curriculum + pylib + presets, with an empty `resources/catalog/`).
# The LeetCode scrape alone is ~20 MB, so 10 MB leaves ample headroom for the
# content to grow while still catching any bulk statement dump that leaks in.
MAX_BUNDLED_RESOURCE_BYTES = 10 * 1024 * 1024


def _is_forbidden(name: str) -> bool:
    return fnmatch.fnmatch(name.lower(), FORBIDDEN_GLOB)


def _resolve_entry(entry: str, base: Path) -> list[Path]:
    """Resolve one `bundle.resources` entry to the concrete files the Tauri
    bundler would copy — mirroring its semantics: a glob expands, a directory is
    copied recursively, a plain path is the single file."""
    # Tauri resource entries may be a "src" string or a {src, target} object.
    if any(ch in entry for ch in "*?[]"):
        return sorted(p for p in base.glob(entry) if p.is_file())
    target = base / entry
    if target.is_dir():
        return sorted(p for p in target.rglob("*") if p.is_file())
    if target.is_file():
        return [target]
    return []


def _load_resource_entries() -> list[str]:
    conf = json.loads(TAURI_CONF.read_text(encoding="utf-8"))
    resources = conf.get("bundle", {}).get("resources", [])
    out: list[str] = []
    for r in resources:
        if isinstance(r, str):
            out.append(r)
        elif isinstance(r, dict) and isinstance(r.get("path"), str):
            out.append(r["path"])
        elif isinstance(r, dict) and isinstance(r.get("src"), str):
            out.append(r["src"])
    return out


def check_source_tree() -> list[str]:
    """Gate the source tree exactly as the bundler sees it."""
    errors: list[str] = []
    entries = _load_resource_entries()

    # (0) No entry may literally name the scrape.
    for e in entries:
        if "leetcode" in e.lower():
            errors.append(f"tauri.conf.json bundle.resources entry references leetcode: '{e}'")

    # (1) + (2) Resolve every entry; check names and total size.
    bundled: list[Path] = []
    for e in entries:
        bundled.extend(_resolve_entry(e, SRC_TAURI))

    total = 0
    for p in bundled:
        total += p.stat().st_size
        if _is_forbidden(p.name):
            rel = p.relative_to(ROOT) if ROOT in p.parents else p
            errors.append(f"WOULD BUNDLE a forbidden file: {rel} (matches {FORBIDDEN_GLOB})")

    if total > MAX_BUNDLED_RESOURCE_BYTES:
        errors.append(
            f"bundled resources are {total / 1024 / 1024:.2f} MB, over the "
            f"{MAX_BUNDLED_RESOURCE_BYTES / 1024 / 1024:.0f} MB no-scrape baseline "
            f"— a bulk statement dump may have leaked into resources/"
        )

    print(
        f"boundary: {len(bundled)} file(s) across {len(entries)} resource entr(ies), "
        f"{total / 1024 / 1024:.2f} MB (limit {MAX_BUNDLED_RESOURCE_BYTES / 1024 / 1024:.0f} MB)"
    )
    return errors


def check_built_bundle(bundle_dir: Path) -> list[str]:
    """Scan an already-produced bundle tree (e.g. the staged `resources/` next to
    the packaged binary) for any forbidden file."""
    errors: list[str] = []
    if not bundle_dir.exists():
        return [f"--bundle-dir path does not exist: {bundle_dir}"]
    found = 0
    for root, _dirs, files in os.walk(bundle_dir):
        for f in files:
            found += 1
            if _is_forbidden(f):
                errors.append(f"built bundle contains a forbidden file: {Path(root, f)}")
    print(f"boundary(bundle): scanned {found} file(s) under {bundle_dir}")
    return errors


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "--bundle-dir",
        type=Path,
        help="also scan an already-built bundle tree for forbidden files",
    )
    ap.add_argument("--json", action="store_true", help="emit a machine-readable summary")
    args = ap.parse_args()

    if not TAURI_CONF.exists():
        print(f"FAIL: no tauri.conf.json at {TAURI_CONF}", file=sys.stderr)
        return 1

    errors = check_source_tree()
    if args.bundle_dir:
        errors += check_built_bundle(args.bundle_dir)

    if args.json:
        print(json.dumps({"ok": not errors, "errors": errors}))

    if errors:
        print(f"\nBOUNDARY CHECK FAILED — {len(errors)} violation(s):", file=sys.stderr)
        for e in errors:
            print(f"  !! {e}", file=sys.stderr)
        print(
            "\nA release must never bundle a *leetcode* catalog. "
            "Remove the dev scrape from src-tauri/resources/catalog/ (it is gitignored) "
            "and rebuild from a clean tree.",
            file=sys.stderr,
        )
        return 1

    print("boundary check PASSED: no *leetcode* content bundled; payload at the no-scrape baseline.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
