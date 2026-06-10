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
everyone.

## Reporting bugs & requesting features

Use the [issue templates](https://github.com/kudzaiprichard/anvil/issues/new/choose). For security
issues, follow [SECURITY.md](./SECURITY.md) instead of opening a public issue.

## Questions

Open a [Discussion](https://github.com/kudzaiprichard/anvil/discussions) — happy to help.
