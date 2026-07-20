import { readFileSync, writeFileSync, unlinkSync, existsSync, readdirSync, statSync } from 'fs';
import { execSync } from 'child_process';
import { gunzipSync } from 'zlib';

// Anvil's release script (see RELEASING.md for the full runbook). Run from a
// clean `main`:
//
//   node release.mjs <patch|minor|major> [--dry-run] [--no-watch]
//
//   --dry-run   run every pre-flight gate, then stop — nothing is bumped,
//               committed, tagged, or pushed. Use it to rehearse a release.
//   --no-watch  skip the live build monitor after the tag push.
//
// This is the ONLY thing that should ever cut a release. A normal commit/push
// never triggers one: build.yml is workflow_dispatch-only, and release.yml
// only fires on a `v*` tag push (which is the last thing this script does) or
// a manual dispatch from an existing tag. Every check below runs BEFORE any
// file is touched or any git state changes — if anything fails, the repo is
// exactly as you left it: no half-bumped version, no stray commit, no tag.
// Checks are ordered cheapest-and-most-fatal first, so a doomed release dies
// in milliseconds, not after a multi-minute cargo test run.

const args = process.argv.slice(2);
const type = args[0];
const dryRun = args.includes('--dry-run');
const noWatch = args.includes('--no-watch');

if (!['patch', 'minor', 'major'].includes(type)) {
    console.error('\n  Usage: node release.mjs <patch|minor|major> [--dry-run] [--no-watch]\n');
    console.error('  patch  → bug fixes, small tweaks        (0.1.0 → 0.1.1)');
    console.error('  minor  → new features, UI changes       (0.1.0 → 0.2.0)');
    console.error('  major  → breaking changes, major update (0.1.0 → 1.0.0)\n');
    process.exit(1);
}

function fail(msg) {
    console.error(`\n  ❌ Pre-flight failed: ${msg}\n`);
    process.exit(1);
}

function run(label, cmd) {
    console.log(`\n  ⏳ ${label}...`);
    try {
        execSync(cmd, { stdio: 'inherit' });
        console.log(`  ✅ ${label}`);
    } catch {
        fail(label);
    }
}

function sh(cmd) {
    return execSync(cmd, { encoding: 'utf-8' }).trim();
}

function ghAvailable() {
    try {
        execSync('gh --version', { stdio: 'ignore' });
        return true;
    } catch {
        return false;
    }
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

// ─── Pre-flight checks ────────────────────────────────────────────────────────

// 0. Must be run from a clean, up-to-date `main` — releasing from a feature
//    branch or a stale/dirty checkout is how you ship the wrong thing.
console.log('\n  🔍 Checking repo state...');
{
    const branch = sh('git rev-parse --abbrev-ref HEAD');
    if (branch !== 'main') {
        fail(`must be run from "main", currently on "${branch}". Land your PR and switch: git switch main`);
    }
    if (sh('git status --porcelain') !== '') {
        fail('working tree is not clean. Commit or stash your changes first.');
    }
    execSync('git fetch origin main', { stdio: 'ignore' });
    const localHead = sh('git rev-parse HEAD');
    const remoteHead = sh('git rev-parse origin/main');
    if (localHead !== remoteHead) {
        fail(`local main (${localHead.slice(0, 9)}) is not in sync with origin/main (${remoteHead.slice(0, 9)}). Run: git pull origin main`);
    }
    console.log('  ✅ On a clean, up-to-date main');
}

// 1. Compute the target version now — several checks below need it.
const pkgPath = 'package.json';
const tauriPath = 'src-tauri/tauri.conf.json';
const cargoPath = 'src-tauri/Cargo.toml';

const pkg = JSON.parse(readFileSync(pkgPath, 'utf-8'));
const current = pkg.version;
const [major, minor, patch] = current.split('.').map(Number);

let next;
if (type === 'major') next = `${major + 1}.0.0`;
else if (type === 'minor') next = `${major}.${minor + 1}.0`;
else next = `${major}.${minor}.${patch + 1}`;

// Today's date (YYYY-MM-DD) — used to stamp the CHANGELOG section.
const today = new Date().toISOString().slice(0, 10);

console.log(`\n  Preparing release: ${current} → ${next}${dryRun ? '  (dry run)' : ''}`);

// 2. The tag must not already exist, locally or on origin. Without this
//    check the collision only surfaced at `git tag` — AFTER the version-bump
//    commit was already pushed to main, leaving a half-cut release.
console.log(`\n  🔍 Checking that v${next} is a fresh tag...`);
{
    if (sh(`git tag -l v${next}`) !== '') {
        fail(`tag v${next} already exists locally. Delete it (git tag -d v${next}) or pick a different bump.`);
    }
    if (sh(`git ls-remote --tags origin refs/tags/v${next}`) !== '') {
        fail(`tag v${next} already exists on origin — this version has already been released.`);
    }
    console.log(`  ✅ v${next} is unused`);
}

// 3. CHANGELOG.md is AUTO-MAINTAINED — no hand-remembered step. Below (in the
//    mutation phase) tools/prepare-release.mjs stamps a dated "## [X.Y.Z]"
//    section and commits it alongside the version bump: curated [Unreleased]
//    notes win if present, otherwise the section is generated from the
//    Conventional-Commit subjects since the last tag. Here we only PREVIEW it
//    (--dry-run touches nothing) so a doomed changelog surfaces before the slow
//    gates, and a real --dry-run release shows exactly what will ship.
console.log(`\n  🔍 Previewing the CHANGELOG [${next}] section...`);
try {
    execSync(`node tools/prepare-release.mjs ${type} --version ${next} --base v${current} --date ${today} --dry-run`, { stdio: 'inherit' });
} catch {
    fail('could not preview the CHANGELOG update (see the error above).');
}

// 4. Tauri version alignment — NPM @tauri-apps/api minor must match Rust tauri minor.
//    This is the class of issue that broke v0.6.0's first CI run.
console.log('\n  🔍 Checking Tauri package version alignment...');
{
    const lockRaw = JSON.parse(readFileSync('package-lock.json', 'utf-8'));
    const npmVersion = lockRaw.packages?.['node_modules/@tauri-apps/api']?.version;
    if (!npmVersion) fail('@tauri-apps/api not found in package-lock.json');

    const cargoToml = readFileSync(cargoPath, 'utf-8');
    const rustMatch = cargoToml.match(/^tauri\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"/m);
    if (!rustMatch) fail('tauri crate version not found in src-tauri/Cargo.toml');
    const rustVersion = rustMatch[1];

    // Compare major.minor only — Tauri's own check
    const [npmMajor, npmMinor] = npmVersion.split('.').map(Number);
    // Rust version may be a range like "2.10.0" or "^2.10"
    const rustClean = rustVersion.replace(/^\^|^~/, '');
    const [rustMajor, rustMinor] = rustClean.split('.').map(Number);

    if (npmMajor !== rustMajor || npmMinor !== rustMinor) {
        fail(
            `@tauri-apps/api (v${npmVersion}) and tauri Rust crate (v${rustVersion}) ` +
            `are on different minor versions (${npmMajor}.${npmMinor} vs ${rustMajor}.${rustMinor}). ` +
            `Align them before releasing.`
        );
    }
    console.log(`  ✅ Tauri versions aligned (NPM ${npmVersion} ↔ Rust ${rustVersion})`);
}

// 5. Never ship an empty shell — the content the installer exists to deliver
//    must actually be present and internally consistent BEFORE the slow
//    gates run. The pack bundle must parse, be non-empty, be fully verified,
//    and match the freeze manifest one-for-one (a build half-done or a
//    truncated gz shows up here, instantly). Curriculum/lesson resources
//    must exist and be non-empty (their semantic validity is checked by
//    build_curriculum.py --check further down).
console.log('\n  🔍 Checking shipped content (no empty shell)...');
{
    const gzPath = 'src-tauri/resources/test-packs.json.gz';
    if (!existsSync(gzPath)) fail(`${gzPath} is missing — there are no test packs to ship.`);
    let packs;
    try {
        packs = JSON.parse(gunzipSync(readFileSync(gzPath)).toString('utf-8'));
    } catch (e) {
        fail(`${gzPath} is corrupt (${e.message}) — rebuild it with: python tools/build_packs.py --bundle`);
    }
    const packCount = Object.keys(packs).length;
    if (packCount === 0) fail('the pack bundle is EMPTY — nothing to judge against. Rebuild it.');
    const unverified = Object.values(packs).filter((p) => p.verified !== true).length;
    if (unverified > 0) fail(`${unverified} pack(s) in the bundle are not verified — the freeze is corrupt.`);

    const manifest = JSON.parse(readFileSync('tools/packs/index.json', 'utf-8'));
    const manifestCount = Object.keys(manifest).length;
    if (packCount !== manifestCount) {
        fail(
            `pack bundle (${packCount}) and freeze manifest (${manifestCount}) disagree — ` +
            `the bundle is stale or half-built. Rebuild it with: python tools/build_packs.py --bundle`
        );
    }

    for (const dir of ['src-tauri/resources/curriculum', 'src-tauri/resources/lessons']) {
        if (!existsSync(dir) || readdirSync(dir).length === 0) {
            fail(`${dir} is missing or empty — the course content is not there to ship.`);
        }
    }
    console.log(`  ✅ ${packCount} verified packs (bundle = manifest), curriculum + lessons present`);
}

// 6. Installer boundary check — THE non-negotiable gate: a release must
//    never bundle a *leetcode* catalog, and the bundled
//    resource payload must stay at the no-scrape baseline. Runs early
//    because it's milliseconds, and on a dev machine that still holds the
//    scrape it is the most common (and most fatal) failure.
run(
    'Installer boundary check (no *leetcode* bundling)',
    'python tools/check_release_boundary.py'
);

// 7. The cross-repo publish token should exist before we build for 20
//    minutes only to fail at the upload step. Warn-only: `gh` may be absent
//    or offline, and the secret's VALUE can't be verified from here anyway.
if (ghAvailable()) {
    try {
        const secrets = sh('gh secret list --repo kudzaiprichard/anvil');
        if (!secrets.includes('RELEASES_REPO_TOKEN')) {
            console.warn(
                '\n  ⚠️  RELEASES_REPO_TOKEN is not set on kudzaiprichard/anvil — the platform ' +
                'builds will succeed but publishing to anvil-releases will fail. See RELEASING.md.'
            );
        } else {
            console.log('\n  ✅ RELEASES_REPO_TOKEN secret is set');
        }
    } catch {
        /* offline or no admin scope — the workflow will tell us */
    }
}

// 8. Lint — same gate as CI's "Lint & build (web)" job.
run('ESLint (npm run lint)', 'npm run lint');

// 9. TypeScript — catches type errors before they hit CI.
run('TypeScript check (tsc --noEmit)', 'npx tsc --noEmit');

// 10. Curriculum content validation — fail-closed:
//     the prereq DAG, every lesson's required parts, quiz answers, diagram
//     indices, every referenced slug has a frozen pack.
run(
    'Curriculum content check (build_curriculum.py --check)',
    'python tools/build_curriculum.py --check'
);

// 11. Next.js static export — validates the frontend bundle that gets packaged
//     into the Tauri app (same command `tauri.conf.json`'s beforeBuildCommand
//     runs, and CI's "Lint & build (web)" job).
run('Next.js static export (npm run build)', 'npm run build');

// 12. The export must have actually produced a UI — an empty `out/` would
//     bundle installers whose window opens onto nothing.
console.log('\n  🔍 Checking the exported frontend is not an empty shell...');
{
    const dist = 'out'; // tauri.conf.json build.frontendDist = "../out"
    if (!existsSync(`${dist}/index.html`)) {
        fail(`${dist}/index.html is missing after the export — the frontend did not build.`);
    }
    const count = (function walk(d) {
        return readdirSync(d).reduce((n, name) => {
            const p = `${d}/${name}`;
            return n + (statSync(p).isDirectory() ? walk(p) : 1);
        }, 0);
    })(dist);
    if (count < 10) {
        fail(`the exported frontend has only ${count} file(s) — that is an empty shell, not the app.`);
    }
    console.log(`  ✅ Frontend export present (${count} files)`);
}

// 13. Rust test suite — same gate as CI's "Rust (fmt, clippy, test)" job (fmt
//     and clippy run in CI on both platforms; the test suite is the slow part
//     worth catching locally before pushing a release tag).
run('Rust test suite (cargo test)', 'cargo test --manifest-path src-tauri/Cargo.toml');

if (dryRun) {
    console.log(`\n  🏁 Dry run complete — every gate passed for v${next}. Nothing was changed.\n`);
    process.exit(0);
}

// ─── Version bump ─────────────────────────────────────────────────────────────
// package.json + tauri.conf.json + src-tauri/Cargo.toml move together —
// RELEASING.md's checklist requires all three in sync.

const tauriConf = JSON.parse(readFileSync(tauriPath, 'utf-8'));
const cargoToml = readFileSync(cargoPath, 'utf-8');

const nextCargoToml = cargoToml.replace(
    /(name = "anvil"\nversion = )"[^"]+"/,
    `$1"${next}"`
);
if (nextCargoToml === cargoToml) {
    fail(`Could not find the anvil package's version line in ${cargoPath} to bump`);
}

pkg.version = next;
tauriConf.version = next;

writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf-8');
writeFileSync(tauriPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf-8');
writeFileSync(cargoPath, nextCargoToml, 'utf-8');

console.log(`\n  Bumping version: ${current} → ${next}\n`);
console.log('  ✅ Updated package.json');
console.log('  ✅ Updated src-tauri/tauri.conf.json');
console.log('  ✅ Updated src-tauri/Cargo.toml');

// Keep Cargo.lock's anvil entry in lockstep with the bumped Cargo.toml so a
// release never lands a stale lockfile (cargo rewrites it on the next build,
// and a `--locked` CI check would fail). Only the local anvil package version
// changes; --offline keeps this a fast, network-free rewrite (the dep graph is
// already resolved by the `cargo test` gate above).
const cargoLockPath = 'src-tauri/Cargo.lock';
try {
    execSync(`cargo update -p anvil --offline --manifest-path ${cargoPath}`, { stdio: 'inherit' });
} catch {
    fail('failed to sync src-tauri/Cargo.lock (cargo update -p anvil) — see the error above.');
}
const cargoLockChanged = sh(`git status --porcelain ${cargoLockPath}`) !== '';
if (cargoLockChanged) console.log('  ✅ Updated src-tauri/Cargo.lock');

// Stamp the dated CHANGELOG section (curated [Unreleased] notes win, else it's
// generated from the commits since the last tag). Idempotent: if a dated
// [next] section already exists this is a no-op and the file is left untouched.
// Pass --version/--base explicitly: this runs AFTER package.json was bumped to
// `next`, so letting prepare-release recompute from package.json would target
// the version *after* this one (and diff against a tag that doesn't exist yet).
try {
    execSync(`node tools/prepare-release.mjs ${type} --version ${next} --base v${current} --date ${today}`, { stdio: 'inherit' });
} catch {
    fail('failed to prepare the CHANGELOG (see the error above).');
}
const changelogChanged = sh('git status --porcelain CHANGELOG.md') !== '';
if (changelogChanged) console.log('  ✅ Updated CHANGELOG.md');

// ─── Commit, tag, push ────────────────────────────────────────────────────────
// Only stage the files we explicitly changed — never `git add .` (risks
// committing local env files or build artifacts). Commit message follows
// .docs/GIT_CONVENTIONS.md: `type(scope): summary` + one bullet per file,
// written via -F from a temp file so multi-line messages survive the shell
// untouched on every platform.

const msgFile = '.release-commit-msg.tmp';
try {
    const filesToCommit = [pkgPath, tauriPath, cargoPath];
    const commitMsg = [
        `chore(release): bump version to v${next}`,
        `- ${pkgPath}: version ${current} -> ${next}`,
        `- ${tauriPath}: version ${current} -> ${next}`,
        `- ${cargoPath}: version ${current} -> ${next}`,
    ];
    if (cargoLockChanged) {
        filesToCommit.push(cargoLockPath);
        commitMsg.push(`- ${cargoLockPath}: sync anvil lock entry to ${next}`);
    }
    if (changelogChanged) {
        filesToCommit.push('CHANGELOG.md');
        commitMsg.push(`- CHANGELOG.md: add [${next}] release notes`);
    }
    writeFileSync(msgFile, commitMsg.join('\n'), 'utf-8');

    execSync(`git add ${filesToCommit.join(' ')}`, { stdio: 'inherit' });
    execSync(`git commit -F ${msgFile}`, { stdio: 'inherit' });
    execSync('git push', { stdio: 'inherit' });
    // Annotated tag (has a message) so it works non-interactively when
    // `tag.gpgsign=true` — a signed tag is annotated and a bare `git tag <name>`
    // aborts with "no tag message". With signing configured this tag is signed;
    // without it, it's just an unsigned annotated tag (still fine).
    execSync(`git tag -m "Release v${next}" v${next}`, { stdio: 'inherit' });
    execSync(`git push origin v${next}`, { stdio: 'inherit' });

    console.log(`\n  Released v${next} successfully!\n`);
    console.log(`  Monitor build: https://github.com/kudzaiprichard/anvil/actions`);
    console.log(`  Draft release: https://github.com/kudzaiprichard/anvil-releases/releases\n`);
} catch {
    console.error('\n  ❌ Git commands failed. Check the error above.');
    console.error(`  The version bump to v${next} may be partially committed — inspect with`);
    console.error('  `git status` / `git log -1` before retrying.\n');
    process.exit(1);
} finally {
    try {
        unlinkSync(msgFile);
    } catch {
        /* nothing to clean up if we never got that far */
    }
}

// ─── Live build monitor ───────────────────────────────────────────────────────
// The tag push above just triggered the Release workflow: one create-release
// job, then four platform builds (Linux, Windows, macOS arm64 + x64). Watch
// it from here so the terminal tells the whole story — no tab-switching to
// know whether the installers actually landed. Purely observational: by this
// point the release is cut; Ctrl-C (or --no-watch) never un-releases
// anything, and a watch hiccup must not report a failed release.

if (noWatch) {
    console.log('  (--no-watch: skipping the live build monitor)\n');
    process.exit(0);
}
if (!ghAvailable()) {
    console.log('  (gh CLI not found: cannot watch the build from here — use the links above)\n');
    process.exit(0);
}

const ICONS = { queued: '○', waiting: '○', pending: '○', in_progress: '◐', completed: '●' };
const CONCLUSION = { success: '✅', failure: '❌', cancelled: '🚫', skipped: '⏭️' };

function jobLine(job) {
    if (job.status === 'completed') {
        return `  ${CONCLUSION[job.conclusion] ?? '●'} ${job.name}`;
    }
    return `  ${ICONS[job.status] ?? '○'} ${job.name} (${job.status.replace('_', ' ')})`;
}

console.log('  👀 Watching the release build (Ctrl-C is safe — the release is already cut)...\n');

let runId = null;
for (let attempt = 0; attempt < 24 && !runId; attempt++) {
    try {
        const runs = JSON.parse(
            sh('gh run list --workflow=release.yml --limit 5 --json databaseId,headBranch,status')
        );
        runId = runs.find((r) => r.headBranch === `v${next}`)?.databaseId ?? null;
    } catch {
        /* transient — retry below */
    }
    if (!runId) await sleep(5000);
}
if (!runId) {
    console.log('  ⚠️  Could not find the workflow run yet — watch it at the Actions link above.\n');
    process.exit(0);
}

const interactive = process.stdout.isTTY === true;
let printedLines = 0;
let lastSnapshot = '';
let conclusion = null;

while (conclusion === null) {
    let view;
    try {
        view = JSON.parse(sh(`gh run view ${runId} --json status,conclusion,jobs`));
    } catch {
        await sleep(10000);
        continue;
    }
    const lines = view.jobs.map(jobLine);
    const snapshot = lines.join('\n');
    if (interactive) {
        if (printedLines > 0) process.stdout.write(`\x1b[${printedLines}A\x1b[0J`);
        process.stdout.write(snapshot + '\n');
        printedLines = lines.length;
    } else if (snapshot !== lastSnapshot) {
        // Non-interactive (CI, piped): print only on change, no cursor tricks.
        console.log(snapshot + '\n');
        lastSnapshot = snapshot;
    }
    if (view.status === 'completed') {
        conclusion = view.conclusion;
        break;
    }
    await sleep(10000);
}

if (conclusion === 'success') {
    console.log(`\n  🎉 All platform builds succeeded for v${next}.`);
    console.log('  Review + publish the draft: https://github.com/kudzaiprichard/anvil-releases/releases\n');
} else {
    console.error(`\n  ❌ The release build finished with conclusion: ${conclusion}.`);
    console.error(`  Inspect the failed job(s): gh run view ${runId} --log-failed`);
    console.error('  After fixing, re-run the workflow from the existing tag (Actions → Release →');
    console.error(`  Run workflow → select v${next}), or cut a follow-up patch release.\n`);
    process.exit(1);
}
