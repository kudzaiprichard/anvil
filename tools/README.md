# Anvil dev tools

Developer-only scripts. **None of these ship in the app** and none write
scraped statement text into a committed file. See `.docs/CONTENT_DESIGN.md`
and `.docs/CONTENT_MAP.md` for the legal and engineering context.

| Script | Purpose |
|---|---|
| `build_packs.py` | **Keyless build (task 0001).** Compiles the hand-authored source `packs/<slug>.json` into the verified bundle + manifest. No model, no API key. This is the path used for authoring batches (task 0002). |
| `generate_test_packs.py` | The verification engine + the (optional, API-backed) generation pipeline. `build_packs.py` reuses its `verify_and_build`; `--self-test` exercises the machinery offline. |
| `build_fixture_bundle.py` | Gzips a `test-packs.json` into the shipped `src-tauri/resources/test-packs.json.gz`. |
| `LANGUAGES.md` | Supported-language registry + the "add a language" checklist. |
| `check_preset_slugs.py` | Re-validates / regenerates the Blind 75 + NeetCode 150 preset slug lists. |
| `lc_scraper.py` | *(separate, personal-use only — not in this repo)* the external tool a user runs against their own LeetCode account to produce `my_questions.json`. |

---

## `build_packs.py` (the authoring path)

Hand-author one source file per problem in `tools/packs/<slug>.json` — the agnostic
core (`judge`, `pattern`, `hints`, `constraints`, `edge_inputs`, `stress`,
`oracle_python`) plus a `solutions` map keyed by language. **It carries no expected
values**; every `expected` is computed by executing the python reference through the
sandbox harness, exactly as the API path does (the one rule, below). Then:

```bash
python tools/build_packs.py --batch 0 --bundle    # build + gzip, no API key
python tools/build_packs.py --only two-sum         # one slug
python tools/build_packs.py --allow-refreeze ...   # intentionally re-verify a frozen pack
```

The build validates each source, runs the proven verifier, cross-checks any extra
languages by agreement, writes `tools/test-packs.json`, and updates the immutability
manifest `tools/packs/index.json`. A pack whose source hash is unchanged and already
bundled is **skipped (frozen)**; a changed hash on a bundled slug prints a loud
warning and is left frozen unless `--allow-refreeze`. See `LANGUAGES.md` for the
language registry. Edge inputs may be bare arg-lists or rich
`{kind, description, input}` entries (kind ∈ edge|boundary|trap).

## `generate_test_packs.py`

Reads a `my_questions.json` scrape and emits a `test-packs.json` keyed by slug.

### The one rule

**The AI never writes expected outputs.** It proposes inputs, reference
solutions, hints, and the pattern note; every `expected` value is computed by
*executing* a verified reference solution through Anvil's own sandbox harness
(`src-tauri/src/services/runner/harness/`). In the code this is enforced
structurally: `compute_expected()` is the only function that produces a value
stored in a pack's `expected` field, and it does so only by running code. A
question that fails any verification step ships with **no pack** (basic mode) —
never with an unverified one.

### Verification pipeline (per question)

1. **Anchor** — parse the statement's own `Example` blocks into ground-truth
   `(input, output)` pairs (Python mirror of `services/example_parse.rs`). No
   parseable example ⇒ no anchor ⇒ skipped.
2. **Classify** the judge type (`exact` / `unordered` / `float` / `in_place` /
   `any_valid` / `design`) — AI classification, mechanically sanity-checked
   against the stub shape and statement phrasing; disagreements are logged.
3. **Generate** optimal + brute-force Python and a JavaScript reference, plus
   the entry point. The optimal solution must reproduce every anchored example.
4. **Extract** structured constraints.
5. **Generate inputs** — AI edge cases + (light) mechanical boundaries + stress
   generator specs.
6. **Compute + cross-check** — `expected` = optimal Python via the harness; the
   brute force must agree on every literal input, and JavaScript must agree on
   everything. Any disagreement ⇒ quarantine.

### Run it

```bash
# Offline machinery check — NO API key, NO `anthropic` package needed.
# Reproduces the two-sum fixture's expected values by execution, cross-checks
# Python/JS, materializes the stress generator, and proves a deliberately-wrong
# solution is rejected by the anchor/cross-check.
python tools/generate_test_packs.py --self-test

# Real generation needs the Claude API. Read the `claude-api` skill first for
# the current model id / params (this pipeline uses claude-opus-4-8 + adaptive
# thinking and asks for JSON-only output).
pip install anthropic
export ANTHROPIC_API_KEY=sk-ant-...

# A single slug (handy for iterating on the prompt/verification):
python tools/generate_test_packs.py --only two-sum

# A small batch from the default scrape (.docs/my_questions.json):
python tools/generate_test_packs.py --limit 5

# Resume an interrupted run (skips slugs already in the output):
python tools/generate_test_packs.py --resume
```

### Flags

| Flag | Meaning |
|---|---|
| `--scrape PATH` | Scrape input (default `.docs/my_questions.json`). |
| `--out PATH` | Output pack file (default `tools/test-packs.json`). |
| `--only a,b,c` | Generate only these slugs. |
| `--limit N` | Cap the number of questions. |
| `--resume` | Skip slugs already present in `--out`; save every 10 either way. |
| `--model ID` | Generation model id (default `claude-opus-4-8`). |
| `--python` / `--node` | Interpreter paths for the harness (default: this Python, `node` on PATH). |
| `--self-test` | Offline machinery check; no API key. |

### Cost

~3,187 questions × ~2k tokens ≈ a one-time run in the low tens of dollars.
Generate **Blind 75 + NeetCode 150 first** (the pilot, task 0010) to prove the
judge taxonomy before the full run.

### From packs to the shipped bundle

```bash
python tools/build_fixture_bundle.py tools/test-packs.json
# -> writes src-tauri/resources/test-packs.json.gz (committed, generated)
```

The Rust side loads that gzip lazily on the first import (`services/pack_store.rs`).

---

## `lc_scraper.py` (personal-use framing)

The scraper is intentionally **not** part of the app and is documented as a
separate, personal-use-only tool. A user runs it against *their own* LeetCode
account to export *their own* questions to `my_questions.json`, which they then
import through Settings → Practice. The app contains zero scraping code and
never connects to LeetCode. `my_questions.json` is gitignored and never
committed, shipped, or redistributed (`.docs/CONTENT_MAP.md`).
