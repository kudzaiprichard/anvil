"use client";

/**
 * Latest solve time per problem — recorded by the practice timer the moment
 * a run passes the full test suite. Session-independent (localStorage) so
 * the timer chip can show what your previous solve took.
 */

const PREFIX = "anvil.solvetime";

export function loadSolveTime(problemId: string): number | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = window.localStorage.getItem(`${PREFIX}.${problemId}`);
    const parsed = raw === null ? NaN : Number(raw);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
  } catch {
    return null;
  }
}

export function saveSolveTime(problemId: string, seconds: number): void {
  try {
    window.localStorage.setItem(`${PREFIX}.${problemId}`, String(seconds));
  } catch {
    // storage unavailable — the timer still shows the session value
  }
}
