/**
 * Real backend: each export is the `invoke()` twin of a mock function, same
 * name and signature (BACKEND_PLAN §7). `index.ts` routes here when running
 * inside Tauri. Invoke rejections carry the serialized `AppError`
 * (`{ kind, message }`) — normalize them to `Error(message)` so existing
 * `toast.error(err.message)` call sites keep working.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
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
  ValidationResult,
} from "@/src/lib/types";
import { applyProblemFilter } from "@/src/lib/api/filters";

export { paramNames } from "@/src/lib/api/mock";

function normalizeError(e: unknown): Error {
  if (e instanceof Error) return e;
  if (typeof e === "object" && e !== null && "message" in e) {
    return new Error(String((e as { message: unknown }).message));
  }
  return new Error(String(e));
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw normalizeError(e);
  }
}

/** The backend sends `lastAttempted` as a local ISO timestamp — turn it
 * into the short labels the table was designed around ("3h ago", "2d ago"). */
function formatRelative(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return iso;
  const mins = Math.max(0, Math.round((Date.now() - then) / 60_000));
  if (mins < 60) return mins <= 1 ? "just now" : `${mins}m ago`;
  const hours = Math.round(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.round(hours / 24);
  if (days === 1) return "yesterday";
  if (days < 7) return `${days}d ago`;
  const weeks = Math.round(days / 7);
  return weeks < 5 ? `${weeks}w ago` : `${Math.round(days / 30)}mo ago`;
}

export async function listProblems(
  filter?: ProblemFilter
): Promise<ProblemSummary[]> {
  const rows = await call<ProblemSummary[]>("list_problems");
  for (const row of rows) {
    if (row.lastAttempted) row.lastAttempted = formatRelative(row.lastAttempted);
  }
  return applyProblemFilter(rows, filter);
}

export async function getProblem(id: string): Promise<Problem | null> {
  return (await call<Problem | null>("get_problem", { id })) ?? null;
}

export async function runCode(req: RunRequest): Promise<RunResult> {
  return call<RunResult>("run_code", { req });
}

export async function submitCode(req: RunRequest): Promise<RunResult> {
  return call<RunResult>("submit_code", { req });
}

export async function detectRuntimes(): Promise<RuntimeInfo[]> {
  return call<RuntimeInfo[]>("detect_runtimes");
}

export async function getProgress(): Promise<Progress> {
  return call<Progress>("get_progress");
}

export async function getDashboard(): Promise<DashboardData> {
  const data = await call<DashboardData>("get_dashboard");
  if (data.continueProblem?.lastAttempted) {
    data.continueProblem.lastAttempted = formatRelative(
      data.continueProblem.lastAttempted
    );
  }
  return data;
}

export async function setProblemStatus(
  id: string,
  action: StatusAction
): Promise<void> {
  return call<void>("set_problem_status", { problemId: id, action });
}

export async function toggleBookmark(id: string): Promise<boolean> {
  return call<boolean>("toggle_bookmark", { problemId: id });
}

export async function getProblemUserState(
  id: string
): Promise<ProblemUserState> {
  return call<ProblemUserState>("get_problem_user_state", { problemId: id });
}

export async function validateUserProblem(
  draft: UserProblemDraft
): Promise<ValidationResult> {
  return call<ValidationResult>("validate_user_problem", { draft });
}

export async function saveUserProblem(
  draft: UserProblemDraft
): Promise<Problem> {
  return call<Problem>("save_user_problem", { draft });
}

export async function saveDraft(
  draft: UserProblemDraft,
  draftId?: string
): Promise<string> {
  return call<string>("save_draft", { draft, draftId: draftId ?? null });
}

export async function listDrafts(): Promise<DraftSummary[]> {
  return call<DraftSummary[]>("list_drafts");
}

export async function getDraft(
  id: string
): Promise<UserProblemDraft | null> {
  return (await call<UserProblemDraft | null>("get_draft", { id })) ?? null;
}

export async function deleteDraft(id: string): Promise<void> {
  return call<void>("delete_draft", { id });
}

/** `false` when the user cancels the save dialog. */
export async function exportProblem(id: string): Promise<boolean> {
  return call<boolean>("export_problem", { id });
}

/** Exports all user + imported problems as one pack. `false` on cancel. */
export async function exportPack(): Promise<boolean> {
  return call<boolean>("export_pack");
}

/** Imports a single-problem file or a multi-problem pack (autodetected).
 *  `null` when the user cancels the open dialog. */
export async function importProblems(): Promise<Problem[] | null> {
  return (await call<Problem[] | null>("import_problems")) ?? null;
}
