// Builds the Windows desktop app and packages it as an MSIX for the Microsoft
// Store. Tauri's bundler only makes NSIS/MSI installers, so the app is built
// with `tauri build --no-bundle` and Microsoft's winapp CLI (`winapp package`)
// packs the binary. This follows Microsoft's "Using winapp CLI with Tauri"
// guide (the same route the OurAfrica desktop app ships through).
//
//   node tools/pack-msix.mjs              Store build: unsigned .msix for Partner Center
//   node tools/pack-msix.mjs --dev        Signed with a local dev certificate, installable here
//   node tools/pack-msix.mjs --skip-build Package the release binary already built
//
// Output, in <cargo target dir>/msix (CARGO_TARGET_DIR is honoured):
//   Anvil-Store_<version>_x64.msix   the Store build, for Partner Center
//   Anvil-dev_<version>_x64.msix     --dev
//
// Needs Windows and the winapp CLI (`winget install Microsoft.winappcli
// --source winget`; CI uses microsoft/setup-WinAppCli). WINAPP overrides the
// executable path.
//
// The Store signs the package on ingestion, so a Store build stays unsigned.
// A --dev build uses a stand-in identity and a self-signed certificate
// (devcert.pfx/.cer beside the output). Trust it once with
// `winapp cert install <devcert.pfx>` from an elevated prompt, then
// double-click the .msix.
//
// Unlike a single-exe Tauri app, Anvil reads its packs, lessons, curriculum
// and pylib from resources/ beside the exe (resource_dir() on Windows is the
// exe's folder), so the bundle.resources globs from tauri.conf.json are staged
// too, and the shipping boundary (no *leetcode* file) is enforced on the stage.

import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gunzipSync } from 'node:zlib';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const tauriDir = join(root, 'src-tauri');
const args = new Set(process.argv.slice(2));
const dev = args.has('--dev');
const skipBuild = args.has('--skip-build');

for (const a of args) {
    if (!['--dev', '--skip-build'].includes(a)) fail(`unknown option ${a}`);
}

function fail(message) {
    console.error(`\n  ❌ pack-msix: ${message}\n`);
    process.exit(1);
}

// Through the shell (npx is a .cmd shim on Windows), as one pre-quoted
// command line. Paths are quoted where they're built.
function run(cmd, cmdArgs, opts = {}) {
    const line = `${cmd} ${cmdArgs.join(' ')}`;
    console.log(`\n  ▶ ${line}`);
    const r = spawnSync(line, { cwd: root, stdio: 'inherit', shell: true, ...opts });
    if (r.status !== 0) fail(`${cmd} exited with ${r.status ?? r.signal}`);
}

if (process.platform !== 'win32') fail('MSIX packages can only be built on Windows.');

// ── version + names ───────────────────────────────────────────────────────────
const tauriConf = JSON.parse(readFileSync(join(tauriDir, 'tauri.conf.json'), 'utf-8'));
const semver = String(tauriConf.version ?? '');
if (!/^\d+\.\d+\.\d+$/.test(semver)) {
    fail(`tauri.conf.json version "${semver}" isn't plain major.minor.patch, so it can't map to an MSIX version.`);
}
if (semver.split('.').some((n) => Number(n) > 65535)) fail(`version ${semver} has a part above 65535.`);
// The Store reserves the fourth part, which must be 0.
const msixVersion = `${semver}.0`;
const executable = `${tauriConf.productName ?? 'Anvil'}.exe`;

// ── identity ──────────────────────────────────────────────────────────────────
const identityFile = JSON.parse(readFileSync(join(tauriDir, 'msix', 'store-identity.json'), 'utf-8'));
const identity = {
    name: process.env.MSIX_IDENTITY_NAME || identityFile.identityName,
    publisher: process.env.MSIX_PUBLISHER || identityFile.publisher,
    publisherDisplayName: process.env.MSIX_PUBLISHER_DISPLAY_NAME || identityFile.publisherDisplayName,
    displayName: identityFile.displayName || 'Anvil',
};
if (dev) {
    // A stand-in identity: installable beside the Store build, and it can't be
    // mistaken for an upload because its publisher isn't ours on Partner Center.
    identity.name = 'Anvil.Dev';
    identity.publisher = 'CN=Anvil Dev';
    identity.publisherDisplayName = 'Anvil (dev build)';
    identity.displayName = 'Anvil Dev';
} else {
    if (!identity.name || !identity.publisher) {
        fail(
            'the Microsoft Store identity is not configured. Copy "Package/Identity/Name" and ' +
            '"Package/Identity/Publisher" from Partner Center > Product management > Product identity ' +
            'into src-tauri/msix/store-identity.json (or set MSIX_IDENTITY_NAME and MSIX_PUBLISHER).'
        );
    }
    if (!/^CN=/.test(identity.publisher)) {
        fail(`publisher "${identity.publisher}" must be the full "CN=..." value from Partner Center.`);
    }
}

// ── winapp ────────────────────────────────────────────────────────────────────
// The winget install is an app execution alias in %LOCALAPPDATA%\Microsoft\
// WindowsApps, which some shells leave off PATH.
const winapp = (() => {
    if (process.env.WINAPP) return `"${process.env.WINAPP}"`;
    if (spawnSync('winapp --version', { shell: true, stdio: 'ignore' }).status === 0) return 'winapp';
    // An execution alias is a reparse point that stat() can't follow, so
    // probe it by running it rather than with existsSync.
    const alias = `"${join(process.env.LOCALAPPDATA ?? '', 'Microsoft', 'WindowsApps', 'winapp.exe')}"`;
    if (spawnSync(`${alias} --version`, { shell: true, stdio: 'ignore' }).status === 0) return alias;
    fail('winapp CLI not found. Install it with: winget install Microsoft.winappcli --source winget');
})();

// ── build ─────────────────────────────────────────────────────────────────────
// `--no-bundle`: compile the app (beforeBuildCommand still fetches the prod
// catalog and runs the static export, which is embedded in the binary) and
// skip the NSIS/MSI installers, which aren't used here.
if (!skipBuild) run('npx', ['tauri', 'build', '--no-bundle']);

const targetDir = process.env.CARGO_TARGET_DIR || join(tauriDir, 'target');
const releaseDir = join(targetDir, 'release');
// Tauri may or may not rename the binary after productName; the cargo package
// name (anvil.exe) is the other candidate. Take the newest.
const builtExe = [executable, 'anvil.exe', 'app.exe']
    .map((n) => join(releaseDir, n))
    .filter((p) => existsSync(p))
    .sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs)[0];
if (!builtExe) fail(`no release binary in ${releaseDir}. Run without --skip-build.`);

// ── stage ─────────────────────────────────────────────────────────────────────
// The exe, the tile assets and the bundled resources. Nothing else from
// target/release (it's mostly build intermediates that must not ship).
const outDir = join(targetDir, 'msix');
const stage = join(outDir, 'stage');
rmSync(stage, { recursive: true, force: true });
mkdirSync(join(stage, 'Assets'), { recursive: true });

copyFileSync(builtExe, join(stage, executable));
for (const asset of ['StoreLogo.png', 'Square44x44Logo.png', 'Square71x71Logo.png', 'Square150x150Logo.png']) {
    const src = join(tauriDir, 'icons', asset);
    if (!existsSync(src)) fail(`missing ${src}. Regenerate the icons with npx tauri icon.`);
    copyFileSync(src, join(stage, 'Assets', asset));
}

// bundle.resources, resolved the way Tauri does for the NSIS installer: each
// entry is a path or glob relative to src-tauri/, and a plain directory means
// everything under it. Files keep their relative path, so resources/foo lands
// at <exe dir>/resources/foo.
const globToRegex = (glob) =>
    new RegExp(
        '^' +
        glob
            .replace(/[.+^${}()|[\]\\]/g, '\\$&')
            .replace(/\*\*\//g, '\u0000')
            .replace(/\*\*/g, '\u0001')
            .replace(/\*/g, '[^/]*')
            .replace(/\?/g, '[^/]')
            .replace(/\u0000/g, '(?:.*/)?')
            .replace(/\u0001/g, '.*') +
        '$'
    );
const walk = (dir) =>
    readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
        e.isDirectory() ? walk(join(dir, e.name)) : [join(dir, e.name)]
    );
const resourcePatterns = tauriConf.bundle?.resources ?? [];
if (!Array.isArray(resourcePatterns)) fail('bundle.resources must be an array for pack-msix.');
const allResourceFiles = existsSync(join(tauriDir, 'resources'))
    ? walk(join(tauriDir, 'resources')).map((p) => relative(tauriDir, p).split(sep).join('/'))
    : [];
const staged = new Set();
for (const pattern of resourcePatterns) {
    const isGlob = /[*?]/.test(pattern);
    const re = isGlob ? globToRegex(pattern) : null;
    const matches = allResourceFiles.filter((f) => (isGlob ? re.test(f) : f === pattern || f.startsWith(`${pattern}/`)));
    if (matches.length === 0 && !isGlob && !existsSync(join(tauriDir, pattern))) {
        fail(`bundle resource "${pattern}" doesn't exist.`);
    }
    for (const f of matches) staged.add(f);
}
// Shipping boundary (see RELEASING.md): a *leetcode* file must never ship.
const tainted = [...staged].filter((f) => /leetcode/i.test(f));
if (tainted.length) fail(`refusing to package *leetcode* resources: ${tainted.join(', ')}`);
// A dev catalog fetched under a neutral name (catalog.json) is still the
// LeetCode scrape. Judge catalog files by content and leave them out of the
// package (the developer's copy in src-tauri/resources is not touched). The
// Store build then ships the empty, bring-your-own library like every release.
for (const f of [...staged].filter((f) => f.startsWith('resources/catalog/') && !f.endsWith('/.gitkeep'))) {
    let head = '';
    try {
        const buf = readFileSync(join(tauriDir, ...f.split('/')));
        head = (f.endsWith('.gz') ? gunzipSync(buf) : buf).subarray(0, 8192).toString('utf8');
    } catch {
        fail(`can't read ${f} to check it is shippable.`);
    }
    if (/"source"\s*:\s*"[^"]*leetcode/i.test(head) || /property of LeetCode/i.test(head)) {
        staged.delete(f);
        console.log(`  ⚠ left out ${f}: it holds LeetCode content, which never ships (DISCLAIMER.md §3–4)`);
    }
}
for (const f of staged) {
    const dest = join(stage, ...f.split('/'));
    mkdirSync(dirname(dest), { recursive: true });
    copyFileSync(join(tauriDir, ...f.split('/')), dest);
}
// Tauri creates an empty resources/catalog directory too; keep the folder
// shape even when the shipped catalog is empty (bring-your-own build).
for (const pattern of resourcePatterns) {
    if (!/[*?]/.test(pattern) && existsSync(join(tauriDir, pattern)) && statSync(join(tauriDir, pattern)).isDirectory()) {
        mkdirSync(join(stage, ...pattern.split('/')), { recursive: true });
    }
}
console.log(`  ✅ Staged ${executable}, 4 tile assets and ${staged.size} resource files`);

// The release boundary rules (tools/check_release_boundary.py) applied to
// exactly what will be packed: no *leetcode* name or content (above) and the
// payload under the same 10 MB no-scrape baseline. That script itself also
// gates the source tree, which holds the developer's dev catalog, so the
// stage is checked here instead.
{
    const stagedBytes = [...staged].reduce((n, f) => n + statSync(join(stage, ...f.split('/'))).size, 0);
    const mbStaged = (stagedBytes / 1048576).toFixed(2);
    if (stagedBytes > 10 * 1024 * 1024) fail(`staged resources are ${mbStaged} MB, over the 10 MB no-scrape baseline.`);
    console.log(`  ✅ Boundary rules pass on the stage (${mbStaged} MB of resources, no LeetCode content)`);
}

const xmlEscape = (s) =>
    String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const tokens = {
    IDENTITY_NAME: identity.name,
    PUBLISHER: identity.publisher,
    PUBLISHER_DISPLAY_NAME: identity.publisherDisplayName,
    DISPLAY_NAME: identity.displayName,
    VERSION: msixVersion,
    EXECUTABLE: executable,
};
const template = readFileSync(join(tauriDir, 'msix', 'Package.appxmanifest'), 'utf-8');
const manifest = template.replace(/\{\{([A-Z_]+)\}\}/g, (m, key) => {
    if (!(key in tokens)) fail(`unknown manifest token ${m}`);
    return xmlEscape(tokens[key]);
});
// Beside the staging folder, not in it: everything in `stage` is packed, and
// winapp writes its own AppxManifest.xml into the package.
const manifestPath = join(outDir, 'Package.appxmanifest');
writeFileSync(manifestPath, manifest);

// ── pack ──────────────────────────────────────────────────────────────────────
const baseName = dev ? 'Anvil-dev' : 'Anvil-Store';
const output = join(outDir, `${baseName}_${msixVersion}_x64.msix`);
rmSync(output, { force: true });
const packArgs = [`"${stage}"`, '--manifest', `"${manifestPath}"`, '--executable', executable, '--output', `"${output}"`];

if (dev) {
    const cert = join(outDir, 'devcert.pfx');
    run(winapp, ['cert', 'generate', '--manifest', `"${manifestPath}"`, '--output', `"${cert}"`, '--export-cer', '--if-exists', 'skip']);
    packArgs.push('--cert', `"${cert}"`);
} else {
    // The Store signs on ingestion.
    packArgs.push('--no-sign');
}
run(winapp, ['package', ...packArgs]);

if (!existsSync(output)) fail(`winapp reported success but ${output} is missing.`);
const mb = (statSync(output).size / 1024 / 1024).toFixed(1);
console.log(`\n  ✅ ${output} (${mb} MB, version ${msixVersion}, ${dev ? 'dev-signed' : 'unsigned for the Store'})`);
if (dev) {
    console.log(
        '\n  To install it on this PC: trust the dev certificate once (elevated prompt)\n' +
        `    winapp cert install "${join(outDir, 'devcert.pfx')}"\n` +
        '  then double-click the .msix, or run Add-AppxPackage on it.'
    );
} else {
    console.log('\n  Upload it in Partner Center > Anvil DSA > Submission > Packages.');
}
