/**
 * Real backend: each export is the `invoke()` twin of a mock function, same
 * name and signature (BACKEND_PLAN §7). `index.ts` routes here when running
 * inside Tauri. Invoke rejections carry the serialized `AppError`
 * (`{ kind, message }`) — normalize them to `Error(message)` so existing
 * `toast.error(err.message)` call sites keep working.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  CapstoneOutcome,
  CapstoneView,
  ComplexityReport,
  Curriculum,
  DashboardData,
  DraftSummary,
  GateOutcome,
  Lesson,
  LessonProgress,
  LessonStatus,
  PlacementOutcome,
  PlacementProbe,
  Problem,
  ProblemFilter,
  ProblemSummary,
  ProblemUserState,
  Progress,
  Quiz,
  QuizAnswer,
  QuizGrade,
  Readiness,
  ReviewOutcome,
  ReviewQueue,
  ReviewRating,
  RunRequest,
  RunResult,
  RuntimeInfo,
  StatusAction,
  Unit,
  UnitProgress,
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

/** Phase 1 IPC stubs — no UI reads these yet. */
export async function getCurriculum(): Promise<Curriculum> {
  return call<Curriculum>("get_curriculum");
}

export async function getUnit(id: string): Promise<Unit | null> {
  return (await call<Unit | null>("get_unit", { id })) ?? null;
}

export async function getLesson(id: string): Promise<Lesson | null> {
  return (await call<Lesson | null>("get_lesson", { id })) ?? null;
}

export async function getQuiz(lessonId: string): Promise<Quiz | null> {
  return (await call<Quiz | null>("get_quiz", { lessonId })) ?? null;
}

export async function getPatternPool(): Promise<Quiz> {
  return call<Quiz>("get_pattern_pool");
}

/** Grades a formative quiz submission and records the review signal. `source`
 *  is a lesson id or `PATTERN_POOL_SOURCE`. Never blocks progression. */
export async function submitQuiz(
  source: string,
  answers: QuizAnswer[]
): Promise<QuizGrade> {
  return call<QuizGrade>("submit_quiz", { source, answers });
}

export async function recordLessonProgress(
  lessonId: string,
  status: LessonStatus
): Promise<void> {
  return call<void>("record_lesson_progress", { lessonId, status });
}

export async function getLessonProgress(): Promise<LessonProgress[]> {
  return call<LessonProgress[]>("get_lesson_progress");
}

export async function getProgression(): Promise<UnitProgress[]> {
  return call<UnitProgress[]>("get_progression");
}

export async function evaluateGate(
  unitId: string,
  problemId: string,
  usedHelp: boolean
): Promise<GateOutcome> {
  return call<GateOutcome>("evaluate_gate", { unitId, problemId, usedHelp });
}

/** Phase 7: the Stage-7 mixed capstone (unlabeled cross-unit pool). */
export async function getCapstone(): Promise<CapstoneView | null> {
  return (await call<CapstoneView | null>("get_capstone")) ?? null;
}

/** Scores one capstone attempt (peeked attempts never count). */
export async function evaluateCapstone(
  problemId: string,
  usedHelp: boolean
): Promise<CapstoneOutcome> {
  return call<CapstoneOutcome>("evaluate_capstone", { problemId, usedHelp });
}

/** The diagnostic placement probe (unlabeled recognition items). */
export async function getPlacement(): Promise<PlacementProbe> {
  return call<PlacementProbe>("get_placement");
}

/** Applies the placement probe — places the learner out of recognized units. */
export async function applyPlacement(
  answers: QuizAnswer[]
): Promise<PlacementOutcome> {
  return call<PlacementOutcome>("apply_placement", { answers });
}

/** The honest course-readiness signal (ladder mastery + capstone). */
export async function getReadiness(): Promise<Readiness> {
  return call<Readiness>("get_readiness");
}

/** The FSRS spaced-review queue: Stage-1 problems due to re-solve cold now
 *  (interleaved) + the honest habit header (Phase 6). */
export async function getReviewQueue(): Promise<ReviewQueue> {
  return call<ReviewQueue>("get_review_queue");
}

/** Records a cold re-solve and reschedules the card. `rating` is self-assessed
 *  recall; `again` demotes. */
export async function recordReview(
  problemId: string,
  rating: ReviewRating
): Promise<ReviewOutcome> {
  return call<ReviewOutcome>("record_review", { problemId, rating });
}

export async function runCode(req: RunRequest): Promise<RunResult> {
  return call<RunResult>("run_code", { req });
}

export async function submitCode(req: RunRequest): Promise<RunResult> {
  return call<RunResult>("submit_code", { req });
}

/** Deterministic complexity feedback: profiles the learner's Python solution on
 *  growing inputs and compares to the pack's optimal (Phase 5, no AI). */
export async function analyzeComplexity(
  req: RunRequest
): Promise<ComplexityReport> {
  return call<ComplexityReport>("analyze_complexity", { req });
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
