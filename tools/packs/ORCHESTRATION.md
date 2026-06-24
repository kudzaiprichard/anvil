# Pack-authoring orchestration — single session, multiple Sonnet agents

Replaces the old multi-session parallel plan. We author with **ONE Claude Code
session driving many subagents**, on **Sonnet** (not Opus), in **throttled waves**
sized to avoid the 5-hour session-window limit. This file is the operating procedure.

## Why this shape

- **Single session** = no cross-session lock contention, no per-session progress
  files, one consistent working tree. The freeze lock (`tools/packs/.lock`) is still
  used as a simple guard but only one session ever holds it.
- **Sonnet, not Opus** = ~right quality for these algorithmic packs at a fraction of
  the quota; the execution verifier (`--check`) catches the rare wrong one.
- **Throttled waves** = dispatch ~5 subagents at a time (each ~13 slugs), freeze,
  repeat. Firing many agents at once spikes the session window and stalls everything.
  ~5 concurrent keeps it sustainable.
- **Local-model generation was tried and abandoned** — on a 6 GB GPU even
  `qwen2.5-coder:14b` ran ~15–21 min/pack (CPU spillover) with poor yield. Sonnet
  subagents are the chosen path. (`tools/auto_author.py` was removed.)

## The wave loop

1. **Pick the target set** (a batch's missing authorable slugs, or a curated list).
   Compute "missing" = authorable AND not in `tools/packs/index.json` AND no file on
   disk. Exclude basic-mode (SQL/interactive/random/Tier-B) — see CONTENT_PIPELINE §5.
2. **Slice** into ~13-slug chunks, grouped by type (plain / node / design).
3. **Dispatch ~5 Sonnet subagents** (`subagent_type: general-purpose`,
   `model: "sonnet"`), each pointed at the shared brief and its slug list. The brief
   forbids `git`, requires stdlib-only solutions, a `kind` on every constraint, and
   self-checking via `--check` before reporting.
4. **As agents finish**, expect some to drop on transient "Connection closed" API
   errors or the session limit — **trust the disk, not the report.**
5. **Sweep:** list which target slugs have files on disk; `--check` them all.
   - Passing → keep.
   - Quarantined / invalid-JSON (half-written by a dropped agent) → **delete the
     file** and add the slug to the next wave's redo list.
   - Missing (agent died before writing) → add to the redo list.
6. **Freeze under the lock** (single session, so trivially acquired):
   ```bash
   mkdir tools/packs/.lock
   python tools/build_packs.py --bundle && ( cd src-tauri && cargo test ) && npm run build
   rmdir tools/packs/.lock
   ```
   Never advance on a red gate.
7. **Repeat** with the next slice + the redo list folded in, until the target set is
   `--check`-green and frozen.

## Hard rules (enforced in the subagent brief)

- **NEVER run `git`** (no worktree/clean/checkout/reset) — `src-tauri/src/` and
  `tools/packs/` are untracked; a stray git op deletes them. (This is how the runner
  harness was once lost — see CONTENT_PIPELINE §8.)
- Only create/edit files under `tools/packs/`.
- **Stdlib only** in solutions (no third-party packages).
- **Never hand-type expected values; all prose original** (no LeetCode text copied).
- Every constraint has a `kind`; `len`/`value` are 2-element integer arrays.
- Defer rather than ship wrong — Tier-B/uncertain → basic-mode with a reason.

## Sizing to avoid the limit

- ~5 subagents per wave; each ~13 slugs → ~65 packs/wave.
- Freeze between waves so progress banks; if the window limit hits mid-wave, the
  disk sweep recovers whatever landed and the rest becomes the next wave's redo.
- A wave's wall-clock is ~10–18 min on Sonnet; budget accordingly across the window.

## Status / continuation

Live counts, what's done, what's left, and how to resume are tracked in
`.docs/tasks/content/STATUS.md`. The reusable subagent instructions live in the
authoring brief referenced there + `tools/PACK_AUTHORING_GUIDE.md`.
