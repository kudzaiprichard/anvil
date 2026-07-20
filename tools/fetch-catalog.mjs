#!/usr/bin/env node
// Fetch a catalog into the bundled catalog dir at BUILD TIME.
//
// This is wired into Tauri's before-commands (src-tauri/tauri.conf.json), so a
// `tauri dev` / `tauri build` fetches-and-embeds the catalog for you — no more
// running `npm run catalog:fetch` by hand before every build.
//
//   beforeDevCommand:   npm run catalog:fetch:dev  && npm run dev
//   beforeBuildCommand: npm run catalog:fetch:prod && npm run build
//
// ONE knob — ANVIL_CATALOG_URL — points at the RAW catalog file. Its VALUE is
// what differs per environment: locally your .env points it at your dev
// (LeetCode) catalog; in CI / a release build you set it to your OWN shippable
// catalog. The build always fetches whatever the variable points to.
//
// Modes:
//   --mode dev   (default) Fetch as-is. The LeetCode scrape may live in the
//                catalog dir here — it is dev-only, gitignored, and blocked
//                from installers.
//   --mode prod  SHIPPING build. First PURGE any *leetcode* scrape from the
//                catalog dir (it is legally un-shippable — DISCLAIMER.md §3–4),
//                then fetch. Refuses outright if ANVIL_CATALOG_URL resolves to a
//                *leetcode* file, since that can never be bundled.
//
// The download is SKIPPED when the target file already exists; pass --force (or
// set ANVIL_CATALOG_REFRESH=1) to always re-download.
//
// Usage:
//   npm run catalog:fetch:dev
//   npm run catalog:fetch:prod
//   ANVIL_CATALOG_URL=<raw-url> node tools/fetch-catalog.mjs --mode prod --force
//
// The URL should point at the RAW file, e.g.
//   https://raw.githubusercontent.com/<owner>/<repo>/<branch>/catalog.json

import { readFileSync, existsSync, mkdirSync, writeFileSync, readdirSync, rmSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const CATALOG_DIR = join(ROOT, 'src-tauri', 'resources', 'catalog');

// Minimal .env reader — avoids adding a dependency. Only KEY=VALUE lines,
// ignores comments/blanks; does not override an already-set process env.
function loadDotEnv() {
  const envPath = join(ROOT, '.env');
  if (!existsSync(envPath)) return;
  for (const line of readFileSync(envPath, 'utf8').split(/\r?\n/)) {
    const m = line.match(/^\s*([A-Z_][A-Z0-9_]*)\s*=\s*(.*)\s*$/i);
    if (!m) continue;
    const key = m[1];
    let val = m[2];
    if (/^".*"$/.test(val) || /^'.*'$/.test(val)) val = val.slice(1, -1);
    if (process.env[key] === undefined) process.env[key] = val;
  }
}

function parseArgs() {
  const argv = process.argv.slice(2);
  let mode = process.env.NODE_ENV === 'production' ? 'prod' : 'dev';
  let force =
    process.env.ANVIL_CATALOG_REFRESH === '1' || process.env.ANVIL_CATALOG_REFRESH === 'true';
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--mode') mode = argv[++i];
    else if (a.startsWith('--mode=')) mode = a.slice('--mode='.length);
    else if (a === '--dev') mode = 'dev';
    else if (a === '--prod') mode = 'prod';
    else if (a === '--force') force = true;
  }
  if (mode !== 'dev' && mode !== 'prod') {
    console.error(`ERROR: unknown --mode '${mode}' (expected dev|prod).`);
    process.exit(1);
  }
  return { mode, force };
}

function isLeetcodeName(name) {
  return /leetcode/i.test(name);
}

function fileNameFromUrl(url) {
  try {
    const base = new URL(url).pathname.split('/').pop() || '';
    // Loader convention: any catalog*.json[.gz]. Keep the source name if it
    // already matches; otherwise fall back to the canonical local name.
    if (/^catalog.*\.json(\.gz)?$/i.test(base)) return base;
  } catch {
    /* fall through */
  }
  return 'catalog.json';
}

// Remove any *leetcode* scrape from the catalog dir so a prod/shipping build can
// never sweep it into the installer (the release boundary gate would fail, but
// we fail closed here first). Dev-only, un-shippable — DISCLAIMER.md §3–4.
function purgeScrapes() {
  if (!existsSync(CATALOG_DIR)) return;
  for (const name of readdirSync(CATALOG_DIR)) {
    if (isLeetcodeName(name)) {
      rmSync(join(CATALOG_DIR, name));
      console.log(`  purged dev scrape (un-shippable): ${name}`);
    }
  }
}

async function main() {
  const { mode, force } = parseArgs();
  loadDotEnv();

  const url = process.env.ANVIL_CATALOG_URL;

  // PROD is the SHIPPING build. By design Anvil ships with an EMPTY library
  // (bring-your-own-statement) and users import their own problems, so a
  // bundled catalog is OPTIONAL here — catalog trouble must NEVER fail the
  // release, and *leetcode* content must NEVER be bundled. So in prod a missing
  // URL or a *leetcode* URL both degrade to "ship empty" with a loud, visible
  // (::warning::) notice rather than a hard error. A valid, non-*leetcode* URL
  // is still fetched and bundled below.
  if (mode === 'prod') {
    purgeScrapes(); // strip any dev *leetcode* scrape before we decide
    if (!url) {
      console.log(
        '::warning::ANVIL_CATALOG_URL is not set — shipping an EMPTY catalog ' +
          '(bring-your-own). Point a repo Variable/Secret at your OWN, ' +
          'non-*leetcode* catalog to bundle problems into the installer.',
      );
      return;
    }
    if (isLeetcodeName(fileNameFromUrl(url))) {
      console.log(
        '::warning::ANVIL_CATALOG_URL resolves to a *leetcode* file — legally ' +
          'un-shippable (DISCLAIMER.md §3–4), so it is NOT bundled; shipping an ' +
          'EMPTY catalog. Point ANVIL_CATALOG_URL at your OWN catalog to bundle ' +
          'problems.',
      );
      return;
    }
  } else if (!url) {
    // DEV: a URL is required to populate your working catalog.
    console.error(
      'ERROR: ANVIL_CATALOG_URL is not set — cannot fetch the catalog.\n' +
        '  Set it in .env (see .env.example):\n' +
        '    ANVIL_CATALOG_URL=<raw-url>',
    );
    process.exit(1);
  }

  const outName = fileNameFromUrl(url);

  if (!existsSync(CATALOG_DIR)) mkdirSync(CATALOG_DIR, { recursive: true });
  const outPath = join(CATALOG_DIR, outName);

  if (existsSync(outPath) && !force) {
    console.log(
      `Catalog already present — skipping fetch (${outName}).\n` +
        '  Pass --force (or ANVIL_CATALOG_REFRESH=1) to re-download.',
    );
    return;
  }

  console.log(`Fetching catalog (mode=${mode})…\n  from: ${url}\n  to:   ${outPath}`);

  const res = await fetch(url);
  if (!res.ok) {
    console.error(`ERROR: fetch failed — HTTP ${res.status} ${res.statusText}`);
    process.exit(1);
  }

  const buf = Buffer.from(await res.arrayBuffer());

  // Sanity check: plain-JSON catalogs must parse. Skip for gzipped payloads.
  if (outName.endsWith('.json')) {
    try {
      const parsed = JSON.parse(buf.toString('utf8'));
      const count = Array.isArray(parsed)
        ? parsed.length
        : Array.isArray(parsed?.questions)
          ? parsed.questions.length
          : undefined;
      console.log(`  parsed OK${count !== undefined ? ` — ${count} entries` : ''}`);
    } catch {
      console.error('ERROR: fetched payload is not valid JSON — refusing to write.');
      process.exit(1);
    }
  }

  writeFileSync(outPath, buf);

  const mb = (buf.length / 1024 / 1024).toFixed(2);
  console.log(`Done — wrote ${mb} MB to ${outName}.`);
  if (isLeetcodeName(outName)) {
    console.log(
      'Reminder: this file is *leetcode* content — gitignored, dev-only, and\n' +
        '  blocked from public installers by tools/check_release_boundary.py.\n' +
        '  Do not commit or redistribute it.',
    );
  }
}

main().catch((err) => {
  console.error('ERROR:', err?.message ?? err);
  process.exit(1);
});
