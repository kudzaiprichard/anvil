/**
 * Client-side filtering/sorting shared by both seam backends (mock and
 * Tauri). The Rust side always returns the full annotated list
 * (BACKEND_PLAN §2.1) — this is the single place list shaping happens.
 */

import type {
  Difficulty,
  ProblemFilter,
  ProblemSummary,
} from "@/src/lib/types";

const DIFF_ORDER: Record<Difficulty, number> = { Easy: 0, Medium: 1, Hard: 2 };

export function applyProblemFilter(
  rows: ProblemSummary[],
  filter?: ProblemFilter
): ProblemSummary[] {
  let out = [...rows];
  if (filter?.search) {
    const q = filter.search.toLowerCase();
    out = out.filter(
      (r) => r.title.toLowerCase().includes(q) || String(r.number).includes(q)
    );
  }
  if (filter?.pattern) out = out.filter((r) => r.pattern === filter.pattern);
  if (filter?.difficulty)
    out = out.filter((r) => r.difficulty === filter.difficulty);
  if (filter?.status) out = out.filter((r) => r.status === filter.status);
  switch (filter?.sort) {
    case "difficulty":
      out.sort(
        (a, b) =>
          DIFF_ORDER[a.difficulty] - DIFF_ORDER[b.difficulty] ||
          a.number - b.number
      );
      break;
    case "recent":
      out.sort(
        (a, b) =>
          Number(b.lastAttempted !== undefined) -
          Number(a.lastAttempted !== undefined)
      );
      break;
    default:
      out.sort((a, b) => a.number - b.number);
  }
  return out;
}
