"use client";

import { Check, CircleAlert, FlaskConical, TriangleAlert, X } from "lucide-react";
import { Spinner } from "@/src/components/anvil/spinner";
import { cn } from "@/src/lib/utils";
import type { CaseResult, ValidationResult } from "@/src/lib/types";

export type VerifyState = null | "running" | ValidationResult;

function CaseRow({ result }: { result: CaseResult }) {
  return (
    <div
      className={cn(
        "rounded-[9px] border px-3 py-2",
        result.passed
          ? "bg-surface-2"
          : "border-fail/30 bg-fail/5 dark:bg-fail/10"
      )}
    >
      <div className="flex items-center gap-2">
        {result.passed ? (
          <Check className="size-3.5 stroke-[2.6] text-pass" />
        ) : (
          <X className="size-[13px] stroke-[2.8] text-fail" />
        )}
        <span className="text-[12.5px] font-semibold">Case {result.index}</span>
        {result.hidden && (
          <span className="rounded-full border px-1.5 py-px text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
            hidden
          </span>
        )}
        <span
          className={cn(
            "ml-auto text-[11.5px] font-semibold",
            result.passed ? "text-pass" : "text-fail"
          )}
        >
          {result.passed ? "Passed" : "Failed"}
        </span>
      </div>
      {!result.passed && (result.input || result.error) && (
        <div className="mt-1.5 max-h-40 select-text overflow-y-auto break-all font-mono text-xs leading-[1.85] text-muted-foreground">
          {result.input && (
            <div>
              Input&nbsp;&nbsp;&nbsp;&nbsp;
              <span className="text-foreground">{result.input}</span>
            </div>
          )}
          {result.expected !== undefined && (
            <div>
              Expected&nbsp;
              <span className="font-medium text-pass">{result.expected}</span>
            </div>
          )}
          {result.output !== undefined && (
            <div>
              Got&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
              <span className="font-medium text-fail">{result.output}</span>
            </div>
          )}
          {result.error && <div className="text-fail">{result.error}</div>}
        </div>
      )}
    </div>
  );
}

/**
 * The Forge "prove it works" panel: runs the reference solution against
 * every test case in the sandbox (via `validateUserProblem` — nothing is
 * saved) and shows per-case verdicts, so authors never publish a problem
 * whose expected outputs don't match a working solution.
 */
export function VerificationPanel({
  solutionProvided,
  state,
  onRun,
}: {
  solutionProvided: boolean;
  state: VerifyState;
  onRun: () => void;
}) {
  const result = typeof state === "object" && state !== null ? state : null;
  const cases = result?.caseResults ?? null;
  const failed = cases?.filter((c) => !c.passed) ?? [];

  return (
    <div>
      {!solutionProvided && (
        <div className="mb-3 flex items-start gap-2.5 rounded-[10px] border border-medium/40 bg-medium/5 px-[13px] py-[11px] text-[12.5px] leading-relaxed dark:bg-medium/10">
          <TriangleAlert className="mt-px size-[15px] shrink-0 text-medium" />
          <span className="text-muted-foreground">
            <span className="font-semibold text-foreground">
              No reference solution yet
            </span>{" "}
            (previous step). Verification runs it against your test cases to
            prove the problem is solvable and every expected output is right.
            Without one, the problem publishes{" "}
            <span className="font-semibold text-foreground">unverified</span>.
          </span>
        </div>
      )}

      <button
        type="button"
        onClick={onRun}
        disabled={state === "running"}
        className="flex items-center gap-2 rounded-md border border-primary/30 bg-primary/10 px-3.5 py-2 text-[13px] font-semibold text-primary transition-[filter] hover:brightness-105 disabled:cursor-progress dark:border-primary/40 dark:bg-primary/15"
      >
        {state === "running" ? (
          <>
            <Spinner className="size-[13px] border-primary/40 border-t-primary" />
            Verifying in the sandbox…
          </>
        ) : (
          <>
            <FlaskConical className="size-[14px]" />
            Run verification
          </>
        )}
      </button>

      {result && !result.ok && (
        <div className="mt-3 rounded-[10px] border border-fail/30 bg-fail/5 px-[13px] py-[11px] dark:bg-fail/10">
          <div className="flex items-center gap-2 text-[13px] font-semibold text-fail">
            <CircleAlert className="size-4 stroke-[2.2]" />
            Verification failed
          </div>
          <div className="mt-1.5 flex flex-col gap-1">
            {result.issues.map((issue, i) => (
              <div key={i} className="text-[12.5px] leading-relaxed">
                <span className="font-semibold">{issue.field}</span>{" "}
                <span className="select-text text-muted-foreground">
                  — {issue.message}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {result?.ok && cases && (
        <div className="mt-3">
          {failed.length === 0 ? (
            <div className="flex items-center gap-2.5 rounded-[10px] border border-pass/30 bg-pass/5 px-[13px] py-[11px] dark:bg-pass/10">
              <div className="flex size-[22px] shrink-0 items-center justify-center rounded-full bg-pass">
                <Check className="size-[13px] stroke-[3] text-white" />
              </div>
              <div>
                <div className="text-[13.5px] font-semibold text-pass">
                  Verified — the reference solution passed all {cases.length}{" "}
                  test case{cases.length === 1 ? "" : "s"}.
                </div>
                <div className="mt-px text-[11.5px] text-muted-foreground">
                  Solvers will be judged against exactly these cases.
                </div>
              </div>
            </div>
          ) : (
            <div className="flex items-center gap-2.5 rounded-[10px] border border-fail/30 bg-fail/5 px-[13px] py-[11px] dark:bg-fail/10">
              <div className="flex size-[22px] shrink-0 items-center justify-center rounded-full bg-fail">
                <X className="size-3 stroke-[3] text-white" />
              </div>
              <div className="text-[13.5px] font-semibold text-fail">
                {failed.length} of {cases.length} test case
                {cases.length === 1 ? "" : "s"} failed — fix the expected
                value{failed.length === 1 ? "" : "s"} or the solution.
              </div>
            </div>
          )}
          <div className="mt-2 flex flex-col gap-1.5">
            {cases.map((c) => (
              <CaseRow key={`${c.hidden}-${c.index}`} result={c} />
            ))}
          </div>
        </div>
      )}

      {result?.ok && !cases && (
        <div className="mt-3 flex items-start gap-2.5 rounded-[10px] border border-medium/40 bg-medium/5 px-[13px] py-[11px] text-[12.5px] leading-relaxed dark:bg-medium/10">
          <TriangleAlert className="mt-px size-[15px] shrink-0 text-medium" />
          <span className="text-muted-foreground">
            The schema checks passed, but the tests were not executed — add a
            reference solution (and make sure a Python or Node runtime is
            detected in Settings → Runtime) to fully verify before publishing.
          </span>
        </div>
      )}
    </div>
  );
}
