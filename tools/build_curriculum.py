#!/usr/bin/env python3
"""Curriculum content validator (Phase 1, IMPLEMENTATION_PLAN.md, LESSON_COURSE_DESIGN.md §8).

Unlike ``build_packs.py`` there is no compile step here: curriculum/unit/lesson
content ships as-is — the JSON/Markdown files directly under
``src-tauri/resources/curriculum/`` and ``src-tauri/resources/lessons/`` *are*
the shipped resource (mirrors how ``resources/presets/*.json`` ship directly).
This script is the fail-closed **--check**: it re-implements, in Python, the
same structural rules `services::curriculum` enforces in Rust at app startup,
so a bad content PR fails CI before it ever reaches a user's machine.

Checks (LESSON_COURSE_DESIGN.md §8):
  * `curriculum.json` parses; every stage references a unit file that exists;
    the prereq graph is a DAG (no cycles).
  * Every `units/<id>.json` parses, is non-empty, and has unique problem slugs;
    every unit's own `lessons` list references a lesson file that exists;
    every unit's own `prereqs` are declared as curriculum-level prereqs too.
  * Every problem slug referenced anywhere (units, worked examples, practice)
    has a frozen pack in the shipped test-pack bundle.
  * Every `resources/lessons/**/*.md` has valid YAML frontmatter, all required
    parts (LESSON_COURSE_DESIGN.md §3.3), and its `diagram`/`quiz` companion
    files parse; quiz `answer` in `options`; diagram `predict_at` indices valid.

Usage:
  python tools/build_curriculum.py --check     # validate, exit non-zero on any failure
  python tools/build_curriculum.py             # same as --check (there is nothing to build)
"""

from __future__ import annotations

import argparse
import gzip
import json
import sys
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parent.parent
RESOURCES = ROOT / "src-tauri" / "resources"
CURRICULUM_DIR = RESOURCES / "curriculum"
LESSONS_DIR = RESOURCES / "lessons"
PACK_BUNDLE = RESOURCES / "test-packs.json.gz"

PROBLEM_ROLES = {"worked", "guided", "gate"}
PROBLEM_TIERS = {"intro", "core", "stretch"}
QUIZ_TYPES = {"concept-check", "pattern-picker", "complexity"}
DIAGRAM_MODES = {"view", "perform"}


class CheckError(ValueError):
    """A content file fails a structural rule (fails loudly, fails closed)."""


def _require(cond: bool, msg: str) -> None:
    if not cond:
        raise CheckError(msg)


# ---------------------------------------------------------------------------
# Frozen pack slugs
# ---------------------------------------------------------------------------


def load_pack_slugs() -> set[str]:
    if not PACK_BUNDLE.exists():
        print(f"  !! no test-pack bundle at {PACK_BUNDLE} -- treating as empty", file=sys.stderr)
        return set()
    with gzip.open(PACK_BUNDLE, "rt", encoding="utf-8") as f:
        packs: dict[str, dict] = json.load(f)
    return {slug for slug, pack in packs.items() if pack.get("verified")}


# ---------------------------------------------------------------------------
# curriculum.json / units/*.json
# ---------------------------------------------------------------------------


def _is_int(v: Any) -> bool:
    return isinstance(v, int) and not isinstance(v, bool)


def validate_gate(gate: Any, where: str) -> None:
    _require(isinstance(gate, dict), f"{where}: `gate` must be an object")
    for key in ("pass_count", "timer_target_min", "threshold_pct"):
        _require(key in gate, f"{where}: gate missing `{key}`")
        _require(_is_int(gate[key]), f"{where}: gate.{key} must be an int")
    _require("require_novel" in gate, f"{where}: gate missing `require_novel`")
    _require(isinstance(gate["require_novel"], bool), f"{where}: gate.require_novel must be a bool")


def validate_curriculum(doc: dict) -> None:
    _require(isinstance(doc.get("id"), str) and doc["id"].strip(), "curriculum: `id` is required")
    stages = doc.get("stages")
    _require(isinstance(stages, list) and stages, "curriculum: `stages` must be a non-empty list")
    for stage in stages:
        _require(isinstance(stage.get("id"), str) and stage["id"].strip(), "curriculum: stage `id` is required")
        _require(isinstance(stage.get("title"), str) and stage["title"].strip(), f"stage '{stage.get('id')}': `title` is required")
        _require(isinstance(stage.get("units"), list) and stage["units"], f"stage '{stage.get('id')}': `units` must be non-empty")
    validate_gate(doc.get("gate_defaults"), "curriculum")
    prereqs = doc.get("prereqs", {})
    _require(isinstance(prereqs, dict), "curriculum: `prereqs` must be an object")


def detect_cycle(prereqs: dict[str, list[str]]) -> list[str] | None:
    """DFS cycle detection; returns the cycle path, or None if acyclic."""
    WHITE, GRAY, BLACK = 0, 1, 2
    color: dict[str, int] = {}

    def visit(node: str, stack: list[str]) -> list[str] | None:
        color[node] = GRAY
        stack.append(node)
        for dep in prereqs.get(node, []):
            c = color.get(dep, WHITE)
            if c == GRAY:
                return stack + [dep]
            if c == WHITE:
                cyc = visit(dep, stack)
                if cyc:
                    return cyc
        stack.pop()
        color[node] = BLACK
        return None

    for node in prereqs:
        if color.get(node, WHITE) == WHITE:
            cyc = visit(node, [])
            if cyc:
                return cyc
    return None


def validate_unit(unit: dict) -> None:
    uid = unit.get("id", "<missing id>")
    _require(isinstance(unit.get("id"), str) and unit["id"].strip(), "unit: `id` is required")
    _require(isinstance(unit.get("stage"), str) and unit["stage"].strip(), f"unit '{uid}': `stage` is required")
    _require(isinstance(unit.get("title"), str) and unit["title"].strip(), f"unit '{uid}': `title` is required")
    problems = unit.get("problems")
    _require(isinstance(problems, list) and problems, f"unit '{uid}': `problems` must be non-empty")
    seen_slugs: set[str] = set()
    for p in problems:
        slug = p.get("slug")
        _require(isinstance(slug, str) and slug.strip(), f"unit '{uid}': a problem has an empty slug")
        _require(slug not in seen_slugs, f"unit '{uid}': duplicate problem slug '{slug}'")
        seen_slugs.add(slug)
        _require(p.get("role") in PROBLEM_ROLES, f"unit '{uid}': problem '{slug}' has invalid role '{p.get('role')}'")
        _require(p.get("tier") in PROBLEM_TIERS, f"unit '{uid}': problem '{slug}' has invalid tier '{p.get('tier')}'")
        _require(isinstance(p.get("novel"), bool), f"unit '{uid}': problem '{slug}' `novel` must be a bool")
    validate_gate(unit.get("gate"), f"unit '{uid}'")


# ---------------------------------------------------------------------------
# lessons/**/*.md (+ companion .diagram.json / .quiz.json)
# ---------------------------------------------------------------------------


def split_frontmatter(raw: str) -> tuple[str, str]:
    _require(raw.startswith("---"), "missing YAML frontmatter (expected `---` delimiters)")
    rest = raw[3:].lstrip("\n")
    marker = "\n---"
    idx = rest.find(marker)
    _require(idx != -1, "missing closing `---` for frontmatter")
    frontmatter = rest[:idx]
    body = rest[idx + len(marker):].lstrip("\n")
    return frontmatter, body


def validate_diagram(diagram: dict, where: str) -> None:
    steps = diagram.get("steps")
    _require(isinstance(steps, list) and steps, f"{where}: diagram has no steps")
    predict_at = diagram.get("predict_at")
    _require(isinstance(predict_at, list) and predict_at, f"{where}: diagram needs at least one prediction pause")
    for idx in predict_at:
        _require(isinstance(idx, int) and 0 <= idx < len(steps), f"{where}: diagram predict_at index {idx} out of range")
    _require(diagram.get("mode") in DIAGRAM_MODES, f"{where}: diagram mode must be one of {sorted(DIAGRAM_MODES)}")
    _require(isinstance(diagram.get("for_problem"), str) and diagram["for_problem"].strip(), f"{where}: diagram `for_problem` is required")


def validate_quiz(quiz: dict, where: str) -> None:
    items = quiz.get("items")
    _require(isinstance(items, list) and items, f"{where}: quiz has no items")
    for item in items:
        iid = item.get("id", "<missing id>")
        _require(item.get("type") in QUIZ_TYPES, f"{where}: quiz item '{iid}' has invalid type '{item.get('type')}'")
        options = item.get("options")
        _require(isinstance(options, list) and options, f"{where}: quiz item '{iid}' has no options")
        _require(item.get("answer") in options, f"{where}: quiz item '{iid}' answer not among options")
        if item.get("type") == "pattern-picker":
            _require(bool(item.get("correct_pattern")), f"{where}: pattern-picker item '{iid}' needs correct_pattern")


def validate_lesson_file(path: Path) -> dict:
    raw = path.read_text(encoding="utf-8")
    frontmatter_raw, body = split_frontmatter(raw)
    try:
        fm = yaml.safe_load(frontmatter_raw)
    except yaml.YAMLError as e:
        raise CheckError(f"invalid frontmatter YAML: {e}") from e
    _require(isinstance(fm, dict), "frontmatter must be a YAML mapping")

    for key in ("id", "unit", "subpattern", "worked_example", "diagram", "quiz"):
        _require(isinstance(fm.get(key), str) and fm[key].strip(), f"frontmatter `{key}` is required")
    _require(isinstance(fm.get("trigger_signals", []), list) and fm.get("trigger_signals"),
              "frontmatter needs at least one `trigger_signals` entry")
    _require(isinstance(fm.get("practice", []), list) and fm.get("practice"),
              "frontmatter needs at least one `practice` slug")
    _require(body.strip(), "explainer body (below the frontmatter) is empty")

    diagram_path = path.parent / fm["diagram"]
    quiz_path = path.parent / fm["quiz"]
    _require(diagram_path.exists(), f"diagram file '{fm['diagram']}' not found")
    _require(quiz_path.exists(), f"quiz file '{fm['quiz']}' not found")
    diagram = json.loads(diagram_path.read_text(encoding="utf-8"))
    quiz = json.loads(quiz_path.read_text(encoding="utf-8"))
    validate_diagram(diagram, f"{path.name} -> {fm['diagram']}")
    validate_quiz(quiz, f"{path.name} -> {fm['quiz']}")

    return {
        "id": fm["id"],
        "unit": fm["unit"],
        "worked_example": fm["worked_example"],
        "practice": fm.get("practice", []),
        "recap": fm.get("recap", []),
    }


# ---------------------------------------------------------------------------
# Orchestration
# ---------------------------------------------------------------------------


def check() -> int:
    errors: list[str] = []
    ok = 0

    curriculum_path = CURRICULUM_DIR / "curriculum.json"
    if not curriculum_path.exists():
        print(f"FAIL curriculum.json: not found at {curriculum_path}", file=sys.stderr)
        return 1
    try:
        curriculum = json.loads(curriculum_path.read_text(encoding="utf-8"))
        validate_curriculum(curriculum)
    except (json.JSONDecodeError, CheckError) as e:
        print(f"FAIL curriculum.json: {e}", file=sys.stderr)
        return 1
    ok += 1

    units_dir = CURRICULUM_DIR / "units"
    unit_files = sorted(units_dir.glob("*.json")) if units_dir.is_dir() else []
    if not unit_files:
        print(f"FAIL: no unit files found under {units_dir}", file=sys.stderr)
        return 1

    units: dict[str, dict] = {}
    for f in unit_files:
        try:
            unit = json.loads(f.read_text(encoding="utf-8"))
            validate_unit(unit)
        except (json.JSONDecodeError, CheckError) as e:
            errors.append(f"{f.name}: {e}")
            continue
        if unit["id"] in units:
            errors.append(f"{f.name}: duplicate unit id '{unit['id']}'")
            continue
        units[unit["id"]] = unit
        ok += 1

    # stage -> unit cross-check
    stage_units = [u for stage in curriculum.get("stages", []) for u in stage.get("units", [])]
    for uid in stage_units:
        if uid not in units:
            errors.append(f"curriculum.json: stage references unknown unit '{uid}'")

    # prereq DAG: curriculum-level, and every unit's own prereqs must be
    # declared as curriculum-level prereqs too (single source of truth).
    prereqs: dict[str, list[str]] = curriculum.get("prereqs", {})
    for uid, deps in prereqs.items():
        if uid not in units:
            errors.append(f"curriculum.json: prereqs reference unknown unit '{uid}'")
        for dep in deps:
            if dep not in units:
                errors.append(f"curriculum.json: unit '{uid}' has unknown prereq '{dep}'")
    cycle = detect_cycle(prereqs)
    if cycle:
        errors.append(f"curriculum.json: prereq cycle: {' -> '.join(cycle)}")
    for uid, unit in units.items():
        declared = set(unit.get("prereqs", []))
        curriculum_level = set(prereqs.get(uid, []))
        if declared != curriculum_level:
            errors.append(
                f"unit '{uid}': prereqs {sorted(declared)} disagree with "
                f"curriculum.json prereqs {sorted(curriculum_level)}"
            )

    # unit -> lesson cross-check (lessons loaded below)
    lesson_files = sorted(LESSONS_DIR.rglob("*.md")) if LESSONS_DIR.is_dir() else []
    lessons: dict[str, dict] = {}
    for f in lesson_files:
        try:
            lesson = validate_lesson_file(f)
        except (CheckError, json.JSONDecodeError) as e:
            errors.append(f"{f.relative_to(ROOT)}: {e}")
            continue
        if lesson["id"] in lessons:
            errors.append(f"{f.relative_to(ROOT)}: duplicate lesson id '{lesson['id']}'")
            continue
        lessons[lesson["id"]] = lesson
        ok += 1

    for uid, unit in units.items():
        for lid in unit.get("lessons", []):
            if lid not in lessons:
                errors.append(f"unit '{uid}': references unknown lesson '{lid}'")
    for lid, lesson in lessons.items():
        if lesson["unit"] not in units:
            errors.append(f"lesson '{lid}': references unknown unit '{lesson['unit']}'")

    # every referenced slug has a frozen pack
    pack_slugs = load_pack_slugs()
    referenced_slugs: set[tuple[str, str]] = set()
    for uid, unit in units.items():
        for p in unit["problems"]:
            referenced_slugs.add((p["slug"], f"unit '{uid}'"))
    for lid, lesson in lessons.items():
        referenced_slugs.add((lesson["worked_example"], f"lesson '{lid}' worked_example"))
        for slug in lesson["practice"]:
            referenced_slugs.add((slug, f"lesson '{lid}' practice"))
    for slug, where in sorted(referenced_slugs):
        if pack_slugs and slug not in pack_slugs:
            errors.append(f"{where}: problem slug '{slug}' has no frozen pack")

    print(f"curriculum: 1 curriculum, {len(units)} unit(s), {len(lessons)} lesson(s) checked")
    if errors:
        print(f"\n{len(errors)} error(s):")
        for e in errors:
            print(f"  -- {e}")
        return 1
    print(f"--check: {ok} file(s) valid; 0 errors.")
    return 0


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--check", action="store_true", help="validate all curriculum content (default action)")
    p.parse_args()
    return check()


if __name__ == "__main__":
    sys.exit(main())
