# Contributing to Anvil

Thanks for your interest in contributing! Anvil is an offline-first desktop app for DSA / coding-interview
practice, and there are many ways to help — code, original practice problems, docs, design, and translations.

By participating you agree to abide by our [Code of Conduct](./CODE_OF_CONDUCT.md).

## Ways to contribute

- **Code** — the desktop shell (Tauri/Rust), the UI (Next.js/React), the code runner, and tooling.
- **Problems** — author original DSA problems and test cases (no coding required — just Markdown + JSON).
- **Docs, design, accessibility, translations** — all welcome.

## Development setup

See the [prerequisites and getting-started steps in the README](./README.md#prerequisites). In short:

```bash
npm install
npm run dev          # web preview at http://localhost:3000
npm run tauri dev    # run the desktop app
```

Before opening a pull request, make sure these pass:

```bash
npm run lint
npm run build
```

## Pull requests

1. Fork the repo and create a feature branch (`feat/...`, `fix/...`, `docs/...`).
2. Keep PRs small and focused; reference any related issue (`Closes #123`).
3. Use [Conventional Commits](https://www.conventionalcommits.org) for commit messages
   (`feat:`, `fix:`, `refactor:`, `chore:`, `docs:`, …).
4. Ensure `npm run lint` and `npm run build` pass.

### Branch protection & merge rules

`main` is protected — **all** changes land through a pull request, including the maintainer's own. Before
a PR can merge it must:

- pass CI (lint + build on web, and Rust fmt/clippy/test on Linux & Windows), and
- have **1 approving review from the maintainer** (`@kudzaiprichard`, our code owner — see
  [`.github/CODEOWNERS`](./.github/CODEOWNERS)).

Approvals are dismissed when new commits are pushed, so re-request review after changes. Force-pushes and
deletion of `main` are blocked. In short: fork → branch → PR → green CI → maintainer approval → merge.

## Contributing problems — the one non-negotiable rule

> **You may copy the IDEA. You may NOT copy the EXPRESSION.**

All problem content must be **100% original**:

- ✅ You may reuse algorithms, techniques, problem *types*, and famous *names* (e.g. "LRU Cache").
- ❌ You may **not** copy anyone's exact problem **wording, examples, constraints, editorials, or test
  cases** — including from LeetCode/NeetCode — not even as a "starting point."
- By submitting a problem you **warrant that the content is original or that you have the right to
  share it.**

LLM-assisted drafting is allowed only **from the abstract concept** ("write an original problem teaching
the monotonic-stack technique"), never "rewrite this problem." This keeps the project legally clean for
everyone. For the full content & licensing policy — including how user-supplied (non-shipped) catalogs
work — see [DISCLAIMER.md](./DISCLAIMER.md).

A contributed problem is two original parts: a **statement** (an entry in an original `catalog.json`)
and a **test pack** (`tools/packs/<slug>.json` — reference solution, brute-force oracle, generators,
hints). The build (`tools/build_packs.py`) computes expected outputs by *executing* your references in
the sandbox and cross-checking them; a pack whose solutions disagree is quarantined, never frozen.

## Contributing course content (lessons)

Anvil is a **guided course**, not just a problem bank. The course is **data**, not code:
schemas and the loader/engine are compiled, but every stage, unit, lesson, quiz, and diagram is an
editable resource file. Fixing a typo, reordering lessons, swapping a problem slug, tuning a gate
threshold, or adding a quiz is a **content PR** — no Rust/TS change, no recompile.

The content lives under `src-tauri/resources/`:

```
curriculum/
  curriculum.json                  # stages, unit order, prereq DAG, gate defaults, capstone
  units/<unit>.json                # a unit: its lessons, problems (worked/guided/gate), gate, spiral
  pattern-pool.json                # cross-unit interleaved pattern-picker items
lessons/<unit>/
  NN-<subpattern>.md               # explainer prose (YAML frontmatter + Markdown body)
  NN-<subpattern>.quiz.json        # concept-check / complexity / pattern-picker items
  NN-<subpattern>.diagram.json     # prediction-diagram steps + a "what happens next?" pause
```

Each lesson must carry all of: an `explainer` body, `trigger_signals`, a `worked_example` slug, a
prediction `diagram`, a `quiz`, and `practice` slugs — the loader (and the check below) reject a lesson
that is missing any part. A referenced problem is just a LeetCode **slug**; its frozen test pack must
already exist so judging works the moment a learner supplies the statement.

**Validate before you open a PR** — this mirrors, in Python, exactly what the app enforces
fail-closed at startup (DAG has no cycles, every referenced slug has a frozen pack, every lesson has
its required parts, quiz answers ∈ options, diagram indices in range):

```bash
python tools/build_curriculum.py --check     # non-zero exit ⇒ don't ship
```

The **same originality rule as problems applies**: reference LeetCode problems by slug and narrate the
*technique in your own words* — never paste a platform's problem statement, examples, or editorial into
a lesson. See [DISCLAIMER.md](./DISCLAIMER.md).

## Reporting bugs & requesting features

Use the [issue templates](https://github.com/kudzaiprichard/anvil/issues/new/choose). For security
issues, follow [SECURITY.md](./SECURITY.md) instead of opening a public issue.

## Questions

Open a [Discussion](https://github.com/kudzaiprichard/anvil/discussions) — happy to help.
