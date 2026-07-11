"use client";

import { Suspense, useCallback, useEffect, useState } from "react";
import Link from "next/link";
import {
  ArrowRight,
  CalendarClock,
  Check,
  Flame,
  Layers,
  RotateCcw,
  Snowflake,
  Sparkles,
} from "lucide-react";
import { toast } from "sonner";
import { AppShell } from "@/src/components/anvil/app-shell";
import { PatternBadge } from "@/src/components/anvil/pattern-badge";
import { Spinner } from "@/src/components/anvil/spinner";
import {
  getProgression,
  getReviewQueue,
  listProblems,
  recordReview,
} from "@/src/lib/api";
import { cn } from "@/src/lib/utils";
import type {
  Pattern,
  ProblemSummary,
  ReviewItem,
  ReviewQueue,
  ReviewRating,
  UnitProgress,
} from "@/src/lib/types";

/** "two-sum" -> "Two Sum" — fallback label when a problem's statement isn't
 *  loaded (bring-your-own-statement model). */
function prettifySlug(slug: string): string {
  return slug
    .split("-")
    .map((w) => (w.length <= 2 ? w : w[0].toUpperCase() + w.slice(1)))
    .join(" ");
}

/** The four FSRS grades, warmest-to-coolest, each on its own hue. `good` is the
 *  expected default for a clean cold re-solve; `again` is the failure grade that
 *  demotes the card. */
const GRADES: {
  rating: ReviewRating;
  label: string;
  hint: string;
  className: string;
}[] = [
  {
    rating: "again",
    label: "Again",
    hint: "couldn't solve it",
    className: "border-fail/40 text-fail hover:bg-fail/10",
  },
  {
    rating: "hard",
    label: "Hard",
    hint: "struggled",
    className: "border-medium/40 text-medium hover:bg-medium/10",
  },
  {
    rating: "good",
    label: "Good",
    hint: "solved it",
    className: "border-primary/40 text-primary hover:bg-primary/10",
  },
  {
    rating: "easy",
    label: "Easy",
    hint: "effortless",
    className: "border-pass/40 text-pass hover:bg-pass/10",
  },
];

function ReviewContent() {
  const [queue, setQueue] = useState<ReviewQueue | null>(null);
  const [problems, setProblems] = useState<Map<string, ProblemSummary>>(
    new Map()
  );
  const [progression, setProgression] = useState<UnitProgress[]>([]);
  const [pending, setPending] = useState<Set<string>>(new Set());

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const [q, summaries, prog] = await Promise.all([
        getReviewQueue(),
        listProblems(),
        getProgression(),
      ]);
      if (cancelled) return;
      setQueue(q);
      setProblems(new Map(summaries.map((s) => [s.id, s])));
      setProgression(prog);
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const grade = useCallback(
    async (item: ReviewItem, rating: ReviewRating) => {
      setPending((p) => new Set(p).add(item.problemId));
      try {
        const out = await recordReview(item.problemId, rating);
        // Drop it from the due list; it's now scheduled for later.
        setQueue((q) =>
          q
            ? {
                ...q,
                due: q.due.filter((i) => i.problemId !== item.problemId),
                laterCount: q.laterCount + 1,
                habit: {
                  ...q.habit,
                  dueToday: Math.max(0, q.habit.dueToday - 1),
                  reviewedToday: q.habit.reviewedToday + 1,
                },
              }
            : q
        );
        toast.success(
          out.demoted
            ? "Demoted — back in rotation soon. That's the point: you found the gap."
            : `Nice — next cold re-solve in ${out.intervalDays} day${out.intervalDays === 1 ? "" : "s"}.`
        );
      } catch (err) {
        toast.error(
          err instanceof Error ? err.message : "Could not record the review"
        );
      } finally {
        setPending((p) => {
          const next = new Set(p);
          next.delete(item.problemId);
          return next;
        });
      }
    },
    []
  );

  if (!queue) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Spinner className="size-6" />
      </div>
    );
  }

  const problemLabel = (slug: string) => {
    const p = problems.get(slug);
    return p ? `${p.number}. ${p.title}` : prettifySlug(slug);
  };
  const problemPattern = (item: ReviewItem): Pattern | null =>
    problems.get(item.problemId)?.pattern ?? null;

  const masteredCount = progression.filter(
    (p) => p.status === "mastered"
  ).length;
  const unitTotal = progression.length;
  const { habit } = queue;

  return (
    <main className="min-h-0 flex-1 overflow-auto px-7 pb-16 pt-6">
      <div className="mx-auto w-full max-w-[760px]">
        {/* Header */}
        <div
          className="rise flex items-start gap-3"
          style={{ "--rise-i": 0 } as React.CSSProperties}
        >
          <div className="mt-0.5 flex size-9 items-center justify-center rounded-lg bg-gradient-to-b from-primary to-primary/80 text-primary-foreground shadow-sm">
            <RotateCcw className="size-5" />
          </div>
          <div>
            <h1 className="text-[21px] font-semibold tracking-tight">
              Spaced review
            </h1>
            <p className="mt-0.5 text-[13px] text-muted-foreground">
              Solved problems come back on a widening schedule — re-solved{" "}
              <span className="font-medium text-foreground">cold</span>, from
              scratch, interleaved across patterns. Retrieval, not re-reading.
            </p>
          </div>
        </div>

        {/* Honest habit layer: streak-with-freezes + the filling ladder. */}
        <div
          className="rise mt-5 grid grid-cols-1 gap-3 sm:grid-cols-3"
          style={{ "--rise-i": 1 } as React.CSSProperties}
        >
          <div className="rounded-lg border bg-card px-4 py-3">
            <div className="flex items-center gap-1.5">
              <Flame
                className={cn(
                  "size-[15px]",
                  habit.currentStreak > 0
                    ? "text-medium"
                    : "text-muted-foreground"
                )}
              />
              <span className="microlabel text-foreground">Streak</span>
            </div>
            <div className="mt-1.5 flex items-baseline gap-1.5">
              <span className="font-mono text-[20px] font-semibold tabular-nums">
                {habit.currentStreak}
              </span>
              <span className="text-[11.5px] text-muted-foreground">
                day{habit.currentStreak === 1 ? "" : "s"}
              </span>
            </div>
            {habit.freezeActive ? (
              <p className="mt-1 flex items-center gap-1 text-[11px] font-medium text-primary">
                <Snowflake className="size-[11px]" />
                Freeze holding it — don&apos;t miss again
              </p>
            ) : (
              <p className="mt-1 text-[11px] text-muted-foreground">
                best {habit.bestStreak} · never miss twice
              </p>
            )}
          </div>

          <div className="rounded-lg border bg-card px-4 py-3">
            <div className="flex items-center gap-1.5">
              <CalendarClock className="size-[15px] text-primary" />
              <span className="microlabel text-foreground">Today</span>
            </div>
            <div className="mt-1.5 flex items-baseline gap-1.5">
              <span className="font-mono text-[20px] font-semibold tabular-nums">
                {queue.due.length}
              </span>
              <span className="text-[11.5px] text-muted-foreground">due</span>
            </div>
            <p className="mt-1 text-[11px] text-muted-foreground">
              {habit.reviewedToday} reviewed · {queue.laterCount} scheduled
            </p>
          </div>

          <div className="rounded-lg border bg-card px-4 py-3">
            <div className="flex items-center gap-1.5">
              <Layers className="size-[15px] text-pass" />
              <span className="microlabel text-foreground">Ladder</span>
            </div>
            <div className="mt-1.5 flex items-baseline gap-1.5">
              <span className="font-mono text-[20px] font-semibold tabular-nums">
                {masteredCount}
              </span>
              <span className="text-[11.5px] text-muted-foreground">
                /{unitTotal} mastered
              </span>
            </div>
            <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-pass transition-all"
                style={{
                  width: `${unitTotal > 0 ? (masteredCount / unitTotal) * 100 : 0}%`,
                }}
              />
            </div>
          </div>
        </div>

        {/* Due queue */}
        <section
          className="rise mt-8"
          style={{ "--rise-i": 2 } as React.CSSProperties}
        >
          <div className="flex items-center gap-2">
            <Sparkles className="size-[15px] text-primary" />
            <span className="microlabel text-foreground">Due now</span>
            <span className="font-mono text-[11px] text-muted-foreground">
              interleaved — no two of a kind in a row
            </span>
          </div>

          {queue.due.length === 0 ? (
            <div className="mt-4 flex flex-col items-center gap-2 rounded-xl border border-dashed px-6 py-12 text-center">
              <Check className="size-6 text-pass" />
              <p className="text-[14px] font-semibold">All caught up.</p>
              <p className="max-w-[42ch] text-[12.5px] text-muted-foreground">
                {queue.laterCount > 0
                  ? `${queue.laterCount} problem${queue.laterCount === 1 ? "" : "s"} scheduled for later — solve more Stage-1 problems to grow the queue.`
                  : "Solve or gate a Stage-1 problem and it'll enter the review queue to come back cold on a schedule."}
              </p>
              <Link
                href="/learn"
                className="mt-1 inline-flex items-center gap-1.5 text-[12.5px] font-medium text-primary"
              >
                Back to the course
                <ArrowRight className="size-[13px]" />
              </Link>
            </div>
          ) : (
            <ul className="mt-4 flex flex-col gap-3">
              {queue.due.map((item) => {
                const pattern = problemPattern(item);
                const busy = pending.has(item.problemId);
                return (
                  <li
                    key={item.problemId}
                    className={cn(
                      "rounded-xl border bg-card p-4 transition-opacity",
                      busy && "pointer-events-none opacity-50"
                    )}
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-[14px] font-semibold">
                            {problemLabel(item.problemId)}
                          </span>
                          {pattern ? (
                            <PatternBadge pattern={pattern} />
                          ) : (
                            <span className="inline-flex items-center rounded-md bg-muted px-2.5 py-1 text-xs font-medium text-muted-foreground">
                              {prettifySlug(item.unitId)}
                            </span>
                          )}
                        </div>
                        <p className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[11.5px] text-muted-foreground">
                          {item.overdueDays > 0 && (
                            <span>
                              {item.overdueDays}d overdue
                            </span>
                          )}
                          {item.lapses > 0 && (
                            <span className="inline-flex items-center gap-1 text-fail">
                              <RotateCcw className="size-[11px]" />
                              demoted ×{item.lapses}
                            </span>
                          )}
                          {item.overdueDays === 0 && item.lapses === 0 && (
                            <span>fresh — re-solve it from a blank editor</span>
                          )}
                        </p>
                      </div>
                      <Link
                        href={`/problem?id=${item.problemId}`}
                        className="inline-flex shrink-0 items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-[12.5px] font-semibold text-primary-foreground transition-[filter] hover:brightness-110"
                      >
                        Re-solve
                        <ArrowRight className="size-[13px] stroke-[2.2]" />
                      </Link>
                    </div>

                    {/* Grade your recall after re-solving cold. */}
                    <div className="mt-3 border-t pt-3">
                      <p className="mb-2 text-[11px] text-muted-foreground">
                        Re-solved it? Grade how it went — this schedules the next
                        cold review.
                      </p>
                      <div className="grid grid-cols-4 gap-2">
                        {GRADES.map((g) => (
                          <button
                            key={g.rating}
                            type="button"
                            disabled={busy}
                            onClick={() => grade(item, g.rating)}
                            className={cn(
                              "flex flex-col items-center gap-0.5 rounded-md border py-1.5 text-[12px] font-semibold transition-colors active:scale-[0.98] disabled:opacity-50",
                              g.className
                            )}
                          >
                            {g.label}
                            <span className="text-[9.5px] font-normal opacity-70">
                              {g.hint}
                            </span>
                          </button>
                        ))}
                      </div>
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </section>
      </div>
    </main>
  );
}

export default function ReviewPage() {
  return (
    <AppShell>
      <Suspense
        fallback={
          <div className="flex flex-1 items-center justify-center">
            <Spinner className="size-6" />
          </div>
        }
      >
        <ReviewContent />
      </Suspense>
    </AppShell>
  );
}
