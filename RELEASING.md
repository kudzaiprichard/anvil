# Releasing Anvil

Anvil ships as offline desktop installers (Linux, macOS Apple-Silicon + Intel, Windows), built in this
repo by the [`Release`](.github/workflows/release.yml) workflow but **published to the separate
[kudzaiprichard/anvil-releases](https://github.com/kudzaiprichard/anvil-releases) repo**, not this repo's
own Releases tab. This is the maintainer runbook. Every release must clear the **shipping boundary
checklist** first — it is the project's one inviolable safety rule.

## Cutting a release (the normal path)

```bash
node release.mjs patch   # or minor / major
node release.mjs minor --dry-run    # rehearse: every gate, nothing changed
node release.mjs patch --no-watch   # skip the live build monitor at the end
```

`release.mjs` is the only thing that should ever cut a release — see the comment at the top of the file
for exactly what it checks (repo state, a fresh unused tag, a dated `CHANGELOG.md` heading for the
target version, Tauri version alignment, the shipped-content gates, the boundary gate, lint, types,
curriculum validity, the frontend build + a non-empty export, and the full Rust test suite) before it
bumps `package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml`, commits, pushes, and pushes
the `vX.Y.Z` tag that triggers the `Release` workflow. If any check fails, nothing is touched — no
half-bumped version, no stray commit, no tag. Checks run cheapest-and-most-fatal first, so a doomed
release dies in milliseconds, not after a multi-minute test run.

After the tag push the script **watches the Release workflow live from the terminal** (needs the `gh`
CLI): the create-release job and all four platform builds, updating in place until they finish, ending
with the draft-release link on success or the failed-job command on failure. Ctrl-C during the watch is
safe — the release is already cut by then; the monitor is purely observational.

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
- [ ] The target tag `vX.Y.Z` does not already exist, locally or on origin.
- [ ] `CHANGELOG.md` has a dated `## [X.Y.Z] - YYYY-MM-DD` heading for the version being tagged.
- [ ] `@tauri-apps/api` (npm) and the `tauri` crate (Rust) are on the same major.minor.
- [ ] **No empty shell**: `test-packs.json.gz` parses, is non-empty, every pack is `verified`, and the
      pack count equals the freeze manifest (`tools/packs/index.json`); the curriculum and lessons
      resource directories exist and are non-empty.
- [ ] `python tools/check_release_boundary.py` passes (no `*leetcode*` bundled; payload at baseline).
- [ ] `RELEASES_REPO_TOKEN` is set on the repo (warn-only — checked via `gh` when available).
- [ ] `npm run lint` passes.
- [ ] `npx tsc --noEmit` passes.
- [ ] `python tools/build_curriculum.py --check` passes (curriculum content is valid, fail-closed).
- [ ] `npm run build` passes, and the `out/` export actually contains the app (index.html + assets).
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
4. The [`Release`](.github/workflows/release.yml) workflow runs a `create-release` job first, which
   creates the **draft** release on
   [**kudzaiprichard/anvil-releases**](https://github.com/kudzaiprichard/anvil-releases/releases) (empty,
   no assets yet) — not this repo. Once that lands, the 4 platform build jobs run in parallel, each
   building its installer, re-running the boundary gate (belt and suspenders), and uploading its own
   asset onto that one release.
5. Review the draft there once all 4 platform jobs finish, verify the attached installer sizes are at
   the no-scrape baseline, then publish.

> Only a maintainer with push access can run `release.mjs` (it pushes to `main` directly) and publish the
> draft. The `Release` workflow can also be re-run manually from **Actions → Release → Run workflow**,
> but only by selecting an **existing tag** in the "Use workflow from" dropdown — dispatching it from a
> branch is rejected by a guard step.

### Why release creation is a separate job

`create-release` exists only to call `gh release create` exactly once. Don't fold it back into the
platform matrix, even though that looks simpler — `anvil-releases` has no git tag/commit matching the
release's tag name, so `gh release create`'s "already exists" check doesn't reliably collide when several
matrix jobs call it concurrently. An earlier version had each platform job try `gh release create || gh
release upload`, and all 4 (sometimes more, across retries) independently "won" the create, producing
separate duplicate draft releases instead of one shared release with every platform's installer attached.
Serializing creation into its own job the matrix `needs:` closes that race by construction: there is
exactly one `create` call in the whole workflow, and every platform job's publish step is upload-only.
