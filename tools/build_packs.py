#!/usr/bin/env python3
"""Keyless test-pack build (task 0001, CONTENT_DESIGN.md §2-4).

Compiles the hand-authored source files in ``tools/packs/<slug>.json`` into the
shipped, verified bundle. There is **no model and no API key on this path** — the
intelligence is the author (Claude Code) writing the source *now*; this script
only verifies it by execution and freezes the result.

What it does, per source file:

  1. Load the matching scrape row (``.docs/my_questions.json``) for its
     statement-example anchors, code stubs (entry point), and qid.
  2. Translate the source into the ``gen`` shape the proven verifier consumes and
     run :func:`generate_test_packs.verify_and_build` — which anchors the optimal
     solution against the statement examples, **computes every expected value by
     execution** (never authored), cross-checks the brute-force oracle, and
     requires every supported language to agree. Fail ⇒ quarantine (fail closed).
  3. Cross-check any *extra* languages declared in ``solutions`` (beyond the
     always-verified python + javascript) by agreement against the computed
     expecteds — see :data:`LANGUAGES`. Adding a language is purely additive.
  4. Record an immutability entry in ``tools/packs/index.json``:
     ``{ batch, source_hash, verified_langs, verified_at }``.

Immutability / resumability (the freeze):

  * A slug whose source hash is unchanged **and** is already in the bundle is
    skipped — its frozen pack is reused untouched.
  * A slug whose source hash changed but is already bundled trips a **loud
    warning** and is left frozen (the old pack is kept) unless ``--allow-refreeze``
    is passed. We never silently rebuild a pack we already shipped.

Usage:
  python tools/build_packs.py --batch 0                 # build/refresh, no key
  python tools/build_packs.py --batch 0 --bundle        # also gzip the bundle
  python tools/build_packs.py --only two-sum            # one slug
  python tools/build_packs.py --allow-refreeze --only x # intentionally re-verify

See tools/LANGUAGES.md for the supported-language registry and the
"add a language" checklist. See tools/README.md for the wider context.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any, Optional

# Ensure this script's own directory is importable when invoked by path.
sys.path.insert(0, str(Path(__file__).resolve().parent))

# The verification engine is shared, byte-for-byte, with the API path. This
# import pulls in the harness driver, the example parser, and verify_and_build;
# it never imports `anthropic` (that is lazy, inside GenClient, unused here).
import generate_test_packs as engine  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
PACKS_DIR = ROOT / "tools" / "packs"
MANIFEST_PATH = PACKS_DIR / "index.json"
DEFAULT_SCRAPE = ROOT / ".docs" / "my_questions.json"
DEFAULT_OUT = ROOT / "tools" / "test-packs.json"


# ---------------------------------------------------------------------------
# Language registry (see tools/LANGUAGES.md)
# ---------------------------------------------------------------------------
#
# `python` is the source of truth: every `expected` value is computed by
# executing the python reference (CONTENT_DESIGN.md §2). Every *other* language
# is verified *by agreement* against those expecteds — it adds zero new ground
# truth. python + javascript are verified inside verify_and_build today; this
# table lets the build also accept additional languages additively (a new entry
# here + a harness/sandbox wrapper + one solution per problem — nothing in the
# agnostic core changes). `runnable` is False until that one-time runtime infra
# lands, so a declared-but-not-yet-runnable language is reported, not silently
# treated as verified.
LANGUAGES: dict[str, dict[str, Any]] = {
    "python": {"runnable": True, "source_of_truth": True},
    "javascript": {"runnable": True, "source_of_truth": False},
    # Example of the additive shape for a future language:
    # "go": {"runnable": False, "source_of_truth": False},
}

# Verified inside verify_and_build itself (the proven core); never re-run here.
CORE_LANGS = ("python", "javascript")


class SourceError(ValueError):
    """A source file is structurally invalid (fails loudly, fails closed)."""


# ---------------------------------------------------------------------------
# Source schema + validation
# ---------------------------------------------------------------------------


def _require(cond: bool, msg: str) -> None:
    if not cond:
        raise SourceError(msg)


def validate_source(slug: str, src: dict) -> None:
    """Validate one ``tools/packs/<slug>.json`` source. Raises SourceError with a
    precise message on the first problem. The source carries **no expected
    values** by design — that is the structural guarantee no answer is typed."""
    _require(isinstance(src, dict), "source must be a JSON object")
    _require(src.get("slug") == slug, f"`slug` must equal the filename stem {slug!r}")

    judge_type, _ = _judge_fields(src.get("judge"))
    _require(
        judge_type in engine.JUDGE_TYPES,
        f"`judge` type must be one of {sorted(engine.JUDGE_TYPES)}, got {judge_type!r}",
    )

    sols = src.get("solutions")
    _require(isinstance(sols, dict), "`solutions` must be an object keyed by language")
    for lang in CORE_LANGS:
        _require(
            isinstance(sols.get(lang), str) and sols[lang].strip(),
            f"`solutions.{lang}` is required and must be non-empty source",
        )
    for lang in sols:
        _require(
            lang in LANGUAGES,
            f"`solutions.{lang}` is not a registered language "
            f"(see tools/LANGUAGES.md: {sorted(LANGUAGES)})",
        )

    _require(isinstance(src.get("no_anchor_ok", False), bool), "`no_anchor_ok` must be a boolean")

    io = src.get("io_types")
    if io is not None:
        _require(
            isinstance(io, dict) and isinstance(io.get("params"), list) and "returns" in io,
            "`io_types` must be {params: [<type>, ...], returns: <type>}",
        )
        _NODE_TYPES = ("json", "linked_list", "tree")

        def _ok_type(t):
            if isinstance(t, str):
                return t in _NODE_TYPES
            return isinstance(t, dict) and set(t) == {"list_of"} and _ok_type(t["list_of"])

        for t in list(io["params"]) + [io["returns"]]:
            _require(_ok_type(t),
                     f"io_types entries must be json|linked_list|tree|{{list_of: <type>}}, got {t!r}")

    _require(isinstance(src.get("pattern", ""), str), "`pattern` must be a string")
    hints = src.get("hints", [])
    _require(
        isinstance(hints, list) and all(isinstance(h, str) for h in hints),
        "`hints` must be a list of strings",
    )
    constraints = src.get("constraints", [])
    _require(isinstance(constraints, list), "`constraints` must be a list")
    # `len`/`value` bounds map to Rust `(i64, i64)` — they MUST be 2-element
    # integer arrays (floats/booleans/wrong-length break bundle deserialization
    # at runtime, degrading the whole store to empty). Omit the field for
    # non-integer params (e.g. float[] values) rather than using float bounds.
    for c in constraints:
        if not isinstance(c, dict):
            continue
        for bound in ("len", "value"):
            if bound not in c:
                continue
            v = c[bound]
            _require(
                isinstance(v, list)
                and len(v) == 2
                and all(isinstance(x, int) and not isinstance(x, bool) for x in v),
                f"constraint `{bound}` must be a 2-element array of integers "
                f"(got {v!r}); omit it for non-integer params",
            )

    edge = src.get("edge_inputs", [])
    _require(isinstance(edge, list), "`edge_inputs` must be a list")
    for item in edge:
        if isinstance(item, dict):
            args = item.get("input", item.get("args"))
            _require(isinstance(args, list),
                     "an `edge_inputs` entry needs `input` as a positional arg-list")
            _require(item.get("kind", "edge") in ("edge", "boundary", "trap"),
                     "an `edge_inputs` `kind` must be edge|boundary|trap")
            for banned in ("expected", "output", "outputs"):
                _require(banned not in item,
                         f"`edge_inputs` entry must not contain `{banned}` — expecteds are computed")
        else:
            _require(isinstance(item, list),
                     "`edge_inputs` entries must be arg-lists or {kind, description, input} objects")
    if src.get("no_anchor_ok"):
        _require(
            len(edge) > 0,
            "`no_anchor_ok` packs must supply `edge_inputs` — they are the only literal tests "
            "when the statement examples cannot be parsed (e.g. design / in_place)",
        )

    # Guard the load-bearing rule structurally: reject any stray expected field.
    for banned in ("expected", "output", "outputs", "tests", "expected_outputs"):
        _require(banned not in src, f"source must not contain `{banned}` — expecteds are computed")

    stress = src.get("stress", [])
    _require(isinstance(stress, list), "`stress` must be a list")
    for s in stress:
        _require(isinstance(s, dict) and "generator_python" in s,
                 "each `stress` entry needs a `generator_python`")

    if judge_type == "any_valid":
        _, extra = _judge_fields(src.get("judge"))
        _require(
            bool(extra.get("validator_python")) and bool(extra.get("validator_javascript")),
            "`any_valid` judge needs `validator_python` and `validator_javascript`",
        )


def _judge_fields(judge: Any) -> tuple[str, dict]:
    """Normalize `judge` (a bare string or an object) → (type, extra-fields)."""
    if isinstance(judge, str):
        return judge, {}
    if isinstance(judge, dict):
        jtype = judge.get("type", "")
        extra = {k: v for k, v in judge.items() if k != "type"}
        return jtype, extra
    return "", {}


def source_hash(src: dict) -> str:
    """Stable content hash of a source file (canonical JSON). Drives the freeze:
    same hash ⇒ same pack, skip; changed hash on a bundled slug ⇒ loud warning."""
    canonical = json.dumps(src, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return "sha256:" + hashlib.sha256(canonical.encode("utf-8")).hexdigest()


# ---------------------------------------------------------------------------
# Source -> the proven verifier's `gen` shape
# ---------------------------------------------------------------------------


def to_gen(src: dict) -> dict:
    """Translate a source file into the dict shape `verify_and_build` consumes.
    This is the *only* adapter; the verification logic itself is untouched."""
    judge_type, extra = _judge_fields(src["judge"])
    sols = src["solutions"]
    gen: dict[str, Any] = {
        "judge": judge_type,
        "solution_python": sols["python"],
        "solution_javascript": sols["javascript"],
        "pattern": src.get("pattern", ""),
        "hints": src.get("hints", []),
        "constraints": src.get("constraints", []),
        "edge_inputs": src.get("edge_inputs", []),
        "stress": src.get("stress", []),
    }
    if src.get("oracle_python"):
        gen["solution_python_brute"] = src["oracle_python"]
    if src.get("complexity"):
        gen["complexity"] = src["complexity"]
    if src.get("io_types"):
        gen["io_types"] = src["io_types"]
    if judge_type == "float" and "epsilon" in extra:
        gen["epsilon"] = extra["epsilon"]
    if judge_type == "in_place":
        gen["arg_index"] = extra.get("arg_index", 0)
    if judge_type == "any_valid":
        gen["validator_python"] = extra.get("validator_python", "")
        gen["validator_javascript"] = extra.get("validator_javascript", "")
    return gen


def cross_check_extra_languages(src: dict, pack: dict) -> tuple[list[str], Optional[str]]:
    """Verify any languages declared beyond the core py+js by agreement against
    the already-computed expecteds. Returns (extra_verified_langs, error). An
    error means a declared language disagreed or could not run → fail closed."""
    extra_verified: list[str] = []
    judge_type = pack["judge"]["type"]
    inputs = [t["input"] for t in pack["tests"]]
    expected = [t["expected"] for t in pack["tests"]]
    for lang, code in src["solutions"].items():
        if lang in CORE_LANGS:
            continue
        meta = LANGUAGES.get(lang, {})
        if not meta.get("runnable"):
            return extra_verified, (
                f"language {lang!r} has no runtime yet (see tools/LANGUAGES.md); "
                "remove the solution or add the one-time harness/sandbox infra"
            )
        entry_name = pack["entry_point"].get(lang) or pack["entry_point"]["javascript"]
        try:
            got = engine.run_harness(lang, code, entry_name, judge_type, inputs)
        except engine.HarnessError as e:
            return extra_verified, f"{lang} solution crashed: {e}"
        for inp, want, actual in zip(inputs, expected, got):
            if not engine.values_agree(judge_type, want, actual):
                return extra_verified, f"{lang} disagrees on {inp!r}: {want!r} vs {actual!r}"
        extra_verified.append(lang)
    return extra_verified, None


# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------


def load_json(path: Path, default: Any) -> Any:
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8"))
    return default


def build(args: argparse.Namespace) -> int:
    if not PACKS_DIR.is_dir():
        print(f"no source directory: {PACKS_DIR}", file=sys.stderr)
        return 2
    scrape_path = Path(args.scrape)
    if not scrape_path.exists():
        print(f"scrape file not found: {scrape_path}", file=sys.stderr)
        return 2
    scrape = json.loads(scrape_path.read_text(encoding="utf-8"))
    rows = scrape.get("questions", scrape if isinstance(scrape, list) else [])
    questions = {q["slug"]: q for q in rows if q.get("slug")}
    # Scrape-row index per slug → the batch a pack belongs to is row // 200 + 1.
    # This lets ANY session's build tag packs with their correct batch without
    # passing --batch, so parallel sessions never mislabel each other's work.
    row_batch = {
        q["slug"]: (i // 200) + 1 for i, q in enumerate(rows) if q.get("slug")
    }

    out_path = Path(args.out)
    packs: dict[str, dict] = load_json(out_path, {})
    manifest: dict[str, dict] = load_json(MANIFEST_PATH, {})

    only = set(args.only.split(",")) if args.only else None
    source_files = sorted(PACKS_DIR.glob("*.json"))
    source_files = [f for f in source_files if f.name != "index.json"]

    stats: dict[str, list] = {
        "verified": [], "skipped_frozen": [], "locked": [],
        "quarantined": [], "skipped": [],
    }

    for f in source_files:
        slug = f.stem
        if only is not None and slug not in only:
            continue
        try:
            src = json.loads(f.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            stats["quarantined"].append((slug, f"source is not valid JSON: {e}"))
            print(f"  -- {slug}: invalid JSON: {e}", flush=True)
            continue
        try:
            validate_source(slug, src)
        except SourceError as e:
            stats["quarantined"].append((slug, f"invalid source: {e}"))
            print(f"  -- {slug}: invalid source: {e}", flush=True)
            continue

        h = source_hash(src)
        prev = manifest.get(slug)
        bundled = slug in packs

        # --- the freeze (bypassed in --check: verify named slugs fresh) ------
        if args.check:
            pass
        elif prev and prev.get("source_hash") == h and bundled and not args.allow_refreeze:
            stats["skipped_frozen"].append(slug)
            continue
        elif prev and prev.get("source_hash") != h and bundled and not args.allow_refreeze:
            print(
                f"  !! {slug}: source CHANGED but pack is already frozen "
                f"(hash {prev.get('source_hash')} -> {h}). Keeping the frozen pack. "
                f"Pass --allow-refreeze to intentionally re-verify.",
                file=sys.stderr, flush=True,
            )
            stats["locked"].append(slug)
            continue

        # --- (re)verify ------------------------------------------------------
        q = questions.get(slug)
        if q is None:
            stats["skipped"].append((slug, "slug not present in the scrape"))
            print(f"  -- {slug}: not in scrape", flush=True)
            continue
        stub_py = q.get("code_stubs", {}).get("python", "")
        input_lines = [e.get("input_lines", []) for e in q.get("example_tests", [])]
        parsed = engine.parse_examples(input_lines, q.get("body_text", ""), stub_py)
        if not parsed.cases and not src.get("no_anchor_ok"):
            stats["skipped"].append((
                slug,
                "no parseable statement examples (no anchor); set `no_anchor_ok` to author "
                "from manual edge_inputs (design / in_place)",
            ))
            print(f"  -- {slug}: no anchor", flush=True)
            continue

        # `no_anchor_ok` means the statement examples are unreliable as anchors
        # (unparseable, or parsed but misaligned to the wrong input/output — e.g.
        # count-and-say's two example blocks cross-match). Drop them and rely on
        # the author's explicit edge_inputs instead.
        anchors = [] if src.get("no_anchor_ok") else parsed.cases
        result = engine.verify_and_build(q, to_gen(src), anchors)
        if not result.verified:
            stats["quarantined"].append((slug, result.reason))
            print(f"  -- {slug}: {result.reason}", flush=True)
            continue

        extra_langs, lang_err = cross_check_extra_languages(src, result.pack)
        if lang_err is not None:
            stats["quarantined"].append((slug, lang_err))
            print(f"  -- {slug}: {lang_err}", flush=True)
            continue

        packs[slug] = result.pack
        verified_langs = list(CORE_LANGS) + extra_langs
        manifest[slug] = {
            # Explicit --batch wins; else auto-assign from the scrape row; else
            # keep a prior value. Auto-assignment keeps parallel sessions correct.
            "batch": args.batch if args.batch is not None
            else row_batch.get(slug, (prev or {}).get("batch", 0)),
            "source_hash": h,
            "verified_langs": verified_langs,
            "verified_at": result.pack["generated_at"],
        }
        stats["verified"].append(slug)
        tag = "RE-VERIFIED" if (prev and bundled) else "OK"
        print(f"  {tag} {slug} [{'+'.join(verified_langs)}]", flush=True)

    # --- persist (skipped in --check: verify-only, no shared-file writes) ----
    if not args.check:
        out_path.write_text(
            json.dumps(packs, indent=2, sort_keys=True, ensure_ascii=False), encoding="utf-8"
        )
        MANIFEST_PATH.write_text(
            json.dumps(manifest, indent=2, sort_keys=True, ensure_ascii=False), encoding="utf-8"
        )

    if args.bundle and not args.check:
        _write_bundle(packs)

    # --- report --------------------------------------------------------------
    mode = "check (no writes)" if args.check else "keyless, no API"
    print(f"\n=== build summary ({mode}) ===")
    print(f"verified/rebuilt:  {len(stats['verified'])}  {stats['verified']}")
    print(f"skipped (frozen):  {len(stats['skipped_frozen'])}")
    print(f"locked (changed):  {len(stats['locked'])}  {stats['locked']}")
    print(f"quarantined:       {len(stats['quarantined'])}")
    for slug, reason in stats["quarantined"]:
        print(f"    {slug}: {reason}")
    print(f"skipped (no pack): {len(stats['skipped'])}")
    for slug, reason in stats["skipped"]:
        print(f"    {slug}: {reason}")
    if args.check:
        print(f"\n--check: verified {len(stats['verified'])} slug(s); wrote nothing.")
    else:
        print(f"\nbundle now holds {len(packs)} pack(s); wrote {out_path.name} + index.json")
        if not args.bundle:
            print("Next: python tools/build_packs.py --bundle   (or build_fixture_bundle.py)")
    # Fail closed: a non-empty quarantine list is a non-zero exit for CI.
    return 0 if not stats["quarantined"] else 1


def _write_bundle(packs: dict) -> None:
    import gzip

    out = ROOT / "src-tauri" / "resources" / "test-packs.json.gz"
    payload = json.dumps(packs, separators=(",", ":"), ensure_ascii=False)
    out.write_bytes(gzip.compress(payload.encode("utf-8"), mtime=0))
    print(f"wrote {out} ({out.stat().st_size} bytes, {len(packs)} pack(s))")


def main() -> int:
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    p.add_argument("--scrape", default=str(DEFAULT_SCRAPE), help="my_questions.json scrape")
    p.add_argument("--out", default=str(DEFAULT_OUT), help="uncompressed pack output")
    p.add_argument("--only", help="comma-separated slugs to (re)build")
    p.add_argument("--batch", type=int, help="batch number to record in the manifest")
    p.add_argument("--bundle", action="store_true", help="also gzip -> resources/test-packs.json.gz")
    p.add_argument("--allow-refreeze", action="store_true",
                   help="re-verify and overwrite packs whose source changed (rare)")
    p.add_argument("--check", action="store_true",
                   help="verify the named slugs and report; write nothing (safe for parallel use)")
    p.add_argument("--python", help="python interpreter for the harness")
    p.add_argument("--node", help="node interpreter for the harness")
    args = p.parse_args()

    if args.python:
        engine.PYTHON_PROGRAM = args.python
    if args.node:
        engine.NODE_PROGRAM = args.node

    return build(args)


if __name__ == "__main__":
    sys.exit(main())
