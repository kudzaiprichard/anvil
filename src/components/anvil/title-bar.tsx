"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import { AnvilMark } from "./logo";

// Whether we're inside the Tauri shell never changes at runtime — expose it
// as a constant external store so SSR/hydration renders nothing and the
// client flips to the real value right after mount.
const subscribeNever = () => () => {};

/**
 * Custom window chrome for the undecorated Tauri window (34px). The whole
 * bar is a drag region (double-click toggles maximize, handled by Tauri);
 * the right edge holds Windows-style minimize / maximize / close controls.
 * Renders nothing in a plain browser (`npm run dev`), where the OS chrome
 * doesn't exist to replace.
 */
export function TitleBar() {
  const inTauri = useSyncExternalStore(subscribeNever, isTauri, () => false);
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (!inTauri) return;
    const win = getCurrentWindow();
    let unlisten: (() => void) | undefined;
    let disposed = false;
    win.isMaximized().then(setMaximized);
    win
      .onResized(() => {
        win.isMaximized().then(setMaximized);
      })
      .then((un) => {
        if (disposed) un();
        else unlisten = un;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [inTauri]);

  if (!inTauri) return null;

  const win = getCurrentWindow();
  const controlBtn =
    "inline-flex h-full w-[44px] items-center justify-center text-muted-foreground transition-colors hover:bg-accent hover:text-foreground";

  return (
    <header
      data-tauri-drag-region
      className="flex h-[34px] shrink-0 select-none items-stretch border-b bg-sidebar"
    >
      <div className="pointer-events-none flex items-center gap-2 pl-3">
        <AnvilMark className="size-[13px] text-primary" />
        <span className="text-[12px] font-semibold tracking-tight">Anvil</span>
        <span className="microlabel hidden sm:inline">offline forge</span>
      </div>

      <div data-tauri-drag-region className="flex-1" />

      <div className="flex items-stretch">
        <button
          type="button"
          aria-label="Minimize"
          onClick={() => win.minimize()}
          className={controlBtn}
        >
          <Minus className="size-[14px]" strokeWidth={1.6} />
        </button>
        <button
          type="button"
          aria-label={maximized ? "Restore" : "Maximize"}
          onClick={() => win.toggleMaximize()}
          className={controlBtn}
        >
          {maximized ? (
            <Copy className="size-[11px] -scale-x-100" strokeWidth={1.6} />
          ) : (
            <Square className="size-[11px]" strokeWidth={1.6} />
          )}
        </button>
        <button
          type="button"
          aria-label="Close"
          onClick={() => win.close()}
          className="inline-flex h-full w-[44px] items-center justify-center text-muted-foreground transition-colors hover:bg-[#c42b1c] hover:text-white"
        >
          <X className="size-[15px]" strokeWidth={1.6} />
        </button>
      </div>
    </header>
  );
}
