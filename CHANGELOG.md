# Changelog

All notable changes to Anvil are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Sandboxed local code runner** for Python and JavaScript — per-run timeout, memory cap, and
  temp-dir isolation (Job Objects on Windows). User code never runs in the WebView.
- **Offline test-pack judging** — 2,900+ verified packs frozen into
  `src-tauri/resources/test-packs.json.gz`. Packs carry reference solutions plus an independent
  brute-force oracle; the offline build computes expected outputs by *executing* the references and
  cross-checking Python vs JavaScript vs the oracle, so a wrong solution can never be frozen.
- **Name-agnostic catalog loader** — any `catalog*.json` / `catalog*.json.gz` in
  `src-tauri/resources/` is discovered at startup, loaded, merged (de-duplicated by slug), and mapped
  to its frozen test pack (the matched pack becomes the hidden judge). Multiple catalogs coexist and
  either can be swapped in with no code change.
- **Open-source project docs** — `README.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`,
  `DISCLAIMER.md` (content & legal policy), `NOTICE`, and this changelog.

### Changed

- The dev-only LeetCode scrape moved from `.docs/my_questions.json` to
  `src-tauri/resources/catalog_leetcode.json` and is hard-ignored (`*leetcode*`). An original,
  redistributable catalog may be committed normally.

### Notes

- No release has been published yet — the project is in active `0.1.0` development.

[Unreleased]: https://github.com/kudzaiprichard/anvil/commits/main
