/**
 * Typed data contract shared by the UI and the Rust backend (mirrors
 * src-tauri/src/domain/ field-for-field). Built-in, user-authored, and
 * imported problems use the exact same shape and the same runner.
 */

export type Difficulty = "Easy" | "Medium" | "Hard";

export type Language = "python" | "javascript";

export const LANGUAGES: readonly Language[] = ["python", "javascript"];

export const LANGUAGE_LABELS: Record<Language, string> = {
  python: "Python",
  javascript: "JavaScript",
};

export type ProblemSource = "built-in" | "user" | "imported";

/** The 15 patterns problems are organized by (UI_SPEC §6.1). */
export const PATTERNS = [
  "Arrays & Hashing",
  "Two Pointers",
  "Sliding Window",
  "Stack",
  "Binary Search",
  "Linked List",
  "Trees",
  "Heap / Priority Queue",
  "Backtracking",
  "Graphs",
  "1-D DP",
  "2-D DP",
  "Greedy",
  "Intervals",
  "Bit Manipulation",
] as const;

export type Pattern = (typeof PATTERNS)[number];

export interface Example {
  input: string;
  output: string;
  explanation_md?: string;
}

export interface TestCase {
  /** Positional arguments passed to the solve function, JSON-encodable. */
  input: unknown[];
  expected: unknown;
  hidden: boolean;
}

export interface FunctionSignature {
  python: string;
  javascript: string;
  /** Starter stubs for languages beyond the runnable Python/JS pair, carried
   *  from an imported catalog (`cpp`, `java`, …). Display-only today — the
   *  runner still executes only Python/JS — but preserved across re-imports.
   *  Omitted when the catalog ships no extra languages. */
  extra?: Record<string, string>;
}

/**
 * Output comparison mode for a problem's test cases.
 * - `exact` (default): strict deep equality, arrays order-sensitive.
 * - `unordered`: top-level array compared as a set — for "answer in any
 *   order" problems (subsets, permutations, grouped anagrams).
 */
export type Checker = "exact" | "unordered";

/**
 * Full judge taxonomy (CONTENT_DESIGN.md §4). `Checker` covers the original
 * two modes; `Judge` adds the modes imported/pack-backed problems need.
 * Absent on a problem ⇒ derived from `checker`.
 */
export type Judge =
  | { type: "exact" }
  | { type: "unordered" }
  /** Recursive numeric compare with tolerance (e.g. median problems). */
  | { type: "float"; epsilon: number }
  /** Void-return problems judged on a mutated argument. */
  | { type: "in_place"; arg_index: number }
  /** Pack-shipped validator decides (input, output) → bool. */
  | {
      type: "any_valid";
      validator_python: string;
      validator_javascript: string;
    }
  /** Ops-sequence problems (LRU Cache): LeetCode's wire format. */
  | { type: "design" };

/**
 * Which callable the harness invokes per language: "Solution.twoSum" means
 * instantiate the class and call the method; a bare name is a top-level
 * function. Absent ⇒ the legacy `solve` convention. Doubles as the
 * import-time match fingerprint (CONTENT_DESIGN.md §6).
 */
export interface EntryPoint {
  python: string;
  javascript: string;
  arity: number;
  /**
   * Node I/O types (task 0003): when present, the harness (de)serializes
   * ListNode/TreeNode params + return at the call boundary so a LeetCode stub
   * runs unmodified. Absent ⇒ all-JSON (the default for every existing pack).
   */
  io_types?: IoTypes;
}

/** A per-call I/O shape: plain JSON, a linked list, a binary tree, or a list of. */
export type IoType = "json" | "linked_list" | "tree" | { list_of: IoType };

export interface IoTypes {
  params: IoType[];
  returns: IoType;
}

export interface ReferenceSolution {
  python?: string;
  javascript?: string;
  complexity?: { time: string; space: string };
}

export interface Problem {
  id: string;
  /** Display number used in lists ("23. …"). App-level, not part of §8.3. */
  number: number;
  title: string;
  pattern: Pattern;
  difficulty: Difficulty;
  source: ProblemSource;
  description_md: string;
  /**
   * Sanitized HTML statement, present only on imported problems (full
   * fidelity: superscripts, <pre> examples). When set, the problem panel
   * renders this instead of `description_md`. CONTENT_DESIGN.md §8.
   */
  body_html?: string;
  constraints: string[];
  examples: Example[];
  function_signature: FunctionSignature;
  test_cases: TestCase[];
  /** Output comparison mode; defaults to "exact" when omitted. */
  checker?: Checker;
  /** Full judge mode; absent ⇒ derived from `checker`. */
  judge?: Judge;
  /** Harness entry point; absent ⇒ legacy top-level `solve`. */
  entry_point?: EntryPoint;
  hints: string[];
  reference_solution?: ReferenceSolution;
  explanation_md?: string;
  follow_up?: string;
  license: string;
  author: string;
}

/* ---------- test packs (CONTENT_DESIGN.md §3) ---------- */

/** What a literal pack test is probing; surfaced in "reveal failing case". */
export type PackTestKind = "edge" | "boundary" | "trap";

/**
 * A literal verified test: positional args (same convention as
 * `TestCase.input`) with an expected value computed by executing the
 * reference solutions — never authored by the model.
 */
export interface PackTest {
  kind: PackTestKind;
  description: string;
  input: unknown[];
  expected: unknown;
}

/**
 * Deterministic stress-input generator spec; materialized into ordinary
 * hidden test cases at import time — large inputs never ship as literals.
 */
export interface StressSpec {
  description: string;
  seed: number;
  size: number;
  /** Generator source: `def gen(rng, size)` returning the args tuple. */
  generator_python: string;
  note?: string;
}

/** Machine-usable summary of one parameter's constraints. */
export interface ConstraintSpec {
  param: string;
  /** Coarse shape tag, e.g. "int", "int[]", "string". */
  kind: string;
  /** [min, max] length bounds for sequence params. */
  len?: [number, number];
  /** [min, max] value bounds for numeric params/elements. */
  value?: [number, number];
}

export interface PackSolutions {
  python: string;
  javascript: string;
  /** Naive oracle used for differential verification. */
  brute_force_python?: string;
  complexity?: { time: string; space: string };
}

/** One entry of the shipped test-pack bundle, keyed by LeetCode slug. */
export interface TestPack {
  slug: string;
  qid: string;
  schema_version: number;
  entry_point: EntryPoint;
  judge: Judge;
  /** One-sentence pattern explanation (what this problem teaches). */
  pattern: string;
  /** Three progressive hints: nudge → approach → near-answer. */
  hints: string[];
  constraints: ConstraintSpec[];
  tests: PackTest[];
  stress: StressSpec[];
  solutions: PackSolutions;
  verified: boolean;
  generated_at: string;
}

/* ---------- curriculum / course content (LESSON_COURSE_DESIGN.md §3, §7) ----------
 * Phase 1 ships schemas + a fail-closed loader only; no UI reads these yet.
 * Field names are snake_case, same convention as Problem/TestPack/Preset. */

export interface GateConfig {
  pass_count: number;
  require_novel: boolean;
  timer_target_min: number;
  threshold_pct: number;
}

export type ProblemRole = "worked" | "guided" | "gate";
export type ProblemTier = "intro" | "core" | "stretch";

export interface UnitProblem {
  slug: string;
  role: ProblemRole;
  tier: ProblemTier;
  novel: boolean;
}

/** A unit manifest (§3.2): one concept, the mastery-gate boundary. */
export interface Unit {
  id: string;
  stage: string;
  title: string;
  prereqs: string[];
  /** Lesson ids in order; empty until Phase 2 authors lesson content. */
  lessons: string[];
  problems: UnitProblem[];
  gate: GateConfig;
  /** Unit ids whose pattern this unit's practice must resurface. */
  spiral: string[];
}

export interface CurriculumStage {
  id: string;
  title: string;
  units: string[];
}

/** The one implicit course (§3.1): stages, the prereq DAG, gate defaults. */
export interface Curriculum {
  id: string;
  stages: CurriculumStage[];
  /** `unitId -> [prereq unitId]`, the DAG driving unlocking. */
  prereqs: Record<string, string[]>;
  gate_defaults: GateConfig;
}

export type QuizItemType = "concept-check" | "pattern-picker" | "complexity";

export interface QuizItem {
  id: string;
  type: QuizItemType;
  prompt_md: string;
  options: string[];
  answer: string;
  /** The unit/pattern id a `pattern-picker` item is testing recognition of. */
  correct_pattern?: string;
  explanation_md: string;
}

/** One lesson's quiz file (§3.4, §7.4). */
export interface Quiz {
  items: QuizItem[];
}

export type DiagramMode = "view" | "perform";

export interface DiagramStep {
  /** Opaque algorithm-state snapshot for this frame. */
  state: unknown;
  caption_md: string;
}

/** A prediction-diagram spec (§3.5, §7.5): the renderer/animator is engine
 *  (Phase 5); the steps/trace are precomputed data. */
export interface DiagramSpec {
  id: string;
  algorithm: string;
  /** The worked-example problem slug this diagram is keyed to. */
  for_problem: string;
  mode: DiagramMode;
  steps: DiagramStep[];
  /** Step indices where playback pauses to ask "what happens next?" */
  predict_at: number[];
}

/** A fully loaded lesson (§3.3): one sub-pattern, the atom of the course. */
export interface Lesson {
  id: string;
  unit: string;
  subpattern: string;
  explainer_md: string;
  trigger_signals: string[];
  /** The worked-example problem slug. */
  worked_example: string;
  diagram: DiagramSpec;
  quiz: Quiz;
  /** Ordered practice slugs, faded -> independent. */
  practice: string[];
  /** Earlier lesson ids this lesson's recap retrieval pulls from. */
  recap: string[];
  follow_up: string[];
}

/* ---------- progress & status (app types) ---------- */

export type ProblemStatus = "todo" | "in-progress" | "solved" | "needs-review";

export interface ProblemSummary {
  id: string;
  number: number;
  title: string;
  pattern: Pattern;
  difficulty: Difficulty;
  /** Library badge ("imported" / "user"); built-ins render no badge. */
  source: ProblemSource;
  status: ProblemStatus;
  /** Human label like "2d ago"; undefined when never attempted. */
  lastAttempted?: string;
}

export interface PatternStat {
  pattern: Pattern;
  solved: number;
  total: number;
}

export interface Progress {
  solved: number;
  total: number;
  attempted: number;
  streakDays: number;
  bestStreakDays: number;
  mastered: number;
  needsReview: number;
}

export interface ActivityDay {
  /** ISO date (yyyy-mm-dd). */
  date: string;
  count: number;
}

export interface DashboardData {
  progress: Progress;
  /** Daily solve counts for the heatmap, oldest first (~26 weeks). */
  activity: ActivityDay[];
  /** Cumulative solved counts for the progress line, oldest first. */
  cumulative: number[];
  /** Month labels for the line chart axis: [start, mid]. */
  axisLabels: [string, string];
  focus: PatternStat[];
  strong: PatternStat[];
  continueProblem?: ProblemSummary;
  patternStats: PatternStat[];
}

/* ---------- running code ---------- */

export interface RunRequest {
  id: string;
  language: Language;
  code: string;
}

export type RunStatus = "pass" | "fail" | "error" | "timeout";

export interface CaseResult {
  /** 1-based case number. */
  index: number;
  hidden: boolean;
  passed: boolean;
  /** Display strings; omitted for hidden cases. */
  input?: string;
  output?: string;
  expected?: string;
  error?: string;
}

export interface RunResult {
  status: RunStatus;
  cases: CaseResult[];
  passed: number;
  total: number;
  runtimeMs?: number;
  memoryMb?: number;
  /** stderr / traceback when status is "error" or "timeout". */
  error?: string;
}

/* ---------- status mutations (workspace buttons) ---------- */

export type StatusAction =
  | "mark_mastered"
  | "unmark_mastered"
  | "needs_review"
  | "clear_review";

export interface ProblemUserState {
  status: ProblemStatus;
  bookmarked: boolean;
  mastered: boolean;
  /** Code snapshot from the most recent run/submit, for editor restore. */
  lastCode?: string;
  lastLanguage?: Language;
}

/* ---------- runtimes (Settings → Runtime pane) ---------- */

export interface RuntimeInfo {
  /** Short mono badge, e.g. "Py" / "JS". */
  tag: string;
  name: string;
  /** Absolute interpreter path; empty when not found. */
  path: string;
  /** Display version like "v3.12.1"; empty when not found. */
  version: string;
  found: boolean;
}

/* ---------- list filtering ---------- */

export type ProblemSort = "number" | "difficulty" | "recent";

export interface ProblemFilter {
  search?: string;
  pattern?: Pattern;
  difficulty?: Difficulty;
  status?: ProblemStatus;
  sort?: ProblemSort;
}

/* ---------- authoring (create / edit) ---------- */

export interface DraftTestCase {
  /** Raw JSON text as typed in the form. */
  input: string;
  expected: string;
  hidden: boolean;
}

export interface UserProblemDraft {
  /** Present when editing an existing problem. */
  id?: string;
  title: string;
  pattern: Pattern | "";
  difficulty: Difficulty;
  description_md: string;
  constraints: string[];
  follow_up?: string;
  examples: Example[];
  function_signature: FunctionSignature;
  test_cases: DraftTestCase[];
  hints: string[];
  reference_solution: ReferenceSolution;
  /** Legal requirement (PROJECT_SPEC §8.5) — must be true to save. */
  originalityWarranty: boolean;
}

export interface ValidationIssue {
  /** Form section the issue belongs to, e.g. "Test case 2". */
  field: string;
  message: string;
}

export interface ValidationResult {
  ok: boolean;
  issues: ValidationIssue[];
  /** Reference-solution run against the draft's test cases, when executed. */
  caseResults?: CaseResult[];
}

export interface DraftSummary {
  id: string;
  title: string;
  /** Local ISO timestamp of the last save. */
  updatedAt: string;
}
