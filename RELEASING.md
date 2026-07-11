# Releasing Anvil

Anvil ships as offline desktop installers (Linux, macOS Apple-Silicon + Intel, Windows), built in this
repo by the [`Release`](.github/workflows/release.yml) workflow but **published to the separate
[kudzaiprichard/anvil-releases](https://github.com/kudzaiprichard/anvil-releases) repo**, not this repo's
own Releases tab. This is the maintainer runbook. Every release must clear the **shipping boundary
checklist** first — it is the project's one inviolable safety rule.

## Cutting a release (the normal path)

```bash
node release.mjs patch   # or minor / major
```

`release.mjs` is the only thing that should ever cut a release — see the comment at the top of the file
for exactly what it checks (repo state, Tauri version alignment, a `CHANGELOG.md` entry for the target
version, lint, types, curriculum validity, the boundary gate, the frontend build, and the full Rust test
suite) before it bumps `package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml`, commits,
pushes, and pushes the `vX.Y.Z` tag that triggers the `Release` workflow. If any check fails, nothing is
touched — no half-bumped version, no stray commit, no tag.

### One-time setup: the cross-repo publish token

Publishing to `anvil-releases` from a workflow running in `anvil` needs a token scoped to the *other*
repo — the default `GITHUB_TOKEN` Actions provides is scoped only to the repo the workflow runs in.
Set this up once:

1. [github.com/settings/tokens?type=beta](https://github.com/settings/tokens?type=beta) → generate a
   **fine-grained personal access token** scoped to **only** `kudzaiprichard/anvil-releases`, with
   repository permission **Contents: Read and write**.
2. `gh secret set RELEASES_REPO_TOKEN --repo kudzaiprichard/anvil` and paste the token when prompted.
   Never commit this token or paste it anywhere else — the `gh secret set` command sends it straight to
   GitHub's encrypted secret store for the `anvil` repo.

Without this secret set, the `Release` workflow's build steps still succeed, but the final "publish to
anvil-releases" step will fail (no installers get uploaded anywhere).

## The shipping boundary (never violate)

Public installers ship **only** the app, the frozen verified test packs, and the lessons — and **never**
any third-party problem statements, and specifically **no `*leetcode*` catalog** (see
[DISCLAIMER.md](./DISCLAIMER.md) and the blueprint). A disclaimer is not a license. The dev scrape
(`src-tauri/resources/catalog/catalog_leetcode.json`) is gitignored, so a clean checkout — CI, or a
fresh clone — never has it; but the bundler copies the whole `resources/catalog/` directory, so a
release built on a developer machine that still holds the scrape *would* sweep it in.

This is enforced automatically, fail-closed, so it can't happen by accident:

```bash
python tools/check_release_boundary.py
```

It fails the build if **any `*leetcode*` file would be bundled**, or if the total bundled-resource
payload exceeds the **no-scrape baseline** (~4.3 MB clean; the scrape alone is ~20 MB), which catches a
differently-named bulk statement dump too. The gate runs automatically in
[`ci.yml`](.github/workflows/ci.yml) (on every `main` PR), [`build.yml`](.github/workflows/build.yml),
and [`release.yml`](.github/workflows/release.yml) **before** the bundle step.

### Pre-release checklist

`node release.mjs <patch|minor|major>` checks every item below itself, in order, before touching any
file — this list is what it enforces, kept here as the reference (and as the manual fallback if you ever
need to release without the script):

- [ ] On a clean, up-to-date `main`.
- [ ] `@tauri-apps/api` (npm) and the `tauri` crate (Rust) are on the same major.minor.
- [ ] `CHANGELOG.md` has a dated section for the version being tagged.
- [ ] `npm run lint` passes.
- [ ] `npx tsc --noEmit` passes.
- [ ] `python tools/build_curriculum.py --check` passes (curriculum content is valid, fail-closed).
- [ ] `python tools/check_release_boundary.py` passes (no `*leetcode*` bundled; payload at baseline).
- [ ] `npm run build` passes.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` passes.
- [ ] `version` is in sync across `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`
      (release.mjs bumps all three together, atomically).
- [ ] A clean install shows the course working with **user-supplied statements only** (empty library
      until the user brings a catalog; lessons, gates, quizzes, diagrams, and review all function
      offline) — this one is manual, do it once per release, not on every patch.

## Cutting the release

1. Land all release-blocking PRs on `main` (green CI + code-owner approval).
2. Make sure `CHANGELOG.md` has a dated `[X.Y.Z]` section for the version you're about to cut.
3. Run `node release.mjs <patch|minor|major>` from a clean `main` — it runs the whole checklist above,
   then bumps the version, commits, pushes, and pushes the `vX.Y.Z` tag that triggers the build.
4. The [`Release`](.github/workflows/release.yml) workflow builds installers for every platform, runs
   the boundary gate again (belt and suspenders), and publishes them as a **draft** release on
   [**kudzaiprichard/anvil-releases**](https://github.com/kudzaiprichard/anvil-releases/releases) — not
   this repo.
5. Review the draft there, verify the attached installer sizes are at the no-scrape baseline, then
   publish.

> Only a maintainer with push access can run `release.mjs` (it pushes to `main` directly) and publish the
> draft. The `Release` workflow can also be re-run manually from **Actions → Release → Run workflow**,
> but only by selecting an **existing tag** in the "Use workflow from" dropdown — dispatching it from a
> branch is rejected by a guard step.
