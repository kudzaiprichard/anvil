<!-- BEGIN:nextjs-agent-rules -->
# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` before writing any code. Heed deprecation notices.
<!-- END:nextjs-agent-rules -->

# Git, commits & PR workflow

`main` is protected — every change lands via a pull request. See `CONTRIBUTING.md` for the full rules.
For any agent working in this repo:

- **Commits (only when explicitly asked):** format is `type(scope): summary` followed by **one
  `- path: what changed` bullet per file in the commit** — a commit hook enforces this. Group related
  files into one logical commit. Never add Claude/Anthropic attribution, `Co-Authored-By`, or any
  "Generated with…" trailer.
- **Branches & PRs:** do **not** push straight to `main`. Branch (`feat/…`, `fix/…`, `docs/…`,
  `chore/…`) → commit → `gh pr create --fill` → wait for green CI → maintainer approval →
  `gh pr merge --squash --delete-branch`.
- A PR merges only after the required CI checks pass **and** it has 1 approving review from the code
  owner (`@kudzaiprichard`). The repo admin (maintainer) is a bypass actor and may self-merge their own
  PRs via "merge without waiting for requirements".
- Never force-push, `reset --hard`, or rewrite `main`'s history. Never weaken or disable the branch
  protection ruleset unless the maintainer explicitly asks.
