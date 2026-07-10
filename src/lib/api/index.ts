/**
 * THE seam between the UI and the data layer. Inside Tauri every function
 * routes to the Rust backend (`tauri.ts`); in a plain browser (`npm run
 * dev`) it falls back to the mock so UI iteration needs no backend.
 * Signatures and return types are frozen — screens never import mock or
 * tauri modules directly, only this one.
 */

import { isTauri } from "@tauri-apps/api/core";
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
import * as mock from "@/src/lib/api/mock";
import * as backend from "@/src/lib/api/tauri";

export { paramNames } from "@/src/lib/api/mock";

/**
 * Bundled course *content* — the curriculum, units, lessons, quizzes, and the
 * pattern pool — is immutable for the life of the process: it is glob-loaded and
 * validated once at startup and never mutated at runtime. The course UI hops
 * course → unit → lesson and back, and each view independently needs the
 * curriculum plus the full unit set (to resolve prereq / pattern-reveal titles).
 * Without a cache that is a fresh fan-out of ~19 unit reads on every hop. Memoize
 * the in-flight promise per (getter, key) so repeat reads are free and
 * concurrent reads dedupe onto one backend call. Per-user state (progress,
 * gates, reviews, readiness, the capstone view) is deliberately NOT cached here
 * — only pure, immutable content is.
 */
const contentCache = new Map<string, Promise<unknown>>();
function cachedContent<T>(key: string, load: () => Promise<T>): Promise<T> {
  const hit = contentCache.get(key) as Promise<T> | undefined;
  if (hit) return hit;
  const p = load();
  contentCache.set(key, p);
  // A failed load must not poison the cache — drop it so a later call retries.
  p.catch(() => {
    if (contentCache.get(key) === p) contentCache.delete(key);
  });
  return p;
}

export async function listProblems(
  filter?: ProblemFilter
): Promise<ProblemSummary[]> {
  return isTauri() ? backend.listProblems(filter) : mock.listProblems(filter);
}

export async function getProblem(id: string): Promise<Problem | null> {
  return isTauri() ? backend.getProblem(id) : mock.getProblem(id);
}

export async function getCurriculum(): Promise<Curriculum> {
  return cachedContent("curriculum", () =>
    isTauri() ? backend.getCurriculum() : mock.getCurriculum()
  );
}

export async function getUnit(id: string): Promise<Unit | null> {
  return cachedContent(`unit:${id}`, () =>
    isTauri() ? backend.getUnit(id) : mock.getUnit(id)
  );
}

export async function getLesson(id: string): Promise<Lesson | null> {
  return cachedContent(`lesson:${id}`, () =>
    isTauri() ? backend.getLesson(id) : mock.getLesson(id)
  );
}

export async function getQuiz(lessonId: string): Promise<Quiz | null> {
  return cachedContent(`quiz:${lessonId}`, () =>
    isTauri() ? backend.getQuiz(lessonId) : mock.getQuiz(lessonId)
  );
}

export async function getPatternPool(): Promise<Quiz> {
  return cachedContent("patternPool", () =>
    isTauri() ? backend.getPatternPool() : mock.getPatternPool()
  );
}

export async function submitQuiz(
  source: string,
  answers: QuizAnswer[]
): Promise<QuizGrade> {
  return isTauri()
    ? backend.submitQuiz(source, answers)
    : mock.submitQuiz(source, answers);
}

export async function recordLessonProgress(
  lessonId: string,
  status: LessonStatus
): Promise<void> {
  return isTauri()
    ? backend.recordLessonProgress(lessonId, status)
    : mock.recordLessonProgress(lessonId, status);
}

export async function getLessonProgress(): Promise<LessonProgress[]> {
  return isTauri() ? backend.getLessonProgress() : mock.getLessonProgress();
}

export async function getProgression(): Promise<UnitProgress[]> {
  return isTauri() ? backend.getProgression() : mock.getProgression();
}

export async function evaluateGate(
  unitId: string,
  problemId: string,
  usedHelp: boolean
): Promise<GateOutcome> {
  return isTauri()
    ? backend.evaluateGate(unitId, problemId, usedHelp)
    : mock.evaluateGate(unitId, problemId, usedHelp);
}

export async function getCapstone(): Promise<CapstoneView | null> {
  return isTauri() ? backend.getCapstone() : mock.getCapstone();
}

export async function evaluateCapstone(
  problemId: string,
  usedHelp: boolean
): Promise<CapstoneOutcome> {
  return isTauri()
    ? backend.evaluateCapstone(problemId, usedHelp)
    : mock.evaluateCapstone(problemId, usedHelp);
}

export async function getPlacement(): Promise<PlacementProbe> {
  return isTauri() ? backend.getPlacement() : mock.getPlacement();
}

export async function applyPlacement(
  answers: QuizAnswer[]
): Promise<PlacementOutcome> {
  return isTauri() ? backend.applyPlacement(answers) : mock.applyPlacement(answers);
}

export async function getReadiness(): Promise<Readiness> {
  return isTauri() ? backend.getReadiness() : mock.getReadiness();
}

export async function getReviewQueue(): Promise<ReviewQueue> {
  return isTauri() ? backend.getReviewQueue() : mock.getReviewQueue();
}

export async function recordReview(
  problemId: string,
  rating: ReviewRating
): Promise<ReviewOutcome> {
  return isTauri()
    ? backend.recordReview(problemId, rating)
    : mock.recordReview(problemId, rating);
}

export async function runCode(req: RunRequest): Promise<RunResult> {
  return isTauri() ? backend.runCode(req) : mock.runCode(req);
}

export async function submitCode(req: RunRequest): Promise<RunResult> {
  return isTauri() ? backend.submitCode(req) : mock.submitCode(req);
}

export async function analyzeComplexity(
  req: RunRequest
): Promise<ComplexityReport> {
  return isTauri()
    ? backend.analyzeComplexity(req)
    : mock.analyzeComplexity(req);
}

export async function detectRuntimes(): Promise<RuntimeInfo[]> {
  return isTauri() ? backend.detectRuntimes() : mock.detectRuntimes();
}

export async function getProgress(): Promise<Progress> {
  return isTauri() ? backend.getProgress() : mock.getProgress();
}

export async function getDashboard(): Promise<DashboardData> {
  return isTauri() ? backend.getDashboard() : mock.getDashboard();
}

export async function setProblemStatus(
  id: string,
  action: StatusAction
): Promise<void> {
  return isTauri()
    ? backend.setProblemStatus(id, action)
    : mock.setProblemStatus(id, action);
}

export async function toggleBookmark(id: string): Promise<boolean> {
  return isTauri() ? backend.toggleBookmark(id) : mock.toggleBookmark(id);
}

export async function getProblemUserState(
  id: string
): Promise<ProblemUserState> {
  return isTauri()
    ? backend.getProblemUserState(id)
    : mock.getProblemUserState(id);
}

export async function validateUserProblem(
  draft: UserProblemDraft
): Promise<ValidationResult> {
  return isTauri()
    ? backend.validateUserProblem(draft)
    : mock.validateUserProblem(draft);
}

export async function saveUserProblem(
  draft: UserProblemDraft
): Promise<Problem> {
  return isTauri() ? backend.saveUserProblem(draft) : mock.saveUserProblem(draft);
}

export async function saveDraft(
  draft: UserProblemDraft,
  draftId?: string
): Promise<string> {
  return isTauri()
    ? backend.saveDraft(draft, draftId)
    : mock.saveDraft(draft, draftId);
}

export async function listDrafts(): Promise<DraftSummary[]> {
  return isTauri() ? backend.listDrafts() : mock.listDrafts();
}

export async function getDraft(id: string): Promise<UserProblemDraft | null> {
  return isTauri() ? backend.getDraft(id) : mock.getDraft(id);
}

export async function deleteDraft(id: string): Promise<void> {
  return isTauri() ? backend.deleteDraft(id) : mock.deleteDraft(id);
}

export async function exportProblem(id: string): Promise<boolean> {
  return isTauri() ? backend.exportProblem(id) : mock.exportProblem(id);
}

export async function exportPack(): Promise<boolean> {
  return isTauri() ? backend.exportPack() : mock.exportPack();
}

export async function importProblems(): Promise<Problem[] | null> {
  return isTauri() ? backend.importProblems() : mock.importProblems();
}
