"use client";

import {
  Suspense,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import {
  ChevronDown,
  Code2,
  Maximize,
  Minimize,
  RotateCcw,
  ShieldCheck,
  Trophy,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { AppShell } from "@/src/components/anvil/app-shell";
import { CodeEditor } from "@/src/components/anvil/code-editor";
import { Spinner } from "@/src/components/anvil/spinner";
import { ProblemSheet } from "@/src/components/problems/problem-sheet";
import { ProblemPane } from "@/src/components/workspace/problem-pane";
import {
  ResultsPanel,
  type ResultsTab,
  type RunState,
} from "@/src/components/workspace/results-panel";
import { useWorkspaceShortcuts } from "@/src/components/workspace/use-workspace-shortcuts";
import { TopBar } from "@/src/components/workspace/top-bar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/src/components/shadcn/dropdown-menu";
import {
  analyzeComplexity,
  evaluateCapstone,
  evaluateGate,
  getProblem,
  getProblemUserState,
  listProblems,
  setProblemStatus,
  submitCode,
  toggleBookmark,
} from "@/src/lib/api";
import { loadAutosave, saveAutosave } from "@/src/lib/code-autosave";
import {
  getEditorPrefs,
  setEditorPrefs,
  useEditorPrefs,
  type WorkspaceLayout,
} from "@/src/lib/settings";
import {
  PracticeTimer,
  type PracticeTimerHandle,
} from "@/src/components/workspace/practice-timer";
import type { Language, Problem, ProblemSummary } from "@/src/lib/types";
import { LANGUAGE_LABELS, LANGUAGES } from "@/src/lib/types";

function clamp(v: number, min: number, max: number) {
  return Math.min(max, Math.max(min, v));
}

function ColDivider({
  onPointerDown,
}: {
  onPointerDown: (e: React.PointerEvent) => void;
}) {
  return (
    <div
      role="separator"
      aria-orientation="vertical"
      onPointerDown={onPointerDown}
      className="w-px shrink-0 cursor-col-resize border-r transition-colors hover:border-primary/50"
    />
  );
}

function RowGrip({
  onPointerDown,
}: {
  onPointerDown: (e: React.PointerEvent) => void;
}) {
  return (
    <div
      role="separator"
      aria-orientation="horizontal"
      onPointerDown={onPointerDown}
      className="absolute inset-x-0 -top-[5px] z-10 flex h-[9px] cursor-row-resize items-center justify-center"
    >
      <span className="h-[3px] w-[34px] rounded-sm bg-border" />
    </div>
  );
}

/**
 * Arranges the three workspace panes into the layout chosen in Settings →
 * Appearance. The pane contents are passed in as slots so switching layouts
 * only moves them around the tree.
 */
function WorkspaceBody({
  bodyRef,
  layout,
  maximized,
  leftPct,
  resultsH,
  resultsW,
  dragProblemLeft,
  dragProblemRight,
  dragRow,
  dragResultsCol,
  problem,
  editor,
  results,
}: {
  bodyRef: React.RefObject<HTMLDivElement | null>;
  layout: WorkspaceLayout;
  maximized: boolean;
  leftPct: number;
  resultsH: number;
  resultsW: number;
  dragProblemLeft: (e: React.PointerEvent) => void;
  dragProblemRight: (e: React.PointerEvent) => void;
  dragRow: (e: React.PointerEvent) => void;
  dragResultsCol: (e: React.PointerEvent) => void;
  problem: React.ReactNode;
  editor: React.ReactNode;
  results: React.ReactNode;
}) {
  const problemBox = (
    <div className="min-w-0 shrink-0" style={{ width: `${leftPct}%` }}>
      {problem}
    </div>
  );
  const resultsDock = (
    <div
      className="relative flex shrink-0 flex-col border-t"
      style={{ height: resultsH }}
    >
      <RowGrip onPointerDown={dragRow} />
      {results}
    </div>
  );

  return (
    <div ref={bodyRef} className="flex min-h-0 flex-1">
      {maximized ? (
        editor
      ) : layout === "mirrored" ? (
        <>
          <div className="flex min-w-0 flex-1 flex-col">
            {editor}
            {resultsDock}
          </div>
          <ColDivider onPointerDown={dragProblemRight} />
          {problemBox}
        </>
      ) : layout === "bottom" ? (
        <div className="flex min-w-0 flex-1 flex-col">
          <div className="flex min-h-0 flex-1">
            {problemBox}
            <ColDivider onPointerDown={dragProblemLeft} />
            {editor}
          </div>
          {resultsDock}
        </div>
      ) : layout === "columns" ? (
        <>
          {problemBox}
          <ColDivider onPointerDown={dragProblemLeft} />
          {editor}
          <ColDivider onPointerDown={dragResultsCol} />
          <div
            className="flex min-w-0 shrink-0 flex-col"
            style={{ width: resultsW }}
          >
            {results}
          </div>
        </>
      ) : layout === "editor-deck" ? (
        <div className="flex min-w-0 flex-1 flex-col">
          {editor}
          <div
            className="relative flex shrink-0 border-t"
            style={{ height: resultsH }}
          >
            <RowGrip onPointerDown={dragRow} />
            {problemBox}
            <ColDivider onPointerDown={dragProblemLeft} />
            <div className="flex min-w-0 flex-1 flex-col">{results}</div>
          </div>
        </div>
      ) : (
        /* classic */
        <>
          {problemBox}
          <ColDivider onPointerDown={dragProblemLeft} />
          <div className="flex min-w-0 flex-1 flex-col">
            {editor}
            {resultsDock}
          </div>
        </>
      )}
    </div>
  );
}

/** "arrays-hashing" -> "Arrays Hashing" — a readable unit label for the gate
 *  banner without an extra fetch. */
function prettifyUnit(slug: string): string {
  return slug
    .split("-")
    .map((w) => (w.length <= 2 ? w : w[0].toUpperCase() + w.slice(1)))
    .join(" ");
}

function Workspace() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const id = searchParams.get("id");

  // Mastery-gate mode (COURSE_BLUEPRINT.md §6): launched from a unit's gate
  // with `?gate=<unitId>&target=<min>`. Hints/solution are guarded, a soft
  // timer shows the target, and a pass is scored via `evaluateGate`.
  const gateUnit = searchParams.get("gate");
  const gateTargetRaw = searchParams.get("target");
  const gateTarget = gateTargetRaw ? Number(gateTargetRaw) : undefined;
  const [gateHelpUsed, setGateHelpUsed] = useState(false);

  // Mixed-capstone mode (Phase 7, §4): launched from the capstone with
  // `?capstone=1&target=<min>`. Same closed-book rules as a gate — hints/solution
  // guarded, soft timer, complexity off — but the problem is *unlabeled* and a
  // pass is scored via `evaluateCapstone`.
  const capstoneMode = searchParams.get("capstone") === "1";
  const examMode = !!gateUnit || capstoneMode;

  // Where the workspace's back button returns to. A lesson launches practice /
  // worked-example problems with `?from=<lessonId>`; gate and capstone carry
  // their own origin. This keeps the desktop app navigable — you can always get
  // back to where you came from instead of being stranded on the problem.
  const fromLesson = searchParams.get("from");
  const backNav = fromLesson
    ? { href: `/learn?lesson=${fromLesson}`, label: "Back to lesson" }
    : gateUnit
      ? { href: `/learn?unit=${gateUnit}`, label: "Back to unit" }
      : capstoneMode
        ? { href: "/learn?capstone=1", label: "Back to capstone" }
        : { href: "/problems", label: "Problems" };

  // `problem` is derived: stale loads don't render while a new id is loading.
  const [loaded, setLoaded] = useState<{ id: string; problem: Problem } | null>(
    null
  );
  const problem = loaded && loaded.id === id ? loaded.problem : null;
  const [summaries, setSummaries] = useState<ProblemSummary[]>([]);
  const [language, setLanguage] = useState<Language>("python");
  const [codeByLang, setCodeByLang] = useState<Record<Language, string>>({
    python: "",
    javascript: "",
  });
  const [runState, setRunState] = useState<RunState>("idle");
  const [resultsTab, setResultsTab] = useState<ResultsTab>("testcase");
  const [selectedCase, setSelectedCase] = useState(0);
  const [sheetOpen, setSheetOpen] = useState(false);
  const [maximized, setMaximized] = useState(false);
  const [bookmarked, setBookmarked] = useState(false);

  // pane sizes — restored from prefs, persisted when a divider drag ends
  const bodyRef = useRef<HTMLDivElement>(null);
  const [leftPct, setLeftPct] = useState(() => getEditorPrefs().paneLeftPct);
  const [resultsH, setResultsH] = useState(() => getEditorPrefs().paneResultsH);
  const [resultsW, setResultsW] = useState(() => getEditorPrefs().paneResultsW);
  const sizesRef = useRef({ leftPct, resultsH, resultsW });
  useEffect(() => {
    sizesRef.current = { leftPct, resultsH, resultsW };
  });

  const prefs = useEditorPrefs();
  const timerRef = useRef<PracticeTimerHandle>(null);

  useEffect(() => {
    listProblems().then(setSummaries);
  }, []);

  // Redirect bare /problem to the first problem once the list is known.
  useEffect(() => {
    if (!id && summaries.length > 0) {
      router.replace(`/problem?id=${summaries[0].id}`);
    }
  }, [id, summaries, router]);

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    Promise.all([getProblem(id), getProblemUserState(id)]).then(([p, s]) => {
      if (cancelled) return;
      if (!p) {
        toast.error("Problem not found");
        router.replace("/problems");
        return;
      }
      setLoaded({ id, problem: p });
      // Restore priority per language: live autosave (survives navigating
      // away without running) > last-run snapshot > per-language starter.
      const next: Record<Language, string> = {
        python: loadAutosave(id, "python") ?? p.function_signature.python,
        javascript:
          loadAutosave(id, "javascript") ?? p.function_signature.javascript,
      };
      if (s.lastCode && s.lastLanguage) {
        if (loadAutosave(id, s.lastLanguage) === null) {
          next[s.lastLanguage] = s.lastCode;
        }
        setLanguage(s.lastLanguage);
      }
      // Single-language problems (e.g. the concurrency set is Python-only —
      // no JS threads) ship an empty stub for the missing language; never
      // leave the editor parked on one. Checked against whatever language is
      // actually active (the restored one or the carried-over selection).
      setLanguage((prev) => {
        const active = s.lastLanguage ?? prev;
        if (p.function_signature[active]?.trim()) return active;
        return LANGUAGES.find((l) => p.function_signature[l]?.trim()) ?? active;
      });
      setCodeByLang(next);
      setBookmarked(s.bookmarked);
      setRunState("idle");
      setResultsTab("testcase");
      setSelectedCase(0);
      setGateHelpUsed(false);
    });
    return () => {
      cancelled = true;
    };
  }, [id, router]);

  const handleToggleBookmark = useCallback(async () => {
    if (!id) return;
    try {
      setBookmarked(await toggleBookmark(id));
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Bookmark failed");
    }
  }, [id]);

  const handleMarkMastered = useCallback(async () => {
    if (!id) return;
    try {
      await setProblemStatus(id, "mark_mastered");
      toast.success("Marked as mastered.");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Could not mark as mastered"
      );
    }
  }, [id]);

  const code = codeByLang[language];

  // Debounced autosave — typing then navigating away never loses work.
  useEffect(() => {
    if (!problem) return;
    const handle = setTimeout(
      () => saveAutosave(problem.id, language, code),
      600
    );
    return () => clearTimeout(handle);
  }, [problem, language, code]);

  const setCode = useCallback(
    (next: string) =>
      setCodeByLang((prev) =>
        prev[language] === next ? prev : { ...prev, [language]: next }
      ),
    [language]
  );

  const index = useMemo(
    () => summaries.findIndex((s) => s.id === id),
    [summaries, id]
  );
  const goTo = useCallback(
    (target?: ProblemSummary) => {
      if (target) router.push(`/problem?id=${target.id}`);
    },
    [router]
  );

  // Scores a passing gate submit and reports the outcome. A help-used solve is
  // recorded but doesn't count; a counted pass that masters the unit routes
  // back to the unit view so the learner sees the next unit unlock.
  const scoreGate = useCallback(
    async (problemId: string, solveTime: string | null) => {
      if (!gateUnit) return;
      try {
        const outcome = await evaluateGate(gateUnit, problemId, gateHelpUsed);
        if (!outcome.counted) {
          toast.warning(
            "Solved — but this gate problem used help, so it doesn't count toward mastery."
          );
          return;
        }
        if (outcome.unitMastered) {
          const extra = outcome.unlocked.length
            ? ` ${outcome.unlocked.map(prettifyUnit).join(", ")} unlocked.`
            : "";
          toast.success(`Gate passed — unit mastered!${extra}`);
          router.push(`/learn?unit=${gateUnit}`);
        } else {
          const { passedCount, passCount } = outcome.gate;
          toast.success(
            `Gate solve counted${solveTime ? ` (${solveTime})` : ""} — ${passedCount}/${passCount} cleared.`
          );
        }
      } catch (err) {
        toast.error(
          err instanceof Error ? err.message : "Could not score the gate"
        );
      }
    },
    [gateUnit, gateHelpUsed, router]
  );

  // Scores a passing capstone submit (Phase 7). A help-used solve is recorded
  // but doesn't count; clearing the capstone routes back to it so the learner
  // sees the "interview-ready" verdict.
  const scoreCapstone = useCallback(
    async (problemId: string, solveTime: string | null) => {
      try {
        const outcome = await evaluateCapstone(problemId, gateHelpUsed);
        if (!outcome.counted) {
          toast.warning(
            "Solved — but this capstone problem used help, so it doesn't count."
          );
          return;
        }
        if (outcome.met) {
          toast.success("Capstone cleared — you can solve the unfamiliar. 🏆");
          router.push("/learn?capstone=1");
        } else {
          toast.success(
            `Capstone solve counted${solveTime ? ` (${solveTime})` : ""} — ${outcome.passedCount}/${outcome.total} solved.`
          );
        }
      } catch (err) {
        toast.error(
          err instanceof Error ? err.message : "Could not score the capstone"
        );
      }
    },
    [gateHelpUsed, router]
  );

  // The one Run action: the app is fully offline, so there is no separate
  // Submit — every run executes the full suite (visible + hidden) and
  // records the attempt toward progress.
  const execute = useCallback(async () => {
    if (!problem || runState === "running") return;
    setRunState("running");
    setResultsTab("result");
    try {
      const result = await submitCode({ id: problem.id, language, code });
      setRunState(result);
      if (result.status === "pass") {
        // Freeze the practice clock and record this problem's solve time.
        const solveTime = timerRef.current?.solvedNow() ?? null;
        if (gateUnit) {
          await scoreGate(problem.id, solveTime);
        } else if (capstoneMode) {
          await scoreCapstone(problem.id, solveTime);
        } else {
          toast.success(
            solveTime
              ? `Accepted — solved in ${solveTime}.`
              : "Accepted — all tests passed."
          );
        }
      }
      // statuses changed — refresh the rows that feed the problem sheet
      listProblems().then(setSummaries);
    } catch (err) {
      setRunState("idle");
      toast.error(err instanceof Error ? err.message : "Run failed");
    }
  }, [problem, runState, language, code, gateUnit, capstoneMode, scoreGate, scoreCapstone]);

  // Profiles the current editor code deterministically (Phase 5). Bound to the
  // live language/code so "Analyze" measures exactly what the learner ran.
  const analyzeCurrentComplexity = useCallback(() => {
    if (!problem) return Promise.reject(new Error("No problem loaded"));
    return analyzeComplexity({ id: problem.id, language, code });
  }, [problem, language, code]);

  useWorkspaceShortcuts({
    onRun: execute,
    onPrev: () => goTo(summaries[index - 1] ?? summaries[summaries.length - 1]),
    onNext: () => goTo(summaries[index + 1] ?? summaries[0]),
    onToggleList: () => setSheetOpen((open) => !open),
    onToggleMaximize: () => setMaximized((m) => !m),
    onReset: () => {
      if (!problem) return;
      setCode(problem.function_signature[language]);
      toast("Editor reset to starter code.");
    },
  });

  // divider drags — one rig: capture the body rect on pointerdown, then
  // route pointer moves into the relevant pane-size setter.
  const startDrag = useCallback(
    (move: (ev: PointerEvent, rect: DOMRect) => void) =>
      (e: React.PointerEvent) => {
        e.preventDefault();
        const body = bodyRef.current;
        if (!body) return;
        const rect = body.getBoundingClientRect();
        const onMove = (ev: PointerEvent) => move(ev, rect);
        const up = () => {
          window.removeEventListener("pointermove", onMove);
          window.removeEventListener("pointerup", up);
          // drag finished — remember the arrangement for next time
          const s = sizesRef.current;
          setEditorPrefs({
            paneLeftPct: s.leftPct,
            paneResultsH: s.resultsH,
            paneResultsW: s.resultsW,
          });
        };
        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", up);
      },
    []
  );

  const dragProblemLeft = startDrag((ev, rect) =>
    setLeftPct(clamp(((ev.clientX - rect.left) / rect.width) * 100, 22, 65))
  );
  const dragProblemRight = startDrag((ev, rect) =>
    setLeftPct(clamp(100 - ((ev.clientX - rect.left) / rect.width) * 100, 22, 65))
  );
  const dragRow = startDrag((ev, rect) =>
    setResultsH(clamp(rect.bottom - ev.clientY, 120, rect.height - 140))
  );
  const dragResultsCol = startDrag((ev, rect) =>
    setResultsW(clamp(rect.right - ev.clientX, 280, 600))
  );

  if (!problem) {
    return (
      <AppShell>
        <TopBar
          running={false}
          onOpenList={() => setSheetOpen(true)}
          onPrev={() => undefined}
          onNext={() => undefined}
          onShuffle={() => undefined}
          onRun={() => undefined}
          back={backNav}
        />
        <div className="flex flex-1 items-center justify-center">
          <Spinner className="size-6" />
        </div>
      </AppShell>
    );
  }

  return (
    <AppShell
      status={
        <span>
          #{problem.number} · {LANGUAGE_LABELS[language].toLowerCase()} · sandbox
          local{gateUnit ? " · mastery gate" : capstoneMode ? " · capstone" : ""}
        </span>
      }
    >
      <TopBar
        running={runState === "running"}
        back={backNav}
        onOpenList={() => setSheetOpen(true)}
        onPrev={() => goTo(summaries[index - 1] ?? summaries[summaries.length - 1])}
        onNext={() => goTo(summaries[index + 1] ?? summaries[0])}
        onShuffle={() => {
          const others = summaries.filter((s) => s.id !== id);
          goTo(others[Math.floor(Math.random() * others.length)]);
        }}
        onRun={execute}
        timer={
          prefs.showTimer || examMode ? (
            <PracticeTimer
              key={problem.id}
              ref={timerRef}
              problemId={problem.id}
              // Gate/capstone always run the timer (soft target); practice honors the pref.
              autoStart={examMode ? true : prefs.timerAutoStart}
              targetMinutes={examMode ? gateTarget : undefined}
            />
          ) : undefined
        }
      />

      {gateUnit && (
        <div className="flex shrink-0 items-center gap-2.5 border-b border-medium/30 bg-medium/10 px-4 py-2 text-[12.5px]">
          <ShieldCheck className="size-[15px] shrink-0 text-medium" />
          <span className="font-semibold text-medium">Mastery gate</span>
          <span className="text-muted-foreground">
            {prettifyUnit(gateUnit)} · solve it cold — no hints, revealing the solution voids it, timer is a soft target.
          </span>
          <span className="flex-1" />
          <Link
            href={`/learn?unit=${gateUnit}`}
            className="inline-flex items-center gap-1 rounded-md px-2 py-0.5 font-medium text-muted-foreground transition-colors hover:bg-medium/15 hover:text-foreground"
          >
            <X className="size-[13px]" />
            Exit gate
          </Link>
        </div>
      )}

      {capstoneMode && (
        <div className="flex shrink-0 items-center gap-2.5 border-b border-amber-500/30 bg-amber-500/10 px-4 py-2 text-[12.5px]">
          <Trophy className="size-[15px] shrink-0 text-amber-500" />
          <span className="font-semibold text-amber-600 dark:text-amber-400">
            Mixed capstone
          </span>
          <span className="text-muted-foreground">
            Unlabeled — no pattern given. Recognize it and solve it cold: no
            hints, revealing the solution voids it, timer is a soft target.
          </span>
          <span className="flex-1" />
          <Link
            href="/learn?capstone=1"
            className="inline-flex items-center gap-1 rounded-md px-2 py-0.5 font-medium text-muted-foreground transition-colors hover:bg-amber-500/15 hover:text-foreground"
          >
            <X className="size-[13px]" />
            Exit capstone
          </Link>
        </div>
      )}

      <WorkspaceBody
        bodyRef={bodyRef}
        layout={prefs.workspaceLayout}
        maximized={maximized}
        leftPct={leftPct}
        resultsH={resultsH}
        resultsW={resultsW}
        dragProblemLeft={dragProblemLeft}
        dragProblemRight={dragProblemRight}
        dragRow={dragRow}
        dragResultsCol={dragResultsCol}
        problem={
          <ProblemPane
            problem={problem}
            bookmarked={bookmarked}
            onToggleBookmark={handleToggleBookmark}
            gateMode={examMode}
            onGateHelpUsed={() => setGateHelpUsed(true)}
          />
        }
        editor={
          <div className="flex min-h-0 min-w-0 flex-1 flex-col">
            {/* editor header */}
            <div className="flex h-9 shrink-0 items-center gap-2 border-b bg-card pl-3 pr-2">
              <div className="flex items-center gap-[7px] text-[12.5px] font-semibold">
                <Code2 className="size-[14px] stroke-[2.2] text-primary" />
                Code
              </div>
              <div className="flex-1" />
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <button
                    type="button"
                    className="flex items-center gap-1.5 rounded-md border bg-card px-2.5 py-1 text-[12px] font-medium transition-colors hover:bg-accent"
                  >
                    {LANGUAGE_LABELS[language]}
                    <ChevronDown className="size-[13px] text-muted-foreground" />
                  </button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  {LANGUAGES.filter(
                    (l) => !problem || problem.function_signature[l]?.trim()
                  ).map((l) => (
                    <DropdownMenuItem key={l} onClick={() => setLanguage(l)}>
                      {LANGUAGE_LABELS[l]}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
              <button
                type="button"
                title="Reset to starter"
                onClick={() => {
                  setCode(problem.function_signature[language]);
                  toast("Editor reset to starter code.");
                }}
                className="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              >
                <RotateCcw className="size-[15px]" />
              </button>
              <button
                type="button"
                title={maximized ? "Restore layout" : "Maximize editor"}
                onClick={() => setMaximized((m) => !m)}
                className="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              >
                {maximized ? (
                  <Minimize className="size-[15px]" />
                ) : (
                  <Maximize className="size-[15px]" />
                )}
              </button>
            </div>

            {/* editor */}
            <div className="min-h-0 flex-1">
              <CodeEditor
                value={code}
                language={language}
                onChange={setCode}
                fontSize={prefs.fontSize}
                tabSize={prefs.tabSize}
                lineWrap={prefs.lineWrap}
                docKey={problem.id}
                suppressIndentChords
              />
            </div>
          </div>
        }
        results={
          <ResultsPanel
            problem={problem}
            runState={runState}
            tab={resultsTab}
            onTabChange={setResultsTab}
            selectedCase={selectedCase}
            onSelectCase={setSelectedCase}
            onMarkMastered={handleMarkMastered}
            complexityEnabled={!examMode}
            onAnalyzeComplexity={analyzeCurrentComplexity}
          />
        }
      />

      <ProblemSheet
        open={sheetOpen}
        onOpenChange={setSheetOpen}
        currentId={id ?? undefined}
        onSelect={(target) => {
          setSheetOpen(false);
          router.push(`/problem?id=${target}`);
        }}
      />
    </AppShell>
  );
}

export default function ProblemPage() {
  return (
    <Suspense
      fallback={
        <AppShell>
          <div className="flex flex-1 items-center justify-center">
            <Spinner className="size-6" />
          </div>
        </AppShell>
      }
    >
      <Workspace />
    </Suspense>
  );
}
