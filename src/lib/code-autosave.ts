"use client";

/**
 * Editor autosave — the workspace saves in-progress code per problem +
 * language (debounced) so navigating away never loses work. Restores take
 * priority over the last *run* snapshot from the backend, which only updates
 * when the user actually executes. localStorage is plenty: solutions are a
 * few KB and the newest write always wins.
 */

import type { Language } from "@/src/lib/types";

const PREFIX = "anvil.autosave";

function key(problemId: string, language: Language): string {
  return `${PREFIX}.${problemId}.${language}`;
}

export function loadAutosave(
  problemId: string,
  language: Language
): string | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage.getItem(key(problemId, language));
  } catch {
    return null;
  }
}

export function saveAutosave(
  problemId: string,
  language: Language,
  code: string
): void {
  try {
    window.localStorage.setItem(key(problemId, language), code);
  } catch {
    // storage full/unavailable — losing autosave beats crashing the editor
  }
}

export function clearAutosave(problemId: string, language: Language): void {
  try {
    window.localStorage.removeItem(key(problemId, language));
  } catch {
    // ignore
  }
}
