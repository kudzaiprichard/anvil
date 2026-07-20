# Content, Licensing & Legal

This document explains **what Anvil ships, what it deliberately does not ship, and your
responsibilities** when you supply your own problem content. It is part of the project's commitment to
staying legally clean for every user and contributor.

> **This is not legal advice.** If you are unsure whether you may use a particular content source,
> read that source's Terms of Service and, if needed, consult a lawyer.

---

## 1. What this repository licenses to you (MIT)

Everything Anvil authors is original work, released under the [MIT License](./LICENSE) © Kudzai P
Matizirofa. That includes:

- **All source code** — the Tauri/Rust backend, the Next.js/React frontend, and the tooling in
  `tools/`.
- **The test packs** — `tools/packs/*.json` and the frozen bundle
  `src-tauri/resources/test-packs.json.gz`. Each pack is original content: reference solutions, an
  independent brute-force oracle, input generators, constraints, and hints. Packs contain **test data
  and code, not problem statements.**

You may use, modify, and redistribute all of the above under the MIT terms.

## 2. What this repository does **not** contain

Anvil ships with **no third-party problem statements of any kind** — and specifically **no LeetCode
content**: no problem text, titles-as-authored, examples, editorials, or hidden test cases taken from
any external platform.

Out of the box the application's problem library is therefore **empty**. It becomes populated only
when *you* provide a catalog (see below).

## 3. The problem catalog is user-supplied ("bring your own")

Anvil's loader picks up **any** file named `catalog*.json` (or `catalog*.json.gz`) placed in
`src-tauri/resources/` and maps each entry to a frozen test pack by slug. That is the *only* way
statements enter the app.

Anvil does **not**:

- bundle, distribute, or download a catalog;
- include a scraper or any tool that fetches problems from a third-party site;
- endorse obtaining content in violation of any provider's terms.

The catalog file is **your local data**, on your machine, under your control.

## 4. Your responsibilities

You are **solely responsible** for ensuring that any problem statements you load are content you have
the right to use.

- Problem statements on platforms such as **LeetCode** are **copyrighted by those platforms**, and
  their Terms of Service commonly **prohibit scraping, bulk export, and redistribution.**
- Those platforms let *individuals* access problems for their **own** practice. That permission does
  **not** extend to a company, competition, or product reusing or redistributing the content. Anvil is
  a shell for your personal practice — it is **not** a way to repackage or ship anyone's problem set.
- Any catalog you assemble from such sources is for your **personal, individual, offline use only.** Do
  **not** redistribute it, commit it to a fork, publish it, or include it in a build you share.
- To help prevent accidents, this repository's [`.gitignore`](./.gitignore) hard-ignores any catalog
  file whose name contains `leetcode` (`/src-tauri/resources/*leetcode*.json[.gz]`). An **original,
  redistributable** catalog you author yourself may be committed normally.

If you cannot use a source's content within its terms, don't load it.

## 5. Contributed content must be original

Anvil does **not** accept problem statements — those are always your local, bring-your-own data and are
never committed here. What you *do* contribute to this repository — **test packs** (reference
solutions, oracles, generators, hints) and **lesson prose** — must be **100% original**. You may
reference a problem by slug and reuse an algorithm, technique, or well-known problem *name*, but never
copy another author's exact wording, examples, constraints, editorials, or test cases into a pack or a
lesson. This is the rule that keeps everything Anvil ships legally clean. See the non-negotiable rule in
[CONTRIBUTING.md](./CONTRIBUTING.md#contributing-content--the-one-non-negotiable-rule).

## 6. Trademarks & affiliation

"LeetCode", "NeetCode", and other product names are trademarks of their respective owners. They are
referenced here only for identification and compatibility. Anvil is **not affiliated with, endorsed
by, or sponsored by** any of them.

## 7. No warranty

The software is provided "as is", without warranty of any kind, as stated in the [LICENSE](./LICENSE).
The authors accept no liability for how you obtain, store, or use problem content.

---

_Questions about this policy? Open a
[Discussion](https://github.com/kudzaiprichard/anvil/discussions) or email the maintainer at
kudzaiprichard@gmail.com._
