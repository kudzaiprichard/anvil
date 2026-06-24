/**
 * Mock data layer — the browser-dev fallback and the contract reference for
 * the Rust commands. `index.ts` routes here whenever the app is not running
 * inside Tauri, so `npm run dev` keeps working without a backend.
 *
 * Mock run behavior (so every workspace state is reproducible):
 *   - code containing `raise` / `throw`            → runtime error
 *   - code containing `while True` / `while (true)`→ timeout
 *   - unchanged starter code (or `pass` / empty)   → wrong-answer fail
 *   - anything else                                → all tests pass
 */

import type {
  CaseResult,
  DashboardData,
  DraftSummary,
  Problem,
  ProblemFilter,
  ProblemSummary,
  ProblemUserState,
  Progress,
  RunRequest,
  RunResult,
  RuntimeInfo,
  StatusAction,
  UserProblemDraft,
  ValidationIssue,
  ValidationResult,
} from "@/src/lib/types";
import { PATTERNS } from "@/src/lib/types";
import { applyProblemFilter } from "@/src/lib/api/filters";
import { MOCK_PROBLEMS } from "@/src/lib/mock/problems";
import {
  MOCK_PROGRESS,
  buildDashboard,
  problemSummaries,
} from "@/src/lib/mock/progress";

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

/** User-authored problems saved this session (in-memory until SQLite lands). */
const userProblems: Problem[] = [];

function allProblems(): Problem[] {
  return [...MOCK_PROBLEMS, ...userProblems];
}

export async function listProblems(
  filter?: ProblemFilter
): Promise<ProblemSummary[]> {
  await delay(120);
  const rows: ProblemSummary[] = [
    ...problemSummaries(),
    ...userProblems.map((p) => ({
      id: p.id,
      number: p.number,
      title: p.title,
      pattern: p.pattern,
      difficulty: p.difficulty,
      source: p.source,
      status: "todo" as const,
    })),
  ];
  return applyProblemFilter(rows, filter);
}

export async function getProblem(id: string): Promise<Problem | null> {
  await delay(150);
  return allProblems().find((p) => p.id === id) ?? null;
}

function fmtValue(v: unknown): string {
  return JSON.stringify(v)?.replace(/,/g, ", ") ?? "null";
}

function fmtInput(p: Problem, caseIndex: number): string {
  const tc = p.test_cases[caseIndex];
  const params = paramNames(p);
  return tc.input
    .map((arg, i) => `${params[i] ?? `arg${i}`}=${JSON.stringify(arg)}`)
    .join(", ");
}

/** Parameter names parsed from the python signature, for display. */
export function paramNames(p: Problem): string[] {
  const m = p.function_signature.python.match(/def\s+\w+\(([^)]*)\)/);
  if (!m) return [];
  return m[1]
    .split(",")
    .map((s) => s.split(":")[0].trim())
    .filter(Boolean);
}

function executeMock(
  p: Problem,
  code: string,
  includeHidden: boolean
): RunResult {
  const cases = p.test_cases.filter((tc) => includeHidden || !tc.hidden);
  const lower = code.toLowerCase();

  if (/\b(raise|throw)\b/.test(lower)) {
    const line = code.split("\n").findIndex((l) => /\b(raise|throw)\b/i.test(l)) + 1;
    return {
      status: "error",
      cases: [],
      passed: 0,
      total: cases.length,
      error: `Traceback (most recent call last):\n  File "solution.py", line ${line}, in solve\nTypeError: list indices must be integers or slices, not str`,
    };
  }
  if (/while\s+true|while\s*\(\s*true\s*\)/.test(lower)) {
    return {
      status: "timeout",
      cases: [],
      passed: 0,
      total: cases.length,
      error: "Time limit exceeded — execution stopped after 3000 ms.",
    };
  }

  const starter = p.function_signature[
    /function\s+\w+/.test(code) ? "javascript" : "python"
  ];
  const unchanged =
    code.trim() === starter.trim() ||
    code.trim().length === 0 ||
    /^\s*(def\s+\w+\([^)]*\):\s*(#[^\n]*\n)?\s*pass\s*)$/.test(code.trim());

  const results: CaseResult[] = cases.map((tc, i) => {
    const caseIndex = p.test_cases.indexOf(tc);
    const failed = unchanged && i === Math.min(1, cases.length - 1);
    const base: CaseResult = {
      index: i + 1,
      hidden: tc.hidden,
      passed: !failed,
    };
    if (!tc.hidden) {
      base.input = fmtInput(p, caseIndex);
      base.expected = fmtValue(tc.expected);
      base.output = failed ? wrongAnswer(tc.expected) : fmtValue(tc.expected);
    }
    return base;
  });

  const passed = results.filter((r) => r.passed).length;
  return {
    status: passed === results.length ? "pass" : "fail",
    cases: results,
    passed,
    total: results.length,
    runtimeMs: 41,
    memoryMb: 16.2,
  };
}

/** A plausible-looking wrong output derived from the expected value. */
function wrongAnswer(expected: unknown): string {
  if (Array.isArray(expected)) return fmtValue([...expected].reverse());
  if (typeof expected === "number") return fmtValue(expected + 1);
  if (typeof expected === "boolean") return fmtValue(!expected);
  return "null";
}

export async function runCode(req: RunRequest): Promise<RunResult> {
  await delay(1100);
  const p = allProblems().find((x) => x.id === req.id);
  if (!p) throw new Error(`Unknown problem: ${req.id}`);
  return executeMock(p, req.code, false);
}

export async function submitCode(req: RunRequest): Promise<RunResult> {
  await delay(1600);
  const p = allProblems().find((x) => x.id === req.id);
  if (!p) throw new Error(`Unknown problem: ${req.id}`);
  return executeMock(p, req.code, true);
}

/** Session-local UI-state echoes so browser dev behaves like the real app. */
const mockUserState = new Map<string, ProblemUserState>();

function userStateFor(id: string): ProblemUserState {
  return (
    mockUserState.get(id) ?? { status: "todo", bookmarked: false, mastered: false }
  );
}

export async function setProblemStatus(
  id: string,
  action: StatusAction
): Promise<void> {
  await delay(120);
  const s = userStateFor(id);
  if (action === "mark_mastered") s.mastered = true;
  if (action === "unmark_mastered") s.mastered = false;
  if (action === "needs_review") s.status = "needs-review";
  if (action === "clear_review" && s.status === "needs-review") s.status = "solved";
  mockUserState.set(id, s);
}

export async function toggleBookmark(id: string): Promise<boolean> {
  await delay(120);
  const s = userStateFor(id);
  s.bookmarked = !s.bookmarked;
  mockUserState.set(id, s);
  return s.bookmarked;
}

export async function getProblemUserState(
  id: string
): Promise<ProblemUserState> {
  await delay(80);
  return userStateFor(id);
}

export async function detectRuntimes(): Promise<RuntimeInfo[]> {
  await delay(350);
  return [
    { tag: "Py", name: "Python", path: "C:\\Python312\\python.exe", version: "v3.12.1", found: true },
    { tag: "JS", name: "Node.js", path: "C:\\Program Files\\nodejs\\node.exe", version: "v20.11.0", found: true },
  ];
}

export async function getProgress(): Promise<Progress> {
  await delay(80);
  return MOCK_PROGRESS;
}

export async function getDashboard(): Promise<DashboardData> {
  await delay(180);
  return buildDashboard();
}

export async function validateUserProblem(
  draft: UserProblemDraft
): Promise<ValidationResult> {
  await delay(900);
  const issues: ValidationIssue[] = [];

  if (!draft.title.trim()) issues.push({ field: "Title", message: "is required" });
  if (!draft.pattern || !PATTERNS.includes(draft.pattern))
    issues.push({ field: "Pattern", message: "pick one of the 15 patterns" });
  if (!draft.description_md.trim())
    issues.push({ field: "Description", message: "is required" });
  if (draft.examples.length === 0)
    issues.push({ field: "Examples", message: "at least one is required" });
  draft.examples.forEach((ex, i) => {
    if (!ex.input.trim() || !ex.output.trim())
      issues.push({
        field: `Example ${i + 1}`,
        message: "needs both input and output",
      });
  });
  if (!draft.function_signature.python.trim())
    issues.push({ field: "Python signature", message: "is empty" });
  if (!draft.function_signature.javascript.trim())
    issues.push({ field: "JavaScript signature", message: "is empty" });

  let visible = 0;
  let hidden = 0;
  draft.test_cases.forEach((tc, i) => {
    for (const [label, raw] of [
      ["Input", tc.input],
      ["Expected", tc.expected],
    ] as const) {
      try {
        JSON.parse(raw);
      } catch {
        issues.push({
          field: `Test case ${i + 1}`,
          message: `${label} isn't valid JSON`,
        });
      }
    }
    if (tc.hidden) hidden++;
    else visible++;
  });
  if (visible < 1)
    issues.push({ field: "Test cases", message: "need at least 1 visible test" });
  if (hidden < 1)
    issues.push({ field: "Test cases", message: "need at least 1 hidden test" });
  if (!draft.originalityWarranty)
    issues.push({
      field: "Originality warranty",
      message: "must be accepted to save",
    });

  if (issues.length > 0) return { ok: false, issues };

  // Simulate running the reference solution against the test cases.
  const hasSolution =
    Boolean(draft.reference_solution.python?.trim()) ||
    Boolean(draft.reference_solution.javascript?.trim());
  const caseResults: CaseResult[] = draft.test_cases.map((tc, i) => ({
    index: i + 1,
    hidden: tc.hidden,
    passed: true,
    ...(tc.hidden
      ? {}
      : { input: tc.input, expected: tc.expected, output: tc.expected }),
  }));
  return { ok: true, issues: [], caseResults: hasSolution ? caseResults : undefined };
}

export async function saveUserProblem(
  draft: UserProblemDraft
): Promise<Problem> {
  await delay(400);
  const existing = draft.id
    ? userProblems.find((p) => p.id === draft.id)
    : undefined;
  const id =
    draft.id ??
    `${draft.title.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")}-${Date.now() % 10000}`;
  const problem: Problem = {
    id,
    number:
      existing?.number ??
      Math.max(...MOCK_PROBLEMS.map((p) => p.number), ...userProblems.map((p) => p.number), 0) + 1,
    title: draft.title.trim(),
    pattern: draft.pattern as Problem["pattern"],
    difficulty: draft.difficulty,
    source: "user",
    description_md: draft.description_md,
    constraints: draft.constraints.filter((c) => c.trim()),
    examples: draft.examples,
    function_signature: draft.function_signature,
    test_cases: draft.test_cases.map((tc) => ({
      input: JSON.parse(tc.input) as unknown[],
      expected: JSON.parse(tc.expected),
      hidden: tc.hidden,
    })),
    hints: draft.hints.filter((h) => h.trim()),
    reference_solution: draft.reference_solution,
    follow_up: draft.follow_up?.trim() || undefined,
    license: "user-original",
    author: "user",
  };
  if (existing) {
    userProblems[userProblems.indexOf(existing)] = problem;
  } else {
    userProblems.push(problem);
  }
  return problem;
}

/* ---------- drafts (browser dev: localStorage-backed) ---------- */

const DRAFTS_KEY = "anvil.mock.drafts";

type StoredDraft = { draft: UserProblemDraft; updatedAt: string };

function readDrafts(): Record<string, StoredDraft> {
  if (typeof window === "undefined") return {};
  try {
    return JSON.parse(window.localStorage.getItem(DRAFTS_KEY) ?? "{}");
  } catch {
    return {};
  }
}

function writeDrafts(drafts: Record<string, StoredDraft>) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(DRAFTS_KEY, JSON.stringify(drafts));
}

export async function saveDraft(
  draft: UserProblemDraft,
  draftId?: string
): Promise<string> {
  await delay(150);
  const id = draftId ?? `draft-${Date.now()}`;
  const drafts = readDrafts();
  drafts[id] = { draft, updatedAt: new Date().toISOString() };
  writeDrafts(drafts);
  return id;
}

export async function listDrafts(): Promise<DraftSummary[]> {
  await delay(80);
  return Object.entries(readDrafts())
    .map(([id, { draft, updatedAt }]) => ({
      id,
      title: draft.title.trim() || "Untitled draft",
      updatedAt,
    }))
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
}

export async function getDraft(
  id: string
): Promise<UserProblemDraft | null> {
  await delay(80);
  return readDrafts()[id]?.draft ?? null;
}

export async function deleteDraft(id: string): Promise<void> {
  await delay(80);
  const drafts = readDrafts();
  delete drafts[id];
  writeDrafts(drafts);
}

/* ---------- import/export (desktop-only: OS dialogs) ---------- */

export async function exportProblem(id: string): Promise<boolean> {
  void id;
  await delay(100);
  return false; // browser dev: no OS dialogs — silently unavailable
}

export async function exportPack(): Promise<boolean> {
  await delay(100);
  return false; // browser dev: no OS dialogs — silently unavailable
}

export async function importProblems(): Promise<Problem[] | null> {
  await delay(100);
  return null; // browser dev: no OS dialogs — silently unavailable
}
