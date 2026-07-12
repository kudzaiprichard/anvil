# Branding

Source of truth for every installer / store / icon raster Anvil ships. Assets
are generated from code so the whole set stays consistent with the app's
"forged iron & ember" design language (`app/globals.css`) and the hero banner
(`.github/assets/banner.html`).

```bash
npm run branding      # regenerate every asset (needs sharp — present via Next.js)
```

The generator (`generate.mjs`) reuses the **exact** in-app `AnvilMark` geometry
from `src/components/anvil/logo.tsx`, so the dock/taskbar icon matches the mark
shown inside the running app.

## What it produces

| Output | Size | Consumed by |
| --- | --- | --- |
| `../src-tauri/icons/icon.png` | 1024² | `tauri icon` → `.ico`, `.icns`, all PNGs |
| `../src-tauri/installer/nsis-sidebar.bmp` | 164×314 | Windows NSIS welcome/finish panel |
| `../src-tauri/installer/nsis-header.bmp` | 150×57 | Windows NSIS page header |
| `../src-tauri/installer/wix-banner.bmp` | 493×58 | Windows MSI (WiX) top banner |
| `../src-tauri/installer/wix-dialog.bmp` | 493×312 | Windows MSI (WiX) welcome/exit dialog |
| `../src-tauri/installer/dmg-background.png` (+`@2x`) | 660×420 | macOS DMG window background |
| `src/` | — | review-only SVG + PNG previews (`review-*.png`) |

Linux `.deb` / `.rpm` / AppImage reuse the generated PNG icons directly (no
separate installer art), wired through `bundle.linux` in `tauri.conf.json`.

## Regenerating the icon set

After changing `icon.png`, refresh the full platform set:

```bash
npx tauri icon branding/src/master-1024.png -o src-tauri/icons
```

## Editing

Change palette or copy in `generate.mjs` (the `C` palette object and the SVG
builders), rerun `npm run branding`, and eyeball `branding/src/review-*.png`.
BMPs aren't previewable in most image viewers, so a PNG twin is written for each.
The WiX assets keep a light panel where the MSI draws its own (black) text; the
NSIS sidebar and DMG art are full-bleed because their text sits beside the image.
