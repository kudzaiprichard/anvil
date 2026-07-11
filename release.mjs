import { readFileSync, writeFileSync, unlinkSync } from 'fs';
import { execSync } from 'child_process';

// Anvil's release script (see RELEASING.md for the full runbook). Run from a
// clean `main`:
//
//   node release.mjs <patch|minor|major>
//
// This is the ONLY thing that should ever cut a release. A normal commit/push
// never triggers one: build.yml is workflow_dispatch-only, and release.yml
// only fires on a `v*` tag push (which is the last thing this script does) or
// a manual dispatch from an existing tag. Everything below runs BEFORE any
// file is touched or any git state changes — if anything fails, the repo is
// exactly as you left it: no half-bumped version, no stray commit, no tag.

const type = process.argv[2];

if (!['patch', 'minor', 'major'].includes(type)) {
    console.error('\n  Usage: node release.mjs <patch|minor|major>\n');
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

// ─── Pre-flight checks ────────────────────────────────────────────────────────
// Cheapest/fastest checks first so a mistake fails in milliseconds, not after
// a multi-minute cargo test run.

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

// 1. Tauri version alignment — NPM @tauri-apps/api minor must match Rust tauri minor.
//    This is the class of issue that broke v0.6.0's first CI run.
console.log('\n  🔍 Checking Tauri package version alignment...');
{
    const lockRaw = JSON.parse(readFileSync('package-lock.json', 'utf-8'));
    const npmVersion = lockRaw.packages?.['node_modules/@tauri-apps/api']?.version;
    if (!npmVersion) fail('@tauri-apps/api not found in package-lock.json');

    const cargoToml = readFileSync('src-tauri/Cargo.toml', 'utf-8');
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

// 2. Compute the target version now — the changelog check below needs it.
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

console.log(`\n  Preparing release: ${current} → ${next}`);

// 3. CHANGELOG.md must already have a dated entry for the version being
//    released (RELEASING.md's pre-release checklist) — written by hand
//    before running this script, not generated here.
console.log(`\n  🔍 Checking CHANGELOG.md for a [${next}] entry...`);
{
    const changelog = readFileSync('CHANGELOG.md', 'utf-8');
    if (!changelog.includes(`[${next}]`)) {
        fail(
            `CHANGELOG.md has no "[${next}]" entry yet. Add a dated section for v${next} ` +
            `before releasing (see RELEASING.md's pre-release checklist).`
        );
    }
    console.log(`  ✅ CHANGELOG.md has a [${next}] entry`);
}

// 4. Lint — same gate as CI's "Lint & build (web)" job.
run('ESLint (npm run lint)', 'npm run lint');

// 5. TypeScript — catches type errors before they hit CI.
run('TypeScript check (tsc --noEmit)', 'npx tsc --noEmit');

// 6. Curriculum content validation — fail-closed (LESSON_COURSE_DESIGN.md §8):
//    the prereq DAG, every lesson's required parts, quiz answers, diagram
//    indices, every referenced slug has a frozen pack.
run(
    'Curriculum content check (build_curriculum.py --check)',
    'python tools/build_curriculum.py --check'
);

// 7. Installer boundary check — THE non-negotiable gate (COURSE_BLUEPRINT.md
//    §2): a release must never bundle a *leetcode* catalog, and the bundled
//    resource payload must stay at the no-scrape baseline.
run(
    'Installer boundary check (no *leetcode* bundling)',
    'python tools/check_release_boundary.py'
);

// 8. Next.js static export — validates the frontend bundle that gets packaged
//    into the Tauri app (same command `tauri.conf.json`'s beforeBuildCommand
//    runs, and CI's "Lint & build (web)" job).
run('Next.js static export (npm run build)', 'npm run build');

// 9. Rust test suite — same gate as CI's "Rust (fmt, clippy, test)" job (fmt
//    and clippy run in CI on both platforms; the test suite is the slow part
//    worth catching locally before pushing a release tag).
run('Rust test suite (cargo test)', 'cargo test --manifest-path src-tauri/Cargo.toml');

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

// ─── Commit, tag, push ────────────────────────────────────────────────────────
// Only stage the files we explicitly changed — never `git add .` (risks
// committing local env files or build artifacts). Commit message follows
// .docs/GIT_CONVENTIONS.md: `type(scope): summary` + one bullet per file,
// written via -F from a temp file so multi-line messages survive the shell
// untouched on every platform.

const msgFile = '.release-commit-msg.tmp';
try {
    const commitMsg = [
        `chore(release): bump version to v${next}`,
        `- ${pkgPath}: version ${current} -> ${next}`,
        `- ${tauriPath}: version ${current} -> ${next}`,
        `- ${cargoPath}: version ${current} -> ${next}`,
    ].join('\n');
    writeFileSync(msgFile, commitMsg, 'utf-8');

    execSync(`git add ${pkgPath} ${tauriPath} ${cargoPath}`, { stdio: 'inherit' });
    execSync(`git commit -F ${msgFile}`, { stdio: 'inherit' });
    execSync('git push', { stdio: 'inherit' });
    execSync(`git tag v${next}`, { stdio: 'inherit' });
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
