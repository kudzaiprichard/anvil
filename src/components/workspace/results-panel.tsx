"use client";

import { Check, CircleAlert, Star, X } from "lucide-react";
import { Spinner } from "@/src/components/anvil/spinner";
import { cn } from "@/src/lib/utils";
import { paramNames } from "@/src/lib/api";
import type { CaseResult, Problem, RunResult } from "@/src/lib/types";

export type RunState = "idle" | "running" | RunResult;

export type ResultsTab = "testcase" | "result";

function CaseCard({ result }: { result: CaseResult }) {
  const label = result.hidden
    ? `Hidden case ${result.index}`
    : `Case ${result.index}`;
  if (result.passed) {
    return (
      <div className="rounded-[9px] border bg-surface-2 px-3 py-2.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 text-[12.5px] font-semibold">
            <Check className="size-3.5 stroke-[2.6] text-pass" />
            {label}
          </div>
          <span className="text-[11.5px] font-semibold text-pass">Passed</span>
        </div>
        {!result.hidden && result.input && (
          <div className="mt-2 max-h-48 select-text overflow-y-auto break-all font-mono text-xs leading-[1.85] text-muted-foreground">
            <div>
              Input&nbsp;&nbsp;&nbsp;&nbsp;
              <span className="text-foreground">{result.input}</span>
            </div>
            <div>
              Output&nbsp;&nbsp;&nbsp;
              <span className="text-foreground">{result.output}</span>
            </div>
            <div>
              Expected&nbsp;
              <span className="text-foreground">{result.expected}</span>
            </div>
          </div>
        )}
      </div>
    );
  }
  return (
    <div className="rounded-[9px] border-[1.5px] border-fail/30 bg-fail/5 px-3 py-[11px] dark:bg-fail/10">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-[12.5px] font-semibold">
          <X className="size-[13px] stroke-[2.8] text-fail" />
          {label}
        </div>
        <span className="text-[11.5px] font-semibold text-fail">Failed</span>
      </div>
      {!result.hidden && result.input && (
        <div className="mt-[9px] max-h-48 select-text overflow-y-auto break-all font-mono text-xs leading-[1.9] text-muted-foreground">
          <div>
            Input&nbsp;&nbsp;&nbsp;&nbsp;
            <span className="text-foreground">{result.input}</span>
          </div>
          <div>
            Output&nbsp;&nbsp;&nbsp;
            <span className="font-medium text-fail">{result.output}</span>
          </div>
          <div>
            Expected&nbsp;
            <span className="font-medium text-pass">{result.expected}</span>
          </div>
        </div>
      )}
    </div>
  );
}

function ResultBody({
  result,
  onMarkMastered,
}: {
  result: RunResult;
  onMarkMastered: () => void;
}) {
  if (result.status === "error" || result.status === "timeout") {
    const isTimeout = result.status === "timeout";
    return (
      <div>
        <div className="mb-3 flex items-center gap-2.5 rounded-[10px] border border-fail/30 bg-fail/5 px-[13px] py-[11px] dark:bg-fail/10">
          <div className="flex size-[22px] shrink-0 items-center justify-center rounded-full bg-fail">
            <CircleAlert className="size-3.5 stroke-[2.4] text-white" />
          </div>
          <div>
            <div className="text-[13.5px] font-semibold text-fail">
              {isTimeout ? "Time limit exceeded" : "Runtime error"}
            </div>
            <div className="mt-px text-[11.5px] text-muted-foreground">
              {isTimeout
                ? "Your code ran past the time limit. No tests were scored."
                : "Execution stopped while running Case 1. No tests were scored."}
            </div>
          </div>
        </div>
        <div className="max-h-64 select-text overflow-auto rounded-[9px] border border-fail/30 bg-editor px-3.5 py-3 font-mono text-xs leading-[1.9]">
          {(result.error ?? "").split("\n").map((line, i, arr) => (
            <div
              key={i}
              className={cn(
                "whitespace-pre",
                i === arr.length - 1
                  ? "font-semibold text-fail"
                  : "text-muted-foreground"
              )}
            >
              {line}
            </div>
          ))}
        </div>
      </div>
    );
  }

  const allPass = result.status === "pass";
  const failedCount = result.total - result.passed;
  const firstFail = result.cases.find((c) => !c.passed);
  return (
    <div>
      {allPass ? (
        <div className="mb-3 flex items-center justify-between gap-3 rounded-[10px] border border-pass/30 bg-pass/5 px-[13px] py-[11px] dark:bg-pass/10">
          <div className="flex items-center gap-2.5">
            <div className="flex size-[22px] shrink-0 items-center justify-center rounded-full bg-pass">
              <Check className="size-[13px] stroke-[3] text-white" />
            </div>
            <div>
              <div className="text-[13.5px] font-semibold text-pass">
                All tests passed
              </div>
              <div className="mt-px font-mono text-[11.5px] text-muted-foreground">
                Runtime {result.runtimeMs} ms
                {result.memoryMb != null ? ` · Memory ${result.memoryMb} MB` : ""}
              </div>
            </div>
          </div>
          <button
            type="button"
            onClick={onMarkMastered}
            className="flex items-center gap-1.5 rounded-lg border bg-card px-[11px] py-1.5 text-xs font-semibold transition-colors hover:bg-muted"
          >
            <Star className="size-[13px]" />
            Mark mastered
          </button>
        </div>
      ) : (
        <div className="mb-3 flex items-center gap-2.5 rounded-[10px] border border-fail/30 bg-fail/5 px-[13px] py-[11px] dark:bg-fail/10">
          <div className="flex size-[22px] shrink-0 items-center justify-center rounded-full bg-fail">
            <X className="size-3 stroke-[3] text-white" />
          </div>
          <div>
            <div className="text-[13.5px] font-semibold text-fail">
              {failedCount} of {result.total} test cases failed
            </div>
            <div className="mt-px text-[11.5px] text-muted-foreground">
              {firstFail
                ? `Wrong answer on ${
                    firstFail.hidden
                      ? `Hidden case ${firstFail.index}`
                      : `Case ${firstFail.index}`
                  } — your output didn't match the expected result.`
                : ""}
            </div>
          </div>
        </div>
      )}
      <div className="flex flex-col gap-2">
        {result.cases.map((c) => (
          <CaseCard key={`${c.hidden}-${c.index}`} result={c} />
        ))}
      </div>
    </div>
  );
}

/** Bottom-right results panel: Testcase | Test Result (UI_SPEC §6.2). */
export function ResultsPanel({
  problem,
  runState,
  tab,
  onTabChange,
  selectedCase,
  onSelectCase,
  onMarkMastered,
}: {
  problem: Problem;
  runState: RunState;
  tab: ResultsTab;
  onTabChange: (tab: ResultsTab) => void;
  selectedCase: number;
  onSelectCase: (index: number) => void;
  onMarkMastered: () => void;
}) {
  const visibleCases = problem.test_cases.filter((tc) => !tc.hidden);
  const params = paramNames(problem);
  const activeCase = visibleCases[selectedCase] ?? visibleCases[0];
  const result = typeof runState === "object" ? runState : null;

  return (
    <div className="flex h-full min-h-0 flex-col bg-card">
      {/* tabs row */}
      <div className="flex h-[34px] shrink-0 items-center gap-1 border-b pl-3.5 pr-3">
        {(
          [
            { id: "testcase", label: "Testcase" },
            { id: "result", label: "Test Result" },
          ] as const
        ).map(({ id, label }) => (
          <button
            key={id}
            type="button"
            onClick={() => onTabChange(id)}
            className={cn(
              "relative px-2 py-2 text-[12.5px] transition-colors",
              tab === id
                ? "font-semibold text-foreground"
                : "font-medium text-muted-foreground hover:text-foreground"
            )}
          >
            {label}
            {tab === id && (
              <span className="absolute inset-x-1 -bottom-px h-0.5 rounded-sm bg-primary" />
            )}
          </button>
        ))}
        <div className="flex-1" />
        {runState === "running" && (
          <span className="flex items-center gap-[7px] text-xs font-medium text-muted-foreground">
            <Spinner className="size-[11px]" />
            Running…
          </span>
        )}
        {result?.status === "pass" && (
          <span className="inline-flex items-center gap-1.5 rounded-full bg-pass/10 px-2.5 py-[3px] text-xs font-semibold text-pass dark:bg-pass/15">
            <Check className="size-[13px] stroke-[2.6]" />
            {result.passed} / {result.total} passed
          </span>
        )}
        {result?.status === "fail" && (
          <span className="inline-flex items-center gap-1.5 rounded-full bg-fail/10 px-2.5 py-[3px] text-xs font-semibold text-fail dark:bg-fail/15">
            <X className="size-3 stroke-[2.6]" />
            {result.total - result.passed} / {result.total} failed
          </span>
        )}
        {(result?.status === "error" || result?.status === "timeout") && (
          <span className="inline-flex items-center rounded-full bg-fail/10 px-2.5 py-[3px] text-xs font-semibold text-fail dark:bg-fail/15">
            {result.status === "timeout" ? "Time limit exceeded" : "Runtime error"}
          </span>
        )}
      </div>

      {/* body */}
      <div className="min-h-0 flex-1 overflow-auto px-4 pb-4 pt-[13px]">
        {tab === "testcase" && (
          <div>
            <div className="flex items-center gap-1.5">
              {visibleCases.map((_, i) => (
                <button
                  key={i}
                  type="button"
                  onClick={() => onSelectCase(i)}
                  className={cn(
                    "rounded-lg px-3 py-[5px] font-mono text-[12.5px] transition-colors",
                    i === selectedCase
                      ? "border bg-surface-2 font-semibold"
                      : "font-medium text-muted-foreground hover:text-foreground"
                  )}
                >
                  Case {i + 1}
                </button>
              ))}
            </div>
            {activeCase?.input.map((arg, i) => (
              <div key={i}>
                <div
                  className={cn(
                    "text-[11.5px] font-semibold uppercase tracking-wider text-muted-foreground",
                    i === 0 ? "mt-3.5" : "mt-3"
                  )}
                >
                  {params[i] ?? `arg${i}`} =
                </div>
                <div className="mt-1.5 max-h-40 select-text overflow-y-auto break-all rounded-lg border bg-editor px-3 py-[9px] font-mono text-[13px]">
                  {JSON.stringify(arg)?.replace(/,/g, ", ")}
                </div>
              </div>
            ))}
          </div>
        )}

        {tab === "result" && runState === "idle" && (
          <div className="flex h-full min-h-[140px] items-center justify-center text-[13px] text-muted-foreground">
            Run your code to see test results here.
          </div>
        )}
        {tab === "result" && runState === "running" && (
          <div className="flex h-full min-h-40 flex-col items-center justify-center gap-3 text-muted-foreground">
            <Spinner className="size-[26px] border-[2.5px]" />
            <div className="text-[13px] font-medium">
              Running {problem.test_cases.length} test case
              {problem.test_cases.length === 1 ? "" : "s"}…
            </div>
          </div>
        )}
        {tab === "result" && result && (
          <ResultBody result={result} onMarkMastered={onMarkMastered} />
        )}
      </div>
    </div>
  );
}
