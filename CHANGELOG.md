# Changelog

All notable changes to Anvil are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-07-20

Release engineering and delivery: the catalog is now baked into installers at build time, the
installers carry the forged-iron & ember branding on every platform, and cutting a release is
watched end-to-end from the terminal.

### Added

- **Build-time catalog embedding** — the problem catalog is fetched and embedded at build time
  from a single `ANVIL_CATALOG_URL` (dev points at the LeetCode-derived catalog, production at
  Anvil's own original catalog), so installers ship with the catalog baked in instead of fetching
  it at runtime. (#45)
- **Live release build monitor** — after the tag push, `release.mjs` streams each platform build
  job (Linux, Windows, macOS arm64 + x64) to completion in the terminal, and new fail-fast
  pre-flight gates catch a doomed release in milliseconds: a fresh-unused-tag check and empty-shell
  guards on both the pack bundle and the exported frontend. (#32)

### Changed

- **Forged-iron & ember branding** across the installers, with per-platform installer configuration
  for Windows, macOS, and Linux. (#34)
- **Path-aware CI** — the Rust and web jobs are skipped when their inputs did not change, cutting CI
  time on docs- and content-only changes. (#33)

## [0.2.0] - 2026-07-12

Closes the last judging gap: every one of the 3,026 catalog problems now runs against a full
hidden-test pack — including the 48 that previously shipped in Basic or Run-only mode.

### Added

- **48 new verified test packs** (catalog coverage 2,978 → **3,026, 100%**): node-reference
  arguments, cyclic and random-pointer lists, graphs, n-ary/quad/next-pointer trees, multilevel
  lists, serialization codecs, injected interactive objects, randomized-output problems, and the
  six multithreading problems.
- **New judge modes** — `round_trip` (codec problems judged by encode→decode canonicalization),
  `property` (randomized outputs validated by an op-replay validator, never by flaky statistics),
  and `concurrency` (real threads driven by a pack-shipped driver with amplified scheduling —
  tiny GIL switch interval, jitter injection, barrier starts, repeated runs — and a watchdog
  that turns deadlocks into an explained failure instead of a hang).
- **New wire formats** at the runner boundary: node references resolved by identity, deep-copy
  and shared-tail construction, freshness checks that reject returning input nodes from copy
  problems, and per-problem `Node` class injection matching each LeetCode stub.
- **Injected-object shims** with enforced call budgets (`isBadVersion`, `guess`, `rand7`,
  `MountainArray` ≤100 gets, `Master` ≤10 guesses, `CustomFunction`, iterators, nested integers).
- **Single-language packs** — packs may now declare `languages: ["python"]` (the concurrency set
  is Python-only; JavaScript has no shared-memory threads); the workspace hides unavailable
  languages and a second differently-synchronized Python oracle replaces the cross-language check.
- **Attempt-tier history** — every run/submit now records whether it was judged at full, basic,
  or run-only tier (new `tier` column, migration 0007).

### Fixed

- Design-mode problems can now receive real node structures in their constructors
  (binary-search-tree-iterator's long-standing gap).
- Imports no longer attach statement examples whose encoding is incompatible with a pack's wire
  format (they crashed or mis-judged); the pack's own leading tests are shown instead — this also
  gives design problems runnable visible cases for the first time.
- The Unix sandbox no longer blocks thread creation for problems that legitimately need threads
  (`RLIMIT_NPROC` counts tasks on Linux); timeout, memory, and kill-tree guards are unchanged.
- The workspace language selector no longer opens a Python-only problem parked on a carried-over
  JavaScript selection.

## [0.1.0] - 2026-07-10

First public release: a free, offline, guided DSA **course** that trains pattern *recognition* on real
LeetCode problems (bring-your-own-statement) with verified local judging.

### Added

- **Guided DSA course** — one connected, mastery-gated climb of **8 stages / 19 units / 62 lessons**.
  Each lesson teaches one sub-pattern with an explainer, explicit **trigger signals**, an interactive
  **prediction diagram** ("what happens next?"), a worked example solved in the real workspace, faded →
  independent practice, and formative quizzes. All content is data (Markdown + JSON resources); the
  schema, loader, and engine are code; per-user progress is SQLite.
- **Pattern-recognition trainer** — prompt-only, *unlabeled* **pattern-picker** drills (per lesson and a
  cross-unit interleaved pool) that train *which* technique an unfamiliar problem needs — the moat.
- **Mastery gates** — passing a unit requires solving fresh, unseen problems (≥1 novel) **hint-free,
  no-peek, under a soft timer**; only then does the next unit unlock. A **prerequisite DAG** drives
  parallel unlocking, with **diagnostic placement** to start at your frontier and **spiral reuse** so
  earlier patterns keep resurfacing.
- **FSRS spaced review** — solved/gated problems enter an on-device (`fsrs-rs`) spaced, interleaved
  queue and are re-solved *cold*; repeated failure demotes them. Honest habit layer: streaks with
  freezes, no XP-for-everything, no leaderboards.
- **Richer feedback** — a graduated Socratic **hint ladder** (off on gates), deterministic
  **complexity feedback** from op-count traces ("you wrote O(n²), optimal is O(n)"), and a
  **self-explanation gate** before the reference solution unlocks.
- **Stage-7 mixed capstone** — a pool of **unlabeled** problems across all units, plus a readiness
  signal: clearing it is the operational definition of "can solve unfamiliar problems alone."
- **Sandboxed local code runner** for Python and JavaScript — per-run timeout, memory cap, and
  temp-dir isolation (Job Objects on Windows). User code never runs in the WebView.
- **Offline test-pack judging** — 2,900+ verified packs frozen into
  `src-tauri/resources/test-packs.json.gz`. Packs carry reference solutions plus an independent
  brute-force oracle; the offline build computes expected outputs by *executing* the references and
  cross-checking Python vs JavaScript vs the oracle, so a wrong solution can never be frozen.
- **Name-agnostic catalog loader** — any `catalog*.json` / `catalog*.json.gz` in
  `src-tauri/resources/catalog/` is discovered at startup, loaded, merged (de-duplicated by slug), and
  mapped to its frozen test pack (the matched pack becomes the hidden judge). Multiple catalogs coexist
  and either can be swapped in with no code change.
- **Installer boundary gate** — `tools/check_release_boundary.py` fails the build if any `*leetcode*`
  catalog (or any bulk statement dump over the no-scrape baseline) would be bundled. Wired into CI and
  the build/release workflows; see [`RELEASING.md`](./RELEASING.md).
- **Open-source project docs** — `README.md`, `CONTRIBUTING.md` (incl. lesson authoring),
  `CODE_OF_CONDUCT.md`, `SECURITY.md`, `DISCLAIMER.md` (content & legal policy), `RELEASING.md`,
  `NOTICE`, and this changelog.

### Changed

- The dev-only LeetCode scrape moved from `.docs/my_questions.json` to
  `src-tauri/resources/catalog/catalog_leetcode.json` and is hard-ignored (`*leetcode*`). An original,
  redistributable catalog may be committed normally.

### Security

- **No third-party problem statements are ever bundled.** Public installers ship only the app, the
  frozen test packs, and the lessons; the user supplies LeetCode statements themselves. Enforced
  fail-closed by the installer boundary gate (see above).

[Unreleased]: https://github.com/kudzaiprichard/anvil/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/kudzaiprichard/anvil/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/kudzaiprichard/anvil/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kudzaiprichard/anvil/releases/tag/v0.1.0
