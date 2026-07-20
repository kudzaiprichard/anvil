#!/usr/bin/env node
// Auto-maintain CHANGELOG.md so a release NEVER stalls on a missing/mis-dated
// section — and so nobody has to hand-write one. `release.mjs` calls this for
// you; you can also run it directly to preview.
//
//   node tools/prepare-release.mjs <patch|minor|major> [options]
//     --date YYYY-MM-DD   release date (default: today)
//     --base <tag>        commit range base for generation (default: v<current>)
//     --version <x.y.z>   target version (default: computed from package.json)
//     --dry-run           print what would be written; touch nothing
//
// Where the section's CONTENT comes from, in priority order:
//   1. A curated `## [Unreleased]` block — if you wrote notes there, they win.
//   2. Otherwise it is GENERATED from the Conventional-Commit subjects since the
//      last tag (this repo's commit hook enforces `type(scope): summary`), mapped
//      feat→Added, fix→Fixed, perf/refactor/revert→Changed, and grouped.
// Either way it stamps `## [X.Y.Z] - <date>`, leaves `[Unreleased]` empty for the
// next cycle, and maintains the footer compare links. Idempotent: if the target
// section already exists it changes nothing. It never commits, tags, or pushes.

import { readFileSync, writeFileSync } from 'fs';
import { execSync } from 'child_process';

const args = process.argv.slice(2);
const type = args[0];
const dryRun = args.includes('--dry-run');
const flag = (name) => {
    const i = args.indexOf(name);
    return i !== -1 ? args[i + 1] : null;
};

function fail(msg) {
    console.error(`\n  ❌ ${msg}\n`);
    process.exit(1);
}

if (!['patch', 'minor', 'major'].includes(type)) {
    console.error('\n  Usage: node tools/prepare-release.mjs <patch|minor|major> [--date YYYY-MM-DD] [--base <tag>] [--version <x.y.z>] [--dry-run]\n');
    process.exit(1);
}

const current = JSON.parse(readFileSync('package.json', 'utf-8')).version;
const [major, minor, patch] = current.split('.').map(Number);
const next =
    flag('--version') ??
    (type === 'major' ? `${major + 1}.0.0` : type === 'minor' ? `${major}.${minor + 1}.0` : `${major}.${minor}.${patch + 1}`);

const date = flag('--date') ?? new Date().toISOString().slice(0, 10);
if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) fail(`--date must be YYYY-MM-DD (got "${date}").`);

const base = flag('--base') ?? `v${current}`;

const path = 'CHANGELOG.md';
const changelog = readFileSync(path, 'utf-8');

// Already prepared? release.mjs's exact gate — if it passes, we're done.
if (new RegExp(`^## \\[${next.replace(/\./g, '\\.')}\\] - \\d{4}-\\d{2}-\\d{2}`, 'm').test(changelog)) {
    console.log(`\n  ✅ CHANGELOG.md already has a dated [${next}] section — nothing to do.\n`);
    process.exit(0);
}

// ─── Locate the [Unreleased] section and its body ─────────────────────────────
const uMatch = /^## \[Unreleased\][^\n]*$/m.exec(changelog);
if (!uMatch) fail('CHANGELOG.md has no "## [Unreleased]" section — cannot prepare a release.');
const afterU = changelog.slice(uMatch.index + uMatch[0].length);
const nextHeading = /^## \[/m.exec(afterU);
const rest = nextHeading ? afterU.slice(nextHeading.index) : '';
const unreleasedBody = (nextHeading ? afterU.slice(0, nextHeading.index) : afterU).trim();

// ─── Decide the section body: curated notes win, else generate from commits ───
function generateFromCommits() {
    let subjects = [];
    for (const range of [`${base}..HEAD`, 'HEAD']) {
        try {
            subjects = execSync(`git log --no-merges --format=%s ${range}`, { encoding: 'utf-8' })
                .split('\n')
                .map((s) => s.trim())
                .filter(Boolean);
            break; // first range that resolves (base tag may not exist yet)
        } catch {
            /* base tag missing — fall through to the whole-history range */
        }
    }

    const buckets = { Added: [], Changed: [], Fixed: [] };
    const leftovers = []; // non-user-facing types, used only if nothing else lands
    for (const s of subjects) {
        if (/^chore\(release\):\s*bump version/i.test(s)) continue; // our own release commits
        const m = s.match(/^(\w+)(?:\(([^)]*)\))?(!)?:\s*(.+)$/);
        if (!m) continue; // non-conventional subject — skip
        const [, t, , bang, summary] = m;
        const bullet = `- ${bang ? '**Breaking:** ' : ''}${summary}`; // summary already carries any (#PR)
        if (t === 'feat') buckets.Added.push(bullet);
        else if (t === 'fix') buckets.Fixed.push(bullet);
        else if (t === 'perf' || t === 'refactor' || t === 'revert' || bang) buckets.Changed.push(bullet);
        else leftovers.push(bullet);
    }

    // "Works for all": if only chores/docs/etc landed, don't emit an empty
    // section — surface them under Changed rather than shipping a bare heading.
    if (!buckets.Added.length && !buckets.Changed.length && !buckets.Fixed.length) {
        buckets.Changed = leftovers;
    }

    const out = [];
    for (const name of ['Added', 'Changed', 'Fixed']) {
        if (buckets[name].length) out.push(`### ${name}\n\n${buckets[name].join('\n')}`);
    }
    return out.join('\n\n');
}

let body, source;
if (unreleasedBody) {
    body = unreleasedBody;
    source = 'curated [Unreleased] notes';
} else {
    body = generateFromCommits();
    source = `commits since ${base}`;
}
if (!body.trim()) {
    body = '_Maintenance release — no user-facing changes._';
    source = 'fallback (no releasable commits found)';
}

// ─── Rewrite: empty [Unreleased] + dated section, then fix footer links ───────
const head = changelog.slice(0, uMatch.index + uMatch[0].length);
let out = `${head}\n\n## [${next}] - ${date}\n\n${body}\n\n${rest}`;

const unrelLink = out.match(/^\[Unreleased\]:\s*(https?:\/\/\S+?)\/compare\/\S+\.\.\.HEAD\s*$/m);
if (unrelLink) {
    const baseUrl = unrelLink[1];
    out = out.replace(
        /^\[Unreleased\]:\s*\S+\s*$/m,
        `[Unreleased]: ${baseUrl}/compare/v${next}...HEAD\n[${next}]: ${baseUrl}/compare/v${current}...v${next}`
    );
} else {
    console.warn(`\n  ⚠️  No "[Unreleased]: …/compare/…...HEAD" footer link found — add the [${next}] compare link by hand.`);
}

if (dryRun) {
    console.log(`\n  ── CHANGELOG [${next}] - ${date}  (source: ${source}) ──\n`);
    console.log(body.replace(/^/gm, '  '));
    console.log(`\n  🏁 Dry run — nothing written.\n`);
    process.exit(0);
}

writeFileSync(path, out, 'utf-8');
console.log(`\n  ✅ CHANGELOG.md: [Unreleased] → [${next}] - ${date}  (source: ${source}).\n`);
