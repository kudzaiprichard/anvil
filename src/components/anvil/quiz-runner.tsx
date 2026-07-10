"use client";

import { useCallback, useMemo, useState } from "react";
import { Check, RotateCcw, Sparkles, Target, X } from "lucide-react";
import { toast } from "sonner";
import { Markdown } from "@/src/components/anvil/markdown";
import { submitQuiz } from "@/src/lib/api";
import { cn } from "@/src/lib/utils";
import type { QuizItem, QuizItemResult } from "@/src/lib/types";

/**
 * Formative quiz runner (Phase 4, LESSON_COURSE_DESIGN.md §3.4). Presents a set
 * of quiz items — concept-check, complexity, or the moat **pattern-picker**
 * (prompt-only, *unlabeled*: the pattern name is never shown until after the
 * learner commits) — grades the selection server-side via `submitQuiz`, and
 * reveals the trigger explanation. **Never blocks progression**: this is a
 * check, not a gate. A placement submits only the items it shows, so a lesson's
 * concept-check and pattern-picker sections score independently.
 */
export function QuizRunner({
  source,
  items,
  patternLabel,
  className,
}: {
  /** Lesson id, or `PATTERN_POOL_SOURCE` for the interleaved pool. */
  source: string;
  items: QuizItem[];
  /** Resolves a pattern-picker's `correct_pattern` unit id to a display title
   *  (only used in post-answer feedback — never before). */
  patternLabel?: (patternId: string) => string;
  className?: string;
}) {
  const [selected, setSelected] = useState<Record<string, string>>({});
  const [results, setResults] = useState<Map<string, QuizItemResult> | null>(
    null
  );
  const [submitting, setSubmitting] = useState(false);

  const graded = results !== null;
  const answeredAll = useMemo(
    () => items.every((it) => selected[it.id] !== undefined),
    [items, selected]
  );
  const correctCount = useMemo(
    () =>
      results ? [...results.values()].filter((r) => r.correct).length : 0,
    [results]
  );

  const pick = useCallback(
    (itemId: string, option: string) => {
      if (graded) return; // locked after checking; use "Try again" to redo
      setSelected((prev) => ({ ...prev, [itemId]: option }));
    },
    [graded]
  );

  const check = useCallback(async () => {
    if (!answeredAll || submitting) return;
    setSubmitting(true);
    try {
      const grade = await submitQuiz(
        source,
        items.map((it) => ({ itemId: it.id, selected: selected[it.id] }))
      );
      setResults(new Map(grade.results.map((r) => [r.itemId, r])));
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Could not grade the quiz"
      );
    } finally {
      setSubmitting(false);
    }
  }, [answeredAll, submitting, source, items, selected]);

  const reset = useCallback(() => {
    setSelected({});
    setResults(null);
  }, []);

  return (
    <div className={cn("flex flex-col gap-4", className)}>
      {items.map((item, i) => {
        const result = results?.get(item.id);
        return (
          <div
            key={item.id}
            className="rounded-lg border bg-card p-4"
          >
            <div className="flex items-baseline gap-2">
              <span className="font-mono text-[11px] text-muted-foreground">
                {String(i + 1).padStart(2, "0")}
              </span>
              <div className="min-w-0 flex-1">
                <Markdown
                  id={`${item.id}-prompt`}
                  className="[&_p]:!my-0 text-[13.5px]"
                >
                  {item.prompt_md}
                </Markdown>
              </div>
            </div>

            <ul
              className="mt-3 flex flex-col gap-1.5"
              role="radiogroup"
              aria-labelledby={`${item.id}-prompt`}
            >
              {item.options.map((option) => {
                const isSelected = selected[item.id] === option;
                const isAnswer = graded && option === result?.answer;
                const isWrongPick =
                  graded && isSelected && !result?.correct;
                return (
                  <li key={option}>
                    <button
                      type="button"
                      role="radio"
                      aria-checked={isSelected}
                      onClick={() => pick(item.id, option)}
                      disabled={graded}
                      className={cn(
                        "flex w-full items-center gap-2.5 rounded-md border px-3 py-2 text-left text-[13px] transition-colors",
                        !graded &&
                          "hover:bg-accent " +
                            (isSelected
                              ? "border-primary bg-primary/5"
                              : "border-border"),
                        graded && "cursor-default",
                        isAnswer && "border-pass bg-pass/10",
                        isWrongPick && "border-fail bg-fail/10",
                        graded &&
                          !isAnswer &&
                          !isWrongPick &&
                          "border-border opacity-60"
                      )}
                    >
                      <span
                        className={cn(
                          "flex size-[16px] shrink-0 items-center justify-center rounded-full border",
                          isSelected && !graded && "border-primary",
                          isAnswer && "border-pass bg-pass text-white",
                          isWrongPick && "border-fail bg-fail text-white",
                          !isSelected && !graded && "border-muted-foreground/40"
                        )}
                      >
                        {isAnswer && <Check className="size-[11px] stroke-[3]" />}
                        {isWrongPick && <X className="size-[11px] stroke-[3]" />}
                        {isSelected && !graded && (
                          <span className="size-[7px] rounded-full bg-primary" />
                        )}
                      </span>
                      <span className="min-w-0 flex-1">{option}</span>
                    </button>
                  </li>
                );
              })}
            </ul>

            {/* Feedback — revealed only after the learner commits. */}
            {result && (
              <div
                className={cn(
                  "mt-3 rounded-md border-l-2 pl-3 text-[12.5px]",
                  result.correct ? "border-pass" : "border-fail"
                )}
              >
                <p
                  className={cn(
                    "flex items-center gap-1.5 font-semibold",
                    result.correct ? "text-pass" : "text-fail"
                  )}
                >
                  {result.correct ? (
                    <>
                      <Check className="size-[13px] stroke-[2.6]" />
                      Correct
                    </>
                  ) : (
                    <>
                      <X className="size-[13px] stroke-[2.6]" />
                      Not quite
                    </>
                  )}
                  {/* Pattern-picker: the pattern name is unlabeled until now. */}
                  {result.type === "pattern-picker" && result.correctPattern && (
                    <span className="ml-auto inline-flex items-center gap-1 rounded-full border border-primary/30 bg-primary/5 px-2 py-[1px] text-[10.5px] font-semibold text-primary">
                      <Sparkles className="size-[10px]" />
                      {patternLabel?.(result.correctPattern) ??
                        result.correctPattern}
                    </span>
                  )}
                </p>
                <Markdown className="mt-1 [&_p]:!my-0 text-muted-foreground">
                  {result.explanation_md}
                </Markdown>
              </div>
            )}
          </div>
        );
      })}

      {/* Actions + score. Formative: no effect on unlock/mastery. */}
      <div className="flex items-center gap-3">
        {!graded ? (
          <button
            type="button"
            onClick={check}
            disabled={!answeredAll || submitting}
            className={cn(
              "inline-flex items-center gap-1.5 rounded-md px-4 py-2 text-[13px] font-semibold transition-[filter,opacity] active:scale-[0.98]",
              answeredAll
                ? "bg-primary text-primary-foreground hover:brightness-110"
                : "cursor-not-allowed bg-muted text-muted-foreground"
            )}
          >
            <Target className="size-[15px]" />
            {submitting ? "Checking…" : "Check answers"}
          </button>
        ) : (
          <>
            <span
              className={cn(
                "inline-flex items-center gap-1.5 rounded-md px-3 py-2 text-[13px] font-semibold",
                correctCount === items.length
                  ? "bg-pass/10 text-pass"
                  : "bg-medium/10 text-medium"
              )}
            >
              {correctCount}/{items.length} correct
            </span>
            <button
              type="button"
              onClick={reset}
              className="inline-flex items-center gap-1.5 rounded-md border px-3 py-2 text-[12.5px] font-medium text-muted-foreground transition-colors hover:bg-accent"
            >
              <RotateCcw className="size-[13px]" />
              Try again
            </button>
          </>
        )}
        {!graded && !answeredAll && (
          <span className="text-[11.5px] text-muted-foreground">
            Pick an answer for each to check. Quizzes never affect your progress.
          </span>
        )}
      </div>
    </div>
  );
}
