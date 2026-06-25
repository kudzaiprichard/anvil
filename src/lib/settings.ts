"use client";

/**
 * Editor preferences (settings modal → workspace editor). Persisted to
 * localStorage; a custom event keeps every consumer in sync within the tab.
 */

import { useSyncExternalStore } from "react";

/** Arrangement of the workspace panes (problem / editor / results). */
export type WorkspaceLayout =
  | "classic" // problem | (editor over results)
  | "mirrored" // (editor over results) | problem
  | "bottom" // (problem | editor) over full-width results
  | "columns" // problem | editor | results, side by side
  | "editor-deck"; // full-width editor over (problem | results)

export const WORKSPACE_LAYOUTS: { id: WorkspaceLayout; label: string }[] = [
  { id: "classic", label: "Classic" },
  { id: "mirrored", label: "Mirrored" },
  { id: "bottom", label: "Bottom dock" },
  { id: "columns", label: "Three columns" },
  { id: "editor-deck", label: "Editor deck" },
];

export interface EditorPrefs {
  fontSize: number;
  tabSize: 2 | 4;
  lineWrap: boolean;
  /** Appearance: minimize transitions / pass celebration. */
  reduceMotion: boolean;
  /** Appearance: workspace pane arrangement. */
  workspaceLayout: WorkspaceLayout;
  /** Workspace: show the per-problem practice stopwatch in the toolbar. */
  showTimer: boolean;
  /** Timer starts automatically when a problem opens (vs. manual start). */
  timerAutoStart: boolean;
  /** Workspace pane sizes, persisted on drag end so the layout survives. */
  paneLeftPct: number;
  paneResultsH: number;
  paneResultsW: number;
  /** WebView zoom factor (Ctrl+= / Ctrl+- / Ctrl+0), Tauri only. */
  uiZoom: number;
}

export const DEFAULT_EDITOR_PREFS: EditorPrefs = {
  fontSize: 13,
  tabSize: 4,
  lineWrap: false,
  reduceMotion: false,
  workspaceLayout: "classic",
  showTimer: true,
  timerAutoStart: true,
  paneLeftPct: 45,
  paneResultsH: 304,
  paneResultsW: 360,
  uiZoom: 1,
};

const STORAGE_KEY = "anvil.editor-prefs";
const CHANGE_EVENT = "anvil:editor-prefs";

let cached: EditorPrefs | null = null;

function load(): EditorPrefs {
  if (cached) return cached;
  if (typeof window === "undefined") return DEFAULT_EDITOR_PREFS;
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    cached = raw
      ? { ...DEFAULT_EDITOR_PREFS, ...(JSON.parse(raw) as Partial<EditorPrefs>) }
      : DEFAULT_EDITOR_PREFS;
  } catch {
    cached = DEFAULT_EDITOR_PREFS;
  }
  return cached;
}

/** One-shot read for non-reactive call sites (state initializers). */
export function getEditorPrefs(): EditorPrefs {
  return load();
}

export function setEditorPrefs(patch: Partial<EditorPrefs>): void {
  cached = { ...load(), ...patch };
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(cached));
  } catch {
    // storage unavailable — keep the in-memory value
  }
  window.dispatchEvent(new Event(CHANGE_EVENT));
}

function subscribe(onChange: () => void): () => void {
  const handler = () => {
    cached = null;
    onChange();
  };
  window.addEventListener(CHANGE_EVENT, handler);
  window.addEventListener("storage", handler);
  return () => {
    window.removeEventListener(CHANGE_EVENT, handler);
    window.removeEventListener("storage", handler);
  };
}

export function useEditorPrefs(): EditorPrefs {
  return useSyncExternalStore(subscribe, load, () => DEFAULT_EDITOR_PREFS);
}
