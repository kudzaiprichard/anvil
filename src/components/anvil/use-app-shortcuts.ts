"use client";

import { useEffect, useRef } from "react";

export interface AppShortcutHandlers {
  /** Ctrl+K */
  onOpenPalette: () => void;
  /** Ctrl+, */
  onOpenSettings: () => void;
  /** Ctrl+Shift+L */
  onCycleLayout: () => void;
  /** Ctrl+Shift+T */
  onToggleTheme: () => void;
  /** Ctrl+1 / Ctrl+2 / Ctrl+3 — Dashboard / Library / Forge. */
  onNavigate: (index: 0 | 1 | 2) => void;
  /** Ctrl+= / Ctrl+- zoom in/out, Ctrl+0 reset (Tauri WebView only). */
  onZoom: (delta: 1 | -1 | 0) => void;
}

/**
 * App-wide key bindings, active on every screen (the workspace adds its own
 * on top — see `use-workspace-shortcuts.ts`). Every binding is a Ctrl-chord
 * so plain typing is never intercepted; the two hooks share no chords.
 */
export function useAppShortcuts(handlers: AppShortcutHandlers) {
  const handlersRef = useRef(handlers);
  useEffect(() => {
    handlersRef.current = handlers;
  });

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (!e.ctrlKey || e.metaKey || e.altKey) return;
      const h = handlersRef.current;
      const key = e.key.toLowerCase();
      if (e.shiftKey) {
        if (key === "l") {
          // Inside CodeMirror, Ctrl+Shift+L is "select all occurrences" —
          // the editor owns it there.
          const target = e.target as HTMLElement | null;
          if (target?.closest?.(".cm-editor")) return;
          e.preventDefault();
          h.onCycleLayout();
        } else if (key === "t") {
          e.preventDefault();
          h.onToggleTheme();
        }
        return;
      }
      if (key === "k") {
        e.preventDefault();
        h.onOpenPalette();
      } else if (key === ",") {
        e.preventDefault();
        h.onOpenSettings();
      } else if (key === "1" || key === "2" || key === "3") {
        e.preventDefault();
        h.onNavigate((Number(key) - 1) as 0 | 1 | 2);
      } else if (key === "=" || key === "+") {
        e.preventDefault();
        h.onZoom(1);
      } else if (key === "-") {
        e.preventDefault();
        h.onZoom(-1);
      } else if (key === "0") {
        e.preventDefault();
        h.onZoom(0);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
