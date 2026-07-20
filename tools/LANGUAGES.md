# Pack languages — registry & how to add one

Test packs are **language-scalable by design**. One pack splits into an **agnostic
core** (judge type, test inputs, the
*computed* expected outputs, hints, pattern, constraints, stress generators) and a
**`solutions` map keyed by language**. Adding a language is *additive*: drop in one
solution per problem and re-verify — the core is never touched.

## The load-bearing rule

`python` is the **source of truth**: every `expected` value is produced by executing
the python reference through Anvil's own sandbox harness
(`generate_test_packs.compute_expected`). It is *never* hand-typed. Every **other**
language is verified **by agreement** — it runs the *same stored inputs* and must
produce the *same stored expecteds* under the problem's judge semantics. A new
language therefore adds zero new ground truth; it can only confirm or fail.

## Supported languages

Registry lives in `tools/build_packs.py` (`LANGUAGES`). The build requires the
`runnable: True` core languages on every pack and verifies them; a declared but
not-yet-runnable language fails the pack closed (never silently "verified").

| Language | `solutions` key | Runnable | Role |
|---|---|---|---|
| Python | `python` | ✅ | source of truth (expecteds computed here) |
| JavaScript | `javascript` | ✅ | verified by agreement |

`python` + `javascript` are cross-checked inside `verify_and_build` (the proven
core). Any *additional* `solutions.<lang>` key is cross-checked by
`build_packs.cross_check_extra_languages` against the already-computed expecteds.

## Adding a language (checklist)

Two parts: **one-time runtime infra** (per language), then **one solution per
problem** (per pack).

**One-time (not per problem):**
1. Add a sandbox + harness for the language under
   `src-tauri/src/services/runner/` (mirror `harness/harness.py` / `harness.js`:
   the sentinel-line protocol, entry-point resolution, the judge modes).
2. Add a `Language` variant in `src-tauri/src/domain/run.rs` and wire it through
   the runner.
3. Teach `generate_test_packs.run_harness` to launch the new interpreter (the
   file names, suffix shim, and program), so the offline build can execute it.
4. Add the language to `LANGUAGES` in `build_packs.py` with `runnable: True` and
   define its entry-point convention (how `entry_point.<lang>` names a callable).

4b. **Add the toolchain to CI** (`.github/workflows/`) so verification runs there
   too — the build can only verify a language it can execute.

**Per problem (additive, never edits the core):**
5. Add `solutions.<lang>` to each `tools/packs/<slug>.json`. At scale this is a
   **Sonnet subagent generation pass** over the frozen packs (one new solution per
   problem), orchestrated per `tools/packs/ORCHESTRATION.md` and verified by
   execution — the same wave/sweep/freeze loop used for new packs.
6. Re-run `python tools/build_packs.py`. The build keeps the frozen agnostic core
   (same `source_hash` ⇒ the pack is *not* rebuilt unless `--allow-refreeze`), runs
   the new language on the stored inputs, and records it in
   `index.json`'s `verified_langs` only if it agrees with the computed expecteds.

Because step 6 only *reads* the stored expecteds and *adds* a `verified_langs`
entry, shipping a new language for an existing pack cannot alter that pack's tests
or any other language — the immutability guarantee.

**Cost curve:** TypeScript ≈ free (rides the JS harness). Compiled languages
(Java, C++, Go, Rust, Swift, Kotlin, C#) each ≈ one harness + one runner + one
generation pass. The current `.docs/my_questions.json` already carries stubs for
python, javascript, typescript, java, cpp, csharp, go, kotlin, rust, swift, so the
`function_signature` inputs are ready when each language is turned on.

**Operational note:** each harness (`harness.py`/`harness.js`) is embedded into the
Rust runner via `include_str!`, and `src-tauri/src/` is **untracked in git** — never
run `git clean`/worktree-remove (it deletes the harness unrecoverably). If a source
copy is lost, the harness can be extracted from a built `anvil.exe` (contiguous
UTF-8 blob from the `// Anvil test harness` marker).

Because step 6 only *reads* the stored expecteds and *adds* a `verified_langs`
entry, shipping a new language for an existing pack cannot alter that pack's tests
or any other language — which is exactly the immutability guarantee.
