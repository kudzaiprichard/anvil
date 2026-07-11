"use client";

import { useState } from "react";
import {
  AlertTriangle,
  ArrowLeft,
  ArrowRight,
  Check,
  Flag,
  ListChecks,
  RotateCcw,
  Sparkles,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { Markdown } from "@/src/components/anvil/markdown";
import { submitQuiz } from "@/src/lib/api";
import { cn } from "@/src/lib/utils";
import type { QuizGrade, QuizItem } from "@/src/lib/types";

type Phase = "answering" | "review" | "results";

/**
 * A full quiz player (modern-LMS style): one question at a time with a question
 * navigator, **flag for review**, a **review screen** before submission, then a
 * graded **pass/fail results** screen with per-question feedback and **Retry**.
 * Grading is server-side via `submitQuiz` (formative — the same call the inline
 * runner used). `onPass` fires once when a submission clears `passPct`, so a
 * host lesson can gate progression on passing.
 */
export function QuizPlayer({
  source,
  items,
  patternLabel,
  passPct = 80,
  onPass,
}: {
  /** Lesson id, or `PATTERN_POOL_SOURCE` for the interleaved pool. */
  source: string;
  items: QuizItem[];
  /** Resolves a pattern-picker's `correct_pattern` to a title (post-answer). */
  patternLabel?: (patternId: string) => string;
  /** Pass mark, percent. Default 80. */
  passPct?: number;
  /** Fired once when a submission scores ≥ passPct. */
  onPass?: () => void;
}) {
  const [phase, setPhase] = useState<Phase>("answering");
  const [qIndex, setQIndex] = useState(0);
  const [selected, setSelected] = useState<Record<string, string>>({});
  const [flagged, setFlagged] = useState<Set<string>>(new Set());
  const [grade, setGrade] = useState<QuizGrade | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const total = items.length;
  const answeredCount = items.filter(
    (it) => selected[it.id] !== undefined
  ).length;

  const toggleFlag = (id: string) =>
    setFlagged((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const submit = async () => {
    if (submitting) return;
    setSubmitting(true);
    try {
      const g = await submitQuiz(
        source,
        items.map((it) => ({ itemId: it.id, selected: selected[it.id] ?? "" }))
      );
      setGrade(g);
      setPhase("results");
      if ((g.correctCount / g.total) * 100 >= passPct) onPass?.();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Could not grade the quiz");
    } finally {
      setSubmitting(false);
    }
  };

  const retry = () => {
    setSelected({});
    setFlagged(new Set());
    setGrade(null);
    setQIndex(0);
    setPhase("answering");
  };

  const primaryBtn =
    "inline-flex items-center gap-1.5 rounded-md bg-primary px-3.5 py-1.5 text-[12.5px] font-semibold text-primary-foreground transition-[filter,transform] hover:brightness-110 active:scale-[0.98] disabled:opacity-60";
  const borderBtn =
    "inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-[12.5px] font-medium text-muted-foreground transition-colors hover:bg-accent disabled:pointer-events-none disabled:opacity-40";

  /* ---------------------------------------------------------------- results */
  if (phase === "results" && grade) {
    const pct = Math.round((grade.correctCount / grade.total) * 100);
    const passed = pct >= passPct;
    const byId = new Map(grade.results.map((r) => [r.itemId, r]));
    return (
      <div className="overflow-hidden rounded-xl border bg-card">
        <div
          className={cn(
            "flex items-center gap-3 p-4",
            passed ? "bg-pass/10" : "bg-fail/10"
          )}
        >
          <div
            className={cn(
              "flex size-10 shrink-0 items-center justify-center rounded-full",
              passed ? "bg-pass/15 text-pass" : "bg-fail/15 text-fail"
            )}
          >
            {passed ? (
              <Check className="size-5 stroke-[2.6]" />
            ) : (
              <X className="size-5 stroke-[2.6]" />
            )}
          </div>
          <div className="min-w-0 flex-1">
            <p
              className={cn(
                "text-[15px] font-semibold",
                passed ? "text-pass" : "text-fail"
              )}
            >
              {passed ? "Passed" : "Not passed"}
            </p>
            <p className="text-[12px] text-muted-foreground">
              {grade.correctCount}/{grade.total} correct · {pct}%{" "}
              {passed
                ? `— you can continue below`
                : `· need ${passPct}% to continue`}
            </p>
          </div>
          <button type="button" onClick={retry} className={borderBtn}>
            <RotateCcw className="size-[13px]" />
            Retry
          </button>
        </div>
        <ul className="divide-y">
          {items.map((it) => {
            const r = byId.get(it.id);
            return (
              <li key={it.id} className="p-4">
                <div className="flex items-start gap-2.5">
                  <span
                    className={cn(
                      "mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full",
                      r?.correct
                        ? "bg-pass/15 text-pass"
                        : "bg-fail/15 text-fail"
                    )}
                  >
                    {r?.correct ? (
                      <Check className="size-3 stroke-[3]" />
                    ) : (
                      <X className="size-3 stroke-[3]" />
                    )}
                  </span>
                  <div className="min-w-0 flex-1">
                    <div className="text-[13px]">
                      <Markdown className="[&_p]:!my-0">{it.prompt_md}</Markdown>
                    </div>
                    {!r?.correct && (
                      <p className="mt-1.5 text-[12px] text-muted-foreground">
                        You chose{" "}
                        <span className="font-medium text-fail">
                          {r?.selected || "— (blank)"}
                        </span>{" "}
                        · answer:{" "}
                        <span className="font-medium text-pass">{r?.answer}</span>
                      </p>
                    )}
                    {r?.explanation_md && (
                      <Markdown className="mt-1.5 text-[12px] text-muted-foreground [&_p]:!my-0">
                        {r.explanation_md}
                      </Markdown>
                    )}
                    {r?.type === "pattern-picker" && r.correctPattern && (
                      <span className="mt-2 inline-flex items-center gap-1 rounded-full border border-primary/30 bg-primary/5 px-2 py-[1px] text-[10.5px] font-semibold text-primary">
                        <Sparkles className="size-[10px]" />
                        {patternLabel?.(r.correctPattern) ?? r.correctPattern}
                      </span>
                    )}
                  </div>
                </div>
              </li>
            );
          })}
        </ul>
      </div>
    );
  }

  /* ----------------------------------------------------------------- review */
  if (phase === "review") {
    return (
      <div className="rounded-xl border bg-card p-5">
        <div className="flex items-center gap-2">
          <ListChecks className="size-[16px] text-primary" />
          <h3 className="text-[14px] font-semibold">Review before you submit</h3>
        </div>
        <p className="mt-1.5 text-[12.5px] text-muted-foreground">
          {answeredCount}/{total} answered
          {flagged.size > 0 ? ` · ${flagged.size} flagged for review` : ""}. Click
          a question to revisit it.
        </p>
        <div className="mt-4 flex flex-wrap gap-2">
          {items.map((it, i) => {
            const answered = selected[it.id] !== undefined;
            return (
              <button
                key={it.id}
                type="button"
                onClick={() => {
                  setQIndex(i);
                  setPhase("answering");
                }}
                className={cn(
                  "relative flex size-9 items-center justify-center rounded-md border text-[12.5px] font-semibold transition-colors hover:bg-accent",
                  answered
                    ? "border-primary/40 bg-primary/5 text-foreground"
                    : "border-dashed text-muted-foreground"
                )}
              >
                {i + 1}
                {flagged.has(it.id) && (
                  <Flag className="absolute -right-1 -top-1 size-3 fill-medium text-medium" />
                )}
              </button>
            );
          })}
        </div>
        {answeredCount < total && (
          <p className="mt-4 flex items-center gap-1.5 text-[12px] text-medium">
            <AlertTriangle className="size-[13px]" />
            {total - answeredCount} unanswered — they&apos;ll be marked wrong.
          </p>
        )}
        <div className="mt-5 flex items-center justify-between gap-3">
          <button
            type="button"
            onClick={() => setPhase("answering")}
            className={borderBtn}
          >
            <ArrowLeft className="size-[14px]" />
            Back to questions
          </button>
          <button
            type="button"
            onClick={submit}
            disabled={submitting}
            className={primaryBtn}
          >
            {submitting ? "Submitting…" : "Submit quiz"}
            <ArrowRight className="size-[14px] stroke-[2.2]" />
          </button>
        </div>
      </div>
    );
  }

  /* -------------------------------------------------------------- answering */
  const item = items[qIndex];
  const isLastQ = qIndex === total - 1;
  return (
    <div className="rounded-xl border bg-card p-5">
      {/* question navigator */}
      <div className="flex items-center gap-2">
        <span className="microlabel text-foreground">
          Question {qIndex + 1} of {total}
        </span>
        <div className="ml-auto flex items-center gap-1.5">
          {items.map((it, i) => (
            <button
              key={it.id}
              type="button"
              onClick={() => setQIndex(i)}
              aria-label={`Go to question ${i + 1}${
                flagged.has(it.id) ? " (flagged)" : ""
              }`}
              aria-current={i === qIndex ? "true" : undefined}
              className={cn(
                "size-2.5 rounded-full transition-all",
                i === qIndex
                  ? "bg-primary ring-2 ring-primary/25"
                  : selected[it.id] !== undefined
                    ? "bg-primary/40"
                    : "bg-muted hover:bg-muted-foreground/30",
                flagged.has(it.id) &&
                  "ring-2 ring-medium/50 ring-offset-1 ring-offset-card"
              )}
            />
          ))}
        </div>
      </div>

      {/* prompt */}
      <div className="mt-3 text-[14px] leading-relaxed">
        <Markdown className="[&_p]:!my-0">{item.prompt_md}</Markdown>
      </div>

      {/* options */}
      <ul
        role="radiogroup"
        aria-label={`Question ${qIndex + 1} options`}
        className="mt-4 flex flex-col gap-1.5"
      >
        {item.options.map((opt) => {
          const isSel = selected[item.id] === opt;
          return (
            <li key={opt}>
              <button
                type="button"
                role="radio"
                aria-checked={isSel}
                onClick={() =>
                  setSelected((p) => ({ ...p, [item.id]: opt }))
                }
                className={cn(
                  "flex w-full items-center gap-2.5 rounded-md border px-3 py-2 text-left text-[13px] transition-colors hover:bg-accent",
                  isSel ? "border-primary bg-primary/5" : "border-border"
                )}
              >
                <span
                  className={cn(
                    "flex size-[16px] shrink-0 items-center justify-center rounded-full border",
                    isSel ? "border-primary" : "border-muted-foreground/40"
                  )}
                >
                  {isSel && (
                    <span className="size-[7px] rounded-full bg-primary" />
                  )}
                </span>
                <span className="min-w-0 flex-1">{opt}</span>
              </button>
            </li>
          );
        })}
      </ul>

      {/* footer: flag + prev/next/review */}
      <div className="mt-5 flex items-center justify-between gap-2">
        <button
          type="button"
          onClick={() => toggleFlag(item.id)}
          aria-pressed={flagged.has(item.id)}
          className={cn(
            "inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-[12px] font-medium transition-colors hover:bg-accent",
            flagged.has(item.id)
              ? "border-medium/40 bg-medium/10 text-medium"
              : "text-muted-foreground"
          )}
        >
          <Flag
            className={cn(
              "size-[13px]",
              flagged.has(item.id) && "fill-medium"
            )}
          />
          {flagged.has(item.id) ? "Flagged" : "Flag"}
        </button>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setQIndex((i) => Math.max(0, i - 1))}
            disabled={qIndex === 0}
            className={borderBtn}
          >
            <ArrowLeft className="size-[14px]" />
            Prev
          </button>
          {isLastQ ? (
            <button
              type="button"
              onClick={() => setPhase("review")}
              className={primaryBtn}
            >
              Review &amp; submit
              <ArrowRight className="size-[14px] stroke-[2.2]" />
            </button>
          ) : (
            <button
              type="button"
              onClick={() => setQIndex((i) => Math.min(total - 1, i + 1))}
              className={primaryBtn}
            >
              Next
              <ArrowRight className="size-[14px] stroke-[2.2]" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
