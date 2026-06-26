#!/usr/bin/env python3
"""Print a problem's Python+JS stub, statement, and example tests from the dev
scrape (src-tauri/resources/catalog_leetcode.json) — the spec a pack author needs.

Usage:
    python tools/show_problem.py <slug> [<slug> ...]
"""
import io
import json
import os
import sys

try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")  # avoid cp1252 crash on Windows
except Exception:  # noqa: BLE001
    pass

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRAPE = os.path.join(ROOT, "src-tauri", "resources", "catalog_leetcode.json")


def main(slugs):
    with io.open(SCRAPE, encoding="utf-8") as f:
        q = {r["slug"]: r for r in json.load(f)["questions"]}
    for s in slugs:
        r = q.get(s)
        print("=" * 70)
        if not r:
            print(f"SLUG: {s} | NOT FOUND in scrape")
            continue
        cs = r.get("code_stubs") or {}
        print(f"SLUG: {s} | diff: {r.get('difficulty')} | premium: {r.get('is_premium')}")
        print("--- PYTHON STUB ---")
        print((cs.get("python3") or cs.get("python") or "(none)").rstrip())
        print("--- JAVASCRIPT STUB ---")
        print((cs.get("javascript") or "(none)").rstrip())
        print("--- BODY TEXT ---")
        print((r.get("body_text") or "")[:4000])
        print("--- EXAMPLE TESTS ---")
        for t in (r.get("example_tests") or [])[:4]:
            print(t.get("input_raw", ""))


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("usage: python tools/show_problem.py <slug> [<slug> ...]")
        sys.exit(1)
    main(sys.argv[1:])
