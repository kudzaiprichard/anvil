import type {
  ActivityDay,
  DashboardData,
  PatternStat,
  Progress,
  ProblemSummary,
} from "@/src/lib/types";
import { PATTERNS } from "@/src/lib/types";
import { MOCK_PROBLEMS, MOCK_STATUSES } from "./problems";

export const MOCK_PROGRESS: Progress = {
  solved: 42,
  total: 150,
  attempted: 61,
  streakDays: 7,
  bestStreakDays: 12,
  mastered: 18,
  needsReview: 12,
};

/** Pattern totals shown on the dashboard and the library rail. */
export const MOCK_PATTERN_STATS: PatternStat[] = [
  { pattern: "Arrays & Hashing", solved: 8, total: 8 },
  { pattern: "Two Pointers", solved: 6, total: 7 },
  { pattern: "Sliding Window", solved: 5, total: 6 },
  { pattern: "Stack", solved: 4, total: 7 },
  { pattern: "Binary Search", solved: 3, total: 7 },
  { pattern: "Linked List", solved: 2, total: 6 },
  { pattern: "Trees", solved: 5, total: 12 },
  { pattern: "Heap / Priority Queue", solved: 1, total: 5 },
  { pattern: "Backtracking", solved: 0, total: 6 },
  { pattern: "Graphs", solved: 1, total: 8 },
  { pattern: "1-D DP", solved: 2, total: 10 },
  { pattern: "2-D DP", solved: 0, total: 7 },
  { pattern: "Greedy", solved: 1, total: 5 },
  { pattern: "Intervals", solved: 2, total: 4 },
  { pattern: "Bit Manipulation", solved: 3, total: 5 },
];

/** Deterministic ~26 weeks of daily activity ending today. */
function buildActivity(): ActivityDay[] {
  const days: ActivityDay[] = [];
  const today = new Date();
  const totalDays = 26 * 7;
  let seed = 9;
  const rnd = () => {
    seed = (seed * 1103515245 + 12345) & 0x7fffffff;
    return seed / 0x7fffffff;
  };
  for (let i = totalDays - 1; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(today.getDate() - i);
    const r = rnd();
    let count = r < 0.42 ? 0 : r < 0.62 ? 1 : r < 0.8 ? 2 : r < 0.92 ? 3 : 5;
    // taper the most recent weeks like the mockup
    if (i < 14) count = Math.min(count, r < 0.6 ? 1 : 2);
    days.push({ date: d.toISOString().slice(0, 10), count });
  }
  return days;
}

export function buildDashboard(): DashboardData {
  const summaries = problemSummaries();
  const inProgress = summaries.find((p) => p.status === "in-progress");
  const sorted = [...MOCK_PATTERN_STATS].sort(
    (a, b) => a.solved / a.total - b.solved / b.total
  );
  const months = new Date();
  const start = new Date(months);
  start.setDate(start.getDate() - 26 * 7);
  const mid = new Date(months);
  mid.setDate(mid.getDate() - 13 * 7);
  const monthName = (d: Date) => d.toLocaleString("en", { month: "short" });

  return {
    progress: MOCK_PROGRESS,
    activity: buildActivity(),
    cumulative: [0, 1, 3, 6, 9, 12, 16, 20, 25, 30, 35, 42],
    axisLabels: [monthName(start), monthName(mid)],
    focus: sorted.slice(0, 3),
    strong: sorted.slice(-3).reverse(),
    continueProblem: inProgress,
    patternStats: PATTERNS.map(
      (pattern) =>
        MOCK_PATTERN_STATS.find((s) => s.pattern === pattern) ?? {
          pattern,
          solved: 0,
          total: 0,
        }
    ),
  };
}

export function problemSummaries(): ProblemSummary[] {
  return MOCK_PROBLEMS.map((p) => {
    const s = MOCK_STATUSES[p.id] ?? { status: "todo" as const };
    return {
      id: p.id,
      number: p.number,
      title: p.title,
      pattern: p.pattern,
      difficulty: p.difficulty,
      source: p.source,
      status: s.status,
      lastAttempted: s.lastAttempted,
    };
  });
}
