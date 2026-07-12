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
  /**
   * Ops-sequence problems (LRU Cache): LeetCode's wire format. `design_io`
   * node-types the constructor/method call boundary (e.g. a real TreeNode
   * into a BSTIterator constructor); absent ⇒ raw JSON args.
   */
  | { type: "design"; design_io?: DesignIo };

/** Node I/O for one design method; absent fields mean plain JSON. */
export interface MethodIo {
  params?: IoType[];
  returns?: IoType;
}

/**
 * Per-op I/O map for design packs: constructor param types plus a
 * method-name → MethodIo table. Undeclared methods run all-JSON.
 */
export interface DesignIo {
  ctor?: IoType[];
  methods?: Record<string, MethodIo>;
}

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

/**
 * A per-call I/O shape (task 0003 + closing-the-48 Phase B). Leaves are wire
 * names; composites reference an already-built earlier param (`node_ref`,
 * `clone_of`, `tail_of`, `node_index_of`), wrap a type built for judging but
 * never passed (`ctx_only`), or nest (`list_of`). Mirrors Rust `IoType`.
 */
export type IoType =
  | "json"
  | "linked_list"
  | "tree"
  | "cyclic_list"
  | "random_list"
  | "graph"
  | "n_ary_tree"
  | "quad_tree"
  | "next_tree"
  | "multilevel_list"
  | { list_of: IoType }
  | { ctx_only: IoType }
  | { node_ref: { param: number } }
  | { clone_of: { param: number } }
  | { tail_of: { param: number } }
  | { node_index_of: { param: number } };

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

/** One problem in the Stage-7 mixed capstone. `unit` is the pattern it actually
 *  belongs to — carried in content only, never sent to the workspace. */
export interface CapstoneProblem {
  slug: string;
  unit: string;
}

/** The Stage-7 Mixed Capstone (§4): an unlabeled cross-unit pool. */
export interface Capstone {
  id: string;
  stage: string;
  title: string;
  pass_count: number;
  timer_target_min: number;
  problems: CapstoneProblem[];
}

/** The one implicit course (§3.1): stages, the prereq DAG, gate defaults. */
export interface Curriculum {
  id: string;
  stages: CurriculumStage[];
  /** `unitId -> [prereq unitId]`, the DAG driving unlocking. */
  prereqs: Record<string, string[]>;
  gate_defaults: GateConfig;
  /** The optional Stage-7 mixed capstone (present once Stage 7 ships). */
  capstone?: Capstone;
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

/** One learner answer to a quiz item (`submitQuiz` payload). */
export interface QuizAnswer {
  itemId: string;
  /** The option text the learner selected. */
  selected: string;
}

/** The graded outcome of one quiz item (`submitQuiz` result). Formative only —
 *  `answer`/`explanation` are echoed so the runner reveals the trigger after
 *  the learner commits; pattern-picker items also carry `correctPattern`. */
export interface QuizItemResult {
  itemId: string;
  type: QuizItemType;
  correct: boolean;
  selected: string;
  /** The correct option text. */
  answer: string;
  explanation_md: string;
  correctPattern?: string;
}

/** A full graded submission (`submitQuiz`). Never blocks progression. */
export interface QuizGrade {
  correctCount: number;
  total: number;
  results: QuizItemResult[];
}

/** Reserved `submitQuiz` source id for the interleaved cross-unit pool. */
export const PATTERN_POOL_SOURCE = "pattern-pool";

export type DiagramMode = "view" | "perform";

/** One graded choice at a prediction pause. */
export interface DiagramChoice {
  id: string;
  label_md: string;
}

/** The graded "what happens next?" turn on a prediction-pause step (§13.4).
 *  Optional — a pause step without it degrades to think-then-reveal. In
 *  `perform` mode it is the learner's step, graded against ground truth. */
export interface DiagramPredict {
  prompt_md: string;
  choices: DiagramChoice[];
  /** The `id` of the correct choice — engine ground truth. */
  answer: string;
  explanation_md: string;
}

export interface DiagramStep {
  /** Opaque algorithm-state snapshot for this frame. */
  state: unknown;
  caption_md: string;
  /** Present on prediction-pause frames that carry a graded question. */
  predict?: DiagramPredict;
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

/* ---------- progression & mastery gate (Phase 3) ----------
 * Per-user *state* shapes (camelCase), distinct from the snake_case content
 * schemas above. Mirror src-tauri/src/domain/mastery.rs field-for-field. */

/** A unit's lock state: `locked` until every prereq is mastered, `mastered`
 *  once its gate is passed. */
export type UnitStatus = "locked" | "unlocked" | "mastered";

/** How close the learner is to clearing a unit's mastery gate — only hint-free,
 *  no-peek solves are tallied (COURSE_BLUEPRINT.md §6). */
export interface UnitGateState {
  /** Gate problems that must be solved hint-free. */
  passCount: number;
  /** Whether ≥1 of those solves must be a `novel` problem. */
  requireNovel: boolean;
  /** Soft per-problem target in minutes — shown, never enforced. */
  timerTargetMin: number;
  /** Distinct gate problems solved hint-free so far. */
  passedCount: number;
  /** Of those, how many were tagged novel. */
  passedNovel: number;
  /** The gate problem slugs cleared hint-free. */
  solvedSlugs: string[];
  /** Total gate problems in this unit's pool. */
  total: number;
  /** `true` once passedCount ≥ passCount and the novel requirement is met. */
  met: boolean;
}

/** A unit's full progression snapshot for the course/unit views. */
export interface UnitProgress {
  unitId: string;
  status: UnitStatus;
  lessonsTotal: number;
  lessonsComplete: number;
  gate: UnitGateState;
  /** Prereq unit ids not yet mastered — empty when unlocked/mastered. */
  blockedBy: string[];
}

/** The result of one gate attempt (`evaluateGate`). */
export interface GateOutcome {
  /** `false` when the learner used a hint/solution — a peeked attempt never
   *  counts toward mastery. */
  counted: boolean;
  /** This solve tipped the unit over its gate threshold. */
  unitMastered: boolean;
  /** The unit was already mastered before this attempt. */
  alreadyMastered: boolean;
  /** The updated gate tally after this attempt. */
  gate: UnitGateState;
  /** Unit ids that transitioned locked→unlocked because of this pass. */
  unlocked: string[];
}

// --- Phase 7: advanced progression -----------------------------------------

/** One capstone problem as the workspace sees it — no pattern label (§4). */
export interface CapstoneProblemView {
  problemId: string;
  solved: boolean;
}

/** The Stage-7 mixed capstone as the course page shows it (unlabeled pool). */
export interface CapstoneView {
  id: string;
  title: string;
  passCount: number;
  timerTargetMin: number;
  passedCount: number;
  total: number;
  met: boolean;
  /** `true` once every unit is mastered — the capstone only counts toward
   *  readiness then, though it can be attempted early for practice. */
  unlocked: boolean;
  problems: CapstoneProblemView[];
}

/** The result of one capstone attempt (`evaluateCapstone`). */
export interface CapstoneOutcome {
  counted: boolean;
  passedCount: number;
  total: number;
  met: boolean;
}

/** The diagnostic placement probe: unlabeled pattern-picker items that place the
 *  learner out of units they already recognize. */
export interface PlacementProbe {
  items: QuizItem[];
  unitIds: string[];
}

/** The result of submitting the placement probe. */
export interface PlacementOutcome {
  /** Units the learner was placed out of (recognized, prereqs cleared). */
  placed: string[];
  /** Units now unlocked — the learner's new frontier. */
  frontier: string[];
}

/** The honest course-readiness aggregate (§7). */
export interface Readiness {
  unitsTotal: number;
  unitsMastered: number;
  capstoneTotal: number;
  capstoneSolved: number;
  capstoneMet: boolean;
  /** 0–100 overall completion (ladder mastery weighted with capstone clears). */
  percent: number;
  /** `true` only when every unit is mastered and the capstone is met. */
  ready: boolean;
}

/** Where a lesson sits for this user (§6.4). `not-started` is the absence of
 *  a stored row; the backend only ever records the other two. */
export type LessonStatus = "not-started" | "in-progress" | "complete";

/** One lesson's stored progress (`recordLessonProgress` / `getLessonProgress`). */
export interface LessonProgress {
  lessonId: string;
  unitId: string;
  status: LessonStatus;
  /** Local ISO timestamps; absent until the transition that sets them. */
  startedAt?: string;
  completedAt?: string;
}

/* ---------- retention & habit (Phase 6) ---------- */

/** The learner's self-assessed recall after a cold re-solve — the four FSRS
 *  grades. `again` is the failure grade that demotes the card. */
export type ReviewRating = "again" | "hard" | "good" | "easy";

/** Where a card sits in the FSRS state machine (mirrors `review_schedule.state`). */
export type ReviewCardState = "new" | "learning" | "review" | "relearning";

/** One problem due to be re-solved cold now, in the interleaved queue. */
export interface ReviewItem {
  /** LeetCode slug — the workspace opens this for a cold re-solve. */
  problemId: string;
  /** The unit whose pattern this belongs to — drives interleaving + labels. */
  unitId: string;
  state: ReviewCardState;
  /** When the card became due (RFC3339 UTC). */
  dueAt: string;
  /** Last re-solve, or absent for a card that entered the queue and hasn't
   *  been reviewed yet. */
  lastReviewedAt?: string;
  /** Times this problem has been failed (`again`) — the demotion counter. */
  lapses: number;
  /** Whole days overdue (0 when it just came due). */
  overdueDays: number;
}

/** The honest habit layer (COURSE_BLUEPRINT.md §7): a streak that survives one
 *  missed day via a freeze ("never miss twice"). No XP, no leaderboards. */
export interface HabitState {
  /** Consecutive practice days, forgiving one isolated missed day. */
  currentStreak: number;
  bestStreak: number;
  /** A freeze is currently holding the streak together (yesterday was missed);
   *  miss again and it breaks. */
  freezeActive: boolean;
  /** Cards due to re-solve right now. */
  dueToday: number;
  /** Cards already re-solved today. */
  reviewedToday: number;
}

/** The review page payload: what's due (interleaved), how many are scheduled
 *  for later, and the habit header. */
export interface ReviewQueue {
  due: ReviewItem[];
  laterCount: number;
  habit: HabitState;
}

/** The result of recording one re-solve (`recordReview`). */
export interface ReviewOutcome {
  problemId: string;
  state: ReviewCardState;
  dueAt: string;
  /** Days until next due — the spacing interval FSRS just chose. */
  intervalDays: number;
  lapses: number;
  /** `true` when the re-solve was failed (`again`): interval collapsed, lapse
   *  counter bumped. */
  demoted: boolean;
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

/* ---------- deterministic complexity feedback (Phase 5) ---------- */

/** One measurement: `ops` Python-level operations executed at input size `n`. */
export interface ComplexitySample {
  n: number;
  ops: number;
}

/** How the measured growth compares to the pack's declared optimal. */
export type ComplexityVerdict = "optimal" | "slower" | "faster" | "unknown";

/** Result of profiling the learner's solution on growing inputs (op-count via
 *  the runner, no AI). `available: false` ⇒ `note` says why it couldn't run. */
export interface ComplexityReport {
  available: boolean;
  /** Measured class, e.g. "O(n^2)". */
  measured?: string;
  /** Pack-declared optimal, e.g. "O(n)". */
  optimal?: string;
  verdict: ComplexityVerdict;
  /** One-line learner-facing message. */
  note: string;
  samples: ComplexitySample[];
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
