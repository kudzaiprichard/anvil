#!/usr/bin/env node
// Anvil brand-asset generator.
//
// Single source of truth for every installer / store / icon raster the app
// ships. It reuses the exact in-app AnvilMark geometry (src/components/anvil/
// logo.tsx) on a forged-ember squircle, so the dock/taskbar icon matches the
// mark shown inside the running app. Palette + treatment match the hero banner
// (.github/assets/banner.html) and the "forged iron & ember" OKLCH tokens in
// app/globals.css.
//
//   node branding/generate.mjs            # regenerate every asset
//
// Outputs:
//   src-tauri/icons/icon.png              master (feeds `tauri icon`)
//   src-tauri/installer/*.bmp             NSIS header/sidebar, WiX banner/dialog
//   src-tauri/installer/dmg-background*   macOS DMG window art (@1x + @2x)
//   branding/src/*.png|svg                review copies of everything
//
// Requires: sharp (already a dependency via Next.js image optimizer).

import sharp from "sharp";
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const ICONS = join(ROOT, "src-tauri", "icons");
const INSTALLER = join(ROOT, "src-tauri", "installer");
const SRC = join(ROOT, "branding", "src"); // rendered source SVGs + review PNGs

// ── palette (matches banner.html / globals.css) ─────────────────────────────
const C = {
  emberHot: "#F0A35F", // tile top / highlight
  ember: "#D6621F",
  emberMid: "#C76A38", // tile bottom
  emberDeep: "#A83E10",
  accentA: "#F5AD64", // wordmark accent gradient
  accentB: "#E8894B",
  forgeGlow: "#FFB871",
  bone: "#FFF5EC", // anvil face on the ember tile
  boneLow: "#F1EADE",
  ink: "#15110C", // forged-iron base (banner)
  ink2: "#1B1712",
  textHi: "#F3EFE8",
  textMid: "#D7CFC2",
  textLow: "#A99E90",
  panel: "#F4F0E9", // light panel for installer body text (WiX)
  panelInk: "#231C18",
};

const FONT = "'Segoe UI', system-ui, -apple-system, Roboto, sans-serif";

// AnvilMark — identical geometry to src/components/anvil/logo.tsx.
const ANVIL = [
  "M3.2 5.6h15.2c1.8 0 3.3 1 4.4 2.1.4.4.1 1.1-.5 1.1h-4.1v1.1c0 .7.4 1.3 1 1.6l.9.4c.5.2.5 1 0 1.2-2.3.9-4.8 1-7.1.3-1.6-.5-3.3-.5-4.9 0-1.1.3-2.3.5-3.4.5-.6 0-.9-.7-.5-1.1l1.5-1.4c.5-.5.8-1.1.8-1.8v-.8H3.2c-.6 0-1-.4-1-1V6.6c0-.6.4-1 1-1z",
  "M9.4 14.9h5.2l1 2.3H8.4z",
  "M6.5 18.2h11c.6 0 1 .4 1 1v.6c0 .6-.4 1-1 1h-11c-.6 0-1-.4-1-1v-.6c0-.6.4-1 1-1z",
];
const MARK = { cx: 12.5, cy: 13.2, w: 20.6, h: 15.2 };

/** An anvil group centred at (cx,cy), sized `w` px wide, filled `fill`. */
function anvilGroup(cx, cy, w, fill = "url(#bone)", extra = "") {
  const s = w / MARK.w;
  return `<g transform="translate(${cx} ${cy}) scale(${s}) translate(${-MARK.cx} ${-MARK.cy})" ${extra}>
    ${ANVIL.map((d) => `<path d="${d}" fill="${fill}"/>`).join("")}
  </g>`;
}

/** Shared gradient / filter defs used across the banner-style assets. */
function commonDefs() {
  return `
    <linearGradient id="emberTile" x1="0" y1="0" x2="0.55" y2="1">
      <stop offset="0" stop-color="${C.emberHot}"/>
      <stop offset="1" stop-color="${C.emberMid}"/>
    </linearGradient>
    <linearGradient id="wordAccent" x1="0" y1="0" x2="1" y2="0.4">
      <stop offset="0" stop-color="${C.accentA}"/>
      <stop offset="1" stop-color="${C.accentB}"/>
    </linearGradient>
    <pattern id="dots" width="27" height="27" patternUnits="userSpaceOnUse">
      <circle cx="1.2" cy="1.2" r="1.1" fill="#E9E5DD" fill-opacity="0.055"/>
    </pattern>
    <filter id="tileGlow" x="-60%" y="-60%" width="220%" height="220%">
      <feDropShadow dx="0" dy="10" stdDeviation="16" flood-color="#C76A38" flood-opacity="0.42"/>
    </filter>`;
}

/** Forged-iron background rect with the top-right ember glow + dot grid. */
function forgeBg(w, h, { glowX = 0.86 } = {}) {
  return `
    <rect width="${w}" height="${h}" fill="${C.ink}"/>
    <radialGradient id="g1" cx="${glowX}" cy="-0.05" r="0.75">
      <stop offset="0" stop-color="${C.emberHot}" stop-opacity="0.20"/>
      <stop offset="0.62" stop-color="${C.emberHot}" stop-opacity="0"/>
    </radialGradient>
    <radialGradient id="g2" cx="${glowX - 0.06}" cy="0.34" r="0.6">
      <stop offset="0" stop-color="${C.emberMid}" stop-opacity="0.12"/>
      <stop offset="0.6" stop-color="${C.emberMid}" stop-opacity="0"/>
    </radialGradient>
    <rect width="${w}" height="${h}" fill="url(#dots)"/>
    <rect width="${w}" height="${h}" fill="url(#g1)"/>
    <rect width="${w}" height="${h}" fill="url(#g2)"/>`;
}

/** Rounded ember tile with the bone anvil + inset highlight. */
function emberTile(x, y, size, r = size * 0.26) {
  return `
    <g filter="url(#tileGlow)">
      <rect x="${x}" y="${y}" width="${size}" height="${size}" rx="${r}" ry="${r}" fill="url(#emberTile)"/>
    </g>
    <rect x="${x}" y="${y}" width="${size}" height="${size}" rx="${r}" ry="${r}"
          fill="none" stroke="#FFFFFF" stroke-opacity="0.25" stroke-width="1"/>
    ${anvilGroup(x + size / 2, y + size / 2, size * 0.6, C.bone)}`;
}

function spark(x, y, r, o) {
  return `<circle cx="${x}" cy="${y}" r="${r}" fill="${C.emberHot}" opacity="${o}"/>`;
}

// ── the forged-ember app icon ───────────────────────────────────────────────
function iconSVG({ size = 1024, pad = 100 } = {}) {
  const tile = size - pad * 2;
  const r = tile * 0.2245;
  const cx = size / 2;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">
  <defs>
    <linearGradient id="tile" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#F4924B"/><stop offset="0.42" stop-color="${C.ember}"/><stop offset="1" stop-color="${C.emberDeep}"/>
    </linearGradient>
    <radialGradient id="glow" cx="0.5" cy="0.42" r="0.62">
      <stop offset="0" stop-color="${C.forgeGlow}" stop-opacity="0.85"/><stop offset="0.55" stop-color="${C.forgeGlow}" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="sheen" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffffff" stop-opacity="0.30"/><stop offset="0.14" stop-color="#ffffff" stop-opacity="0"/>
    </linearGradient>
    <linearGradient id="floor" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0.72" stop-color="#3a1400" stop-opacity="0"/><stop offset="1" stop-color="#3a1400" stop-opacity="0.28"/>
    </linearGradient>
    <linearGradient id="bone" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#FFFFFF"/><stop offset="1" stop-color="${C.boneLow}"/>
    </linearGradient>
    <filter id="lift" x="-30%" y="-30%" width="160%" height="160%">
      <feDropShadow dx="0" dy="7" stdDeviation="11" flood-color="#3a1200" flood-opacity="0.40"/>
    </filter>
    <clipPath id="tileClip"><rect x="${pad}" y="${pad}" width="${tile}" height="${tile}" rx="${r}" ry="${r}"/></clipPath>
  </defs>
  <g clip-path="url(#tileClip)">
    <rect x="${pad}" y="${pad}" width="${tile}" height="${tile}" fill="url(#tile)"/>
    <rect x="${pad}" y="${pad}" width="${tile}" height="${tile}" fill="url(#glow)"/>
    <rect x="${pad}" y="${pad}" width="${tile}" height="${tile}" fill="url(#floor)"/>
    <rect x="${pad}" y="${pad}" width="${tile}" height="${tile}" fill="url(#sheen)"/>
    ${anvilGroup(cx, size / 2 + 10, tile * 0.6, "url(#bone)", 'filter="url(#lift)"')}
  </g>
  <rect x="${pad + 0.75}" y="${pad + 0.75}" width="${tile - 1.5}" height="${tile - 1.5}" rx="${r}" ry="${r}"
        fill="none" stroke="#ffffff" stroke-opacity="0.16" stroke-width="1.5"/>
</svg>`;
}

// ── NSIS sidebar (164×314) — full-bleed forge art, text sits to its right ───
function nsisSidebarSVG(w = 164, h = 314) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  <defs>${commonDefs()}</defs>
  ${forgeBg(w, h, { glowX: 0.5 })}
  ${spark(126, 60, 2.2, 0.85)}${spark(40, 96, 1.6, 0.6)}${spark(140, 150, 1.3, 0.7)}
  ${emberTile(w / 2 - 33, 52, 66)}
  <text x="${w / 2}" y="168" text-anchor="middle" font-family="${FONT}"
        font-size="30" font-weight="700" letter-spacing="-0.5" fill="${C.textHi}">Anvil</text>
  <text x="${w / 2}" y="196" text-anchor="middle" font-family="${FONT}"
        font-size="12.5" font-weight="600" fill="url(#wordAccent)">Practice DSA offline</text>
  <text x="${w / 2}" y="288" text-anchor="middle" font-family="${FONT}"
        font-size="10.5" font-weight="500" fill="${C.textLow}">No internet · no account</text>
</svg>`;
}

// ── NSIS header (150×57) — compact branded chip, top-right of pages ─────────
function nsisHeaderSVG(w = 150, h = 57) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  <defs>${commonDefs()}</defs>
  ${forgeBg(w, h, { glowX: 0.95 })}
  ${emberTile(16, h / 2 - 18, 36, 10)}
  <text x="66" y="${h / 2 + 6}" font-family="${FONT}"
        font-size="19" font-weight="700" letter-spacing="-0.4" fill="${C.textHi}">Anvil</text>
</svg>`;
}

// ── WiX banner (493×58) — light; MSI draws its title text on the left ───────
function wixBannerSVG(w = 493, h = 58) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  <defs>${commonDefs()}
    <linearGradient id="lp" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0" stop-color="${C.panel}"/><stop offset="1" stop-color="#FBF8F3"/>
    </linearGradient>
  </defs>
  <rect width="${w}" height="${h}" fill="url(#lp)"/>
  <rect x="0" y="${h - 2}" width="${w}" height="2" fill="url(#wordAccent)"/>
  <!-- branding parked on the right; installer draws dialog title on the left -->
  <text x="${w - 62}" y="${h / 2 + 6}" text-anchor="end" font-family="${FONT}"
        font-size="19" font-weight="700" letter-spacing="-0.4" fill="${C.panelInk}">Anvil</text>
  ${emberTile(w - 52, h / 2 - 18, 36, 10)}
</svg>`;
}

// ── WiX dialog (493×312) — dark brand column left, light body panel right ───
function wixDialogSVG(w = 493, h = 312) {
  const col = 170;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  <defs>${commonDefs()}</defs>
  <rect width="${w}" height="${h}" fill="${C.panel}"/>
  <g>
    <clipPath id="colClip"><rect x="0" y="0" width="${col}" height="${h}"/></clipPath>
    <g clip-path="url(#colClip)">
      ${forgeBg(col, h, { glowX: 0.5 })}
      ${spark(132, 66, 2, 0.8)}${spark(38, 120, 1.5, 0.6)}
      ${emberTile(col / 2 - 33, 66, 66)}
      <text x="${col / 2}" y="182" text-anchor="middle" font-family="${FONT}"
            font-size="27" font-weight="700" letter-spacing="-0.5" fill="${C.textHi}">Anvil</text>
      <text x="${col / 2}" y="208" text-anchor="middle" font-family="${FONT}"
            font-size="12" font-weight="600" fill="url(#wordAccent)">Practice DSA offline</text>
    </g>
  </g>
  <rect x="${col}" y="0" width="2" height="${h}" fill="url(#wordAccent)" opacity="0.55"/>
</svg>`;
}

// ── macOS DMG background (660×420 window) ───────────────────────────────────
function dmgBackgroundSVG(w = 660, h = 420) {
  const midY = 236; // vertical centre of the drop targets
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  <defs>${commonDefs()}</defs>
  ${forgeBg(w, h, { glowX: 0.88 })}
  ${spark(150, 70, 2.4, 0.85)}${spark(250, 46, 1.6, 0.6)}${spark(90, 120, 1.5, 0.7)}${spark(560, 96, 2, 0.7)}
  <!-- header brand -->
  ${emberTile(40, 40, 46, 13)}
  <text x="100" y="72" font-family="${FONT}" font-size="26" font-weight="700"
        letter-spacing="-0.5" fill="${C.textHi}">Anvil</text>
  <text x="40" y="118" font-family="${FONT}" font-size="14" font-weight="500"
        fill="${C.textMid}">The free, offline, honest way to master DSA.</text>
  <!-- drag arrow between the app icon (~168) and Applications (~492) drop spots -->
  <g opacity="0.9">
    <line x1="250" y1="${midY}" x2="404" y2="${midY}" stroke="url(#wordAccent)" stroke-width="3.5"
          stroke-linecap="round" stroke-dasharray="2 12"/>
    <path d="M402 ${midY - 9} L420 ${midY} L402 ${midY + 9} Z" fill="${C.accentB}"/>
  </g>
  <text x="${w / 2}" y="352" text-anchor="middle" font-family="${FONT}" font-size="13"
        font-weight="600" fill="${C.textLow}">Drag Anvil onto Applications to install</text>
</svg>`;
}

// ── raster helpers ──────────────────────────────────────────────────────────
async function renderPNG(svg, out, w, h = w, scale = 1) {
  const buf = await sharp(Buffer.from(svg), { density: 96 * scale })
    .resize(w, h, { fit: "fill" })
    .png()
    .toBuffer();
  await mkdir(dirname(out), { recursive: true });
  await writeFile(out, buf);
  return buf;
}

/** Flatten an SVG onto `bg` and write a 24-bit (BI_RGB) BMP — what NSIS/WiX want. */
async function renderBMP(svg, out, w, h, bg = "#15110C") {
  const { data } = await sharp(Buffer.from(svg), { density: 192 })
    .resize(w, h, { fit: "fill" })
    .flatten({ background: bg })
    .raw()
    .toBuffer({ resolveWithObject: true }); // RGB, top-down, no alpha
  const rowBytes = w * 3;
  const pad = (4 - (rowBytes % 4)) % 4;
  const stride = rowBytes + pad;
  const pixels = Buffer.alloc(stride * h);
  for (let y = 0; y < h; y++) {
    const srcRow = (h - 1 - y) * rowBytes; // BMP is bottom-up
    const dstRow = y * stride;
    for (let x = 0; x < w; x++) {
      const s = srcRow + x * 3;
      const d = dstRow + x * 3;
      pixels[d] = data[s + 2]; // B
      pixels[d + 1] = data[s + 1]; // G
      pixels[d + 2] = data[s]; // R
    }
  }
  const header = Buffer.alloc(54);
  header.write("BM", 0);
  header.writeUInt32LE(54 + pixels.length, 2);
  header.writeUInt32LE(54, 10); // pixel data offset
  header.writeUInt32LE(40, 14); // DIB header size
  header.writeInt32LE(w, 18);
  header.writeInt32LE(h, 22);
  header.writeUInt16LE(1, 26); // planes
  header.writeUInt16LE(24, 28); // bpp
  header.writeUInt32LE(pixels.length, 34); // image size
  header.writeInt32LE(2835, 38); // 72dpi X
  header.writeInt32LE(2835, 42); // 72dpi Y
  const bmp = Buffer.concat([header, pixels]);
  await mkdir(dirname(out), { recursive: true });
  await writeFile(out, bmp);
  // review copy as PNG (BMP isn't previewable by image viewers here)
  await renderPNG(svg, join(SRC, `review-${out.split(/[\\/]/).pop().replace(".bmp", ".png")}`), w, h, 2);
  return bmp;
}

async function main() {
  await mkdir(SRC, { recursive: true });
  await mkdir(INSTALLER, { recursive: true });

  // 1. master app icon
  const icon = iconSVG({ size: 1024, pad: 100 });
  await writeFile(join(SRC, "icon.svg"), icon);
  await renderPNG(icon, join(ICONS, "icon.png"), 1024);
  await renderPNG(icon, join(SRC, "icon-64.png"), 64);
  console.log("✓ icon        src-tauri/icons/icon.png (1024)");

  // 2. Windows NSIS
  await renderBMP(nsisSidebarSVG(), join(INSTALLER, "nsis-sidebar.bmp"), 164, 314);
  await renderBMP(nsisHeaderSVG(), join(INSTALLER, "nsis-header.bmp"), 150, 57);
  console.log("✓ nsis        installer/nsis-sidebar.bmp (164×314), nsis-header.bmp (150×57)");

  // 3. Windows WiX / MSI
  await renderBMP(wixBannerSVG(), join(INSTALLER, "wix-banner.bmp"), 493, 58, "#F4F0E9");
  await renderBMP(wixDialogSVG(), join(INSTALLER, "wix-dialog.bmp"), 493, 312, "#F4F0E9");
  console.log("✓ wix         installer/wix-banner.bmp (493×58), wix-dialog.bmp (493×312)");

  // 4. macOS DMG background (@1x + retina @2x)
  const dmg = dmgBackgroundSVG();
  await renderPNG(dmg, join(INSTALLER, "dmg-background.png"), 660, 420, 1);
  await renderPNG(dmg, join(INSTALLER, "dmg-background@2x.png"), 1320, 840, 2);
  console.log("✓ dmg         installer/dmg-background.png (660×420) + @2x");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
