"use client";

import { useEffect, useRef } from "react";

export interface WorkspaceShortcutHandlers {
  /** Ctrl+Enter (Ctrl+' kept as a legacy alias). */
  onRun: () => void;
  /** Ctrl+[ */
  onPrev: () => void;
  /** Ctrl+] */
  onNext: () => void;
  /** Ctrl+P */
  onToggleList: () => void;
  /** Ctrl+Shift+M */
  onToggleMaximize: () => void;
  /** Ctrl+Alt+R */
  onReset: () => void;
}

/**
 * Window-level key bindings for the workspace — exactly the list the
 * Settings → Shortcuts pane advertises (app-wide chords live in
 * `use-app-shortcuts.ts`). Every binding is a Ctrl-chord, so plain typing
 * in the CodeMirror editor is never intercepted; preventDefault stops the
 * browser/WebView defaults (Ctrl+P print dialog etc.).
 */
export function useWorkspaceShortcuts(handlers: WorkspaceShortcutHandlers) {
  const handlersRef = useRef(handlers);
  useEffect(() => {
    handlersRef.current = handlers;
  });

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (!e.ctrlKey || e.metaKey) return;
      const h = handlersRef.current;
      const key = e.key.toLowerCase();
      if (e.altKey) {
        if (key === "r") {
          e.preventDefault();
          h.onReset();
        }
        return;
      }
      if (e.shiftKey) {
        if (key === "m") {
          e.preventDefault();
          h.onToggleMaximize();
        }
        return;
      }
      switch (key) {
        case "enter":
        case "'":
          e.preventDefault();
          h.onRun();
          break;
        case "[":
          e.preventDefault();
          h.onPrev();
          break;
        case "]":
          e.preventDefault();
          h.onNext();
          break;
        case "p":
          e.preventDefault();
          h.onToggleList();
          break;
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
