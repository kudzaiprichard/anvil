"""Builds src-tauri/resources/test-packs.json.gz from the readable fixture
source. Used until the generation pipeline (0009/0010) produces the real
bundle; the real pipeline writes the same format.

Usage: python tools/build_fixture_bundle.py [source.json]
"""

import gzip
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else (
        ROOT / "tools" / "fixtures" / "test-packs.fixture.json"
    )
    packs = json.loads(source.read_text(encoding="utf-8"))
    out = ROOT / "src-tauri" / "resources" / "test-packs.json.gz"
    # mtime=0 keeps the build reproducible (no timestamp in the gzip header)
    payload = json.dumps(packs, separators=(",", ":"), ensure_ascii=False)
    out.write_bytes(gzip.compress(payload.encode("utf-8"), mtime=0))
    print(f"wrote {out} ({out.stat().st_size} bytes, {len(packs)} pack(s))")


if __name__ == "__main__":
    main()
