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

> **Rehearse with `--dry-run` first.** Before any real cut, run `node release.mjs <type> --dry-run`: it
> runs every pre-flight gate (tests, build, boundary, content, version/tag checks) and **changes
> nothing** — no bump, no commit, no tag. It's the cheap way to see a green (or failing) release before
> committing to one, and it prints the exact `CHANGELOG` section that will ship.

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

## Release integrity — signed commits & tags

Both the version-bump **commit** and the `vX.Y.Z` **tag** a release produces are cryptographically
signed and verified as the maintainer's, and repository rulesets **require** it:

- The `main` branch ruleset and the `v*` tag ruleset both carry a **`required_signatures`** rule — every
  commit on `main` and every release tag must have a verified signature.
- The `v*` tag ruleset also restricts tag **creation/update/deletion to repository admins**, so only a
  maintainer can cut a release tag (see [Who can cut a release](#who-can-cut-a-release--and-how-contributor-changes-ship)).
- The maintainer is a branch-protection **bypass actor**, so `release.mjs` can push the bump commit and
  tag directly — and because the maintainer signs, everything pushed there is signed anyway.

Contributors are unaffected: they push to feature branches (not covered by these rulesets), and a
squash-merge lands a single **GitHub-signed** (verified) commit on `main`.

### One-time setup: SSH commit signing (maintainer machine)

`release.mjs` runs non-interactively, so sign with an SSH key that has **no passphrase** (or one loaded
in an agent) — otherwise every release commit/tag would prompt.

```bash
# 1. Tell git to sign commits + tags with your SSH key
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true

# 2. (optional) let `git verify-commit`/`verify-tag` resolve your own signatures locally
printf '%s %s\n' "$(git config user.email)" "$(cat ~/.ssh/id_ed25519.pub)" > ~/.ssh/allowed_signers
git config --global gpg.ssh.allowedSignersFile ~/.ssh/allowed_signers

# 3. Register the PUBLIC key on GitHub as a **Signing key** (a signing key is separate
#    from an authentication key — the same key can be both, but must be added twice):
gh auth refresh -h github.com -s admin:ssh_signing_key   # grant the scope (interactive), then:
gh ssh-key add ~/.ssh/id_ed25519.pub --type signing --title "anvil release signing"
#    …or via the web UI: Settings → SSH and GPG keys → New SSH key → Key type: "Signing Key".
```

Confirm the whole chain works — a signed commit that GitHub itself verifies:

```bash
git verify-commit HEAD    # local → "Good git signature"
gh api "repos/kudzaiprichard/anvil/commits/$(git rev-parse HEAD)" --jq '.commit.verification'
# → { "verified": true, "reason": "valid", … }
```

> **Why the release tag is annotated:** a signed tag is an *annotated* tag, which needs a message. With
> `tag.gpgsign=true`, a bare `git tag <name>` aborts with `fatal: no tag message?`, so `release.mjs`
> creates the tag as `git tag -m "Release vX.Y.Z" vX.Y.Z` — annotated (and therefore signable). If a
> machine has no signing configured, that still produces a valid *unsigned* annotated tag, so the
> release never breaks; it just isn't signed there.

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
      **This is now automatic** — `release.mjs` stamps it for you via `tools/prepare-release.mjs`
      (curated `[Unreleased]` notes win; otherwise the section is generated from the
      Conventional-Commit subjects since the last tag). Preview it any time with
      `npm run release:prepare -- <patch|minor|major> --dry-run`.
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
2. _(Optional.)_ If you want curated release notes instead of the auto-generated ones, write them under
   `## [Unreleased]` in `CHANGELOG.md` — they take precedence. Otherwise skip this: the section is
   generated from the commit history. Preview either way with
   `npm run release:prepare -- <patch|minor|major> --dry-run`.
3. Run `node release.mjs <patch|minor|major>` from a clean `main` — it runs the whole checklist above,
   stamps the dated `CHANGELOG.md` section, then bumps the version, commits (bump + changelog together),
   pushes, and pushes the `vX.Y.Z` tag that triggers the build.
4. The [`Release`](.github/workflows/release.yml) workflow runs a `create-release` job first, which
   creates the **draft** release on
   [**kudzaiprichard/anvil-releases**](https://github.com/kudzaiprichard/anvil-releases/releases) (empty,
   no assets yet) — not this repo. Once that lands, the 4 platform build jobs run in parallel, each
   building its installer, re-running the boundary gate (belt and suspenders), and uploading its own
   asset onto that one release.
5. Review the draft there once all 4 platform jobs finish, verify the attached installer sizes are at
   the no-scrape baseline, then publish.

### Who can cut a release — and how contributor changes ship

Cutting a release is **maintainer-only, by design** — not a limitation of the workflow, but a trust
boundary. `release.mjs` does two things a regular contributor cannot:

- it **pushes the version-bump commit straight to `main`**, a protected branch. Only a branch-protection
  **bypass actor** (the maintainer/admin) may push directly; a contributor's push is rejected;
- it **publishes signed installers** to `anvil-releases` via the `RELEASES_REPO_TOKEN` secret, which
  only the maintainer holds. Tagging a release puts binaries in front of users under the project's name,
  so it must stay with a trusted maintainer.

**Contributors don't tag releases — they land changes, and the maintainer bundles them.** Any change,
however big, ships the same way: fork → branch → PR → green CI → code-owner review → merge to `main`
(see [CONTRIBUTING.md](./CONTRIBUTING.md)). Once it's on `main` it's *in the next release* — whenever the
maintainer next runs `release.mjs`, which packages everything accumulated since the last tag into one
versioned build. A large or breaking change simply warrants a `minor`/`major` bump. If a contributor
needs a release cut for their merged work, they ask a maintainer (open an issue/discussion); they never
tag it themselves.

The `Release` workflow can also be re-run manually from **Actions → Release → Run workflow**, but only by
selecting an **existing tag** in the "Use workflow from" dropdown — dispatching it from a branch is
rejected by a guard step.

### Why release creation is a separate job

`create-release` exists only to call `gh release create` exactly once. Don't fold it back into the
platform matrix, even though that looks simpler — `anvil-releases` has no git tag/commit matching the
release's tag name, so `gh release create`'s "already exists" check doesn't reliably collide when several
matrix jobs call it concurrently. An earlier version had each platform job try `gh release create || gh
release upload`, and all 4 (sometimes more, across retries) independently "won" the create, producing
separate duplicate draft releases instead of one shared release with every platform's installer attached.
Serializing creation into its own job the matrix `needs:` closes that race by construction: there is
exactly one `create` call in the whole workflow, and every platform job's publish step is upload-only.
