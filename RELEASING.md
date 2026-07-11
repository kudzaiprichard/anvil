# Releasing Anvil

Anvil ships as offline desktop installers (Linux, macOS Apple-Silicon + Intel, Windows) built by the
[`Release`](.github/workflows/release.yml) workflow. This is the maintainer runbook. Every release must
clear the **shipping boundary checklist** first — it is the project's one inviolable safety rule.

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

- [ ] `python tools/check_release_boundary.py` passes (no `*leetcode*` bundled; payload at baseline).
- [ ] `python tools/build_curriculum.py --check` passes (curriculum content is valid, fail-closed).
- [ ] `npm run lint && npm run build` pass.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` passes.
- [ ] A clean install shows the course working with **user-supplied statements only** (empty library
      until the user brings a catalog; lessons, gates, quizzes, diagrams, and review all function
      offline).
- [ ] `CHANGELOG.md` has a dated section for the version being tagged.
- [ ] `version` is in sync across `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.

## Cutting the release

1. Land all release-blocking PRs on `main` (green CI + code-owner approval).
2. Confirm the checklist above.
3. Tag `main` and push — the tag push is what triggers the build matrix and drafts the GitHub Release:

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. The [`Release`](.github/workflows/release.yml) workflow builds installers for every platform, runs
   the boundary gate, and attaches the artifacts to a **draft** GitHub Release.
5. Review the draft, verify the attached installer sizes are at the no-scrape baseline, then publish.

> Only a maintainer with push access can tag and publish. The workflow can also be started manually from
> **Actions → Release → Run workflow** (`workflow_dispatch`).
