"use client";

import { Suspense, useCallback, useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import {
  ArrowLeft,
  ArrowRight,
  BookOpen,
  Check,
  CircleDot,
  Dumbbell,
  GraduationCap,
  Lightbulb,
  Lock,
  Target,
} from "lucide-react";
import { toast } from "sonner";
import { AppShell } from "@/src/components/anvil/app-shell";
import { Markdown } from "@/src/components/anvil/markdown";
import { Spinner } from "@/src/components/anvil/spinner";
import {
  getCurriculum,
  getLesson,
  getLessonProgress,
  getUnit,
  listProblems,
  recordLessonProgress,
} from "@/src/lib/api";
import { cn } from "@/src/lib/utils";
import type {
  Curriculum,
  Lesson,
  LessonStatus,
  ProblemSummary,
  Unit,
} from "@/src/lib/types";

/** "two-sum" -> "Two Sum" — fallback label when a problem's statement isn't
 *  loaded (bring-your-own-statement model). */
function prettifySlug(slug: string): string {
  return slug
    .split("-")
    .map((w) => (w.length <= 2 ? w : w[0].toUpperCase() + w.slice(1)))
    .join(" ");
}

const STATUS_META: Record<
  LessonStatus,
  { label: string; className: string; icon: typeof Check }
> = {
  complete: { label: "Complete", className: "text-pass", icon: Check },
  "in-progress": {
    label: "In progress",
    className: "text-medium",
    icon: CircleDot,
  },
  "not-started": {
    label: "Not started",
    className: "text-muted-foreground",
    icon: CircleDot,
  },
};

function StatusPill({ status }: { status: LessonStatus }) {
  const meta = STATUS_META[status];
  const Icon = meta.icon;
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 text-[11px] font-semibold",
        meta.className
      )}
    >
      <Icon className="size-[13px] stroke-[2.4]" />
      {meta.label}
    </span>
  );
}

/* --------------------------------------------------------------------- */
/* Course overview                                                       */
/* --------------------------------------------------------------------- */

function CourseView() {
  const [curriculum, setCurriculum] = useState<Curriculum | null>(null);
  const [units, setUnits] = useState<Map<string, Unit>>(new Map());
  const [lessons, setLessons] = useState<Map<string, Lesson>>(new Map());
  const [progress, setProgress] = useState<Map<string, LessonStatus>>(
    new Map()
  );

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const c = await getCurriculum();
      const unitIds = c.stages.flatMap((s) => s.units);
      const [unitList, prog] = await Promise.all([
        Promise.all(unitIds.map((id) => getUnitSafe(id))),
        getLessonProgress(),
      ]);
      const unitMap = new Map<string, Unit>();
      for (const u of unitList) if (u) unitMap.set(u.id, u);

      // Fetch the (few) authored lessons so we can show their sub-pattern titles.
      const lessonIds = [...unitMap.values()].flatMap((u) => u.lessons);
      const lessonList = await Promise.all(lessonIds.map((id) => getLesson(id)));
      const lessonMap = new Map<string, Lesson>();
      for (const l of lessonList) if (l) lessonMap.set(l.id, l);

      if (cancelled) return;
      setCurriculum(c);
      setUnits(unitMap);
      setLessons(lessonMap);
      setProgress(new Map(prog.map((p) => [p.lessonId, p.status])));
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (!curriculum) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Spinner className="size-6" />
      </div>
    );
  }

  const allLessonIds = [...units.values()].flatMap((u) => u.lessons);
  const doneCount = allLessonIds.filter(
    (id) => progress.get(id) === "complete"
  ).length;

  return (
    <main className="min-h-0 flex-1 overflow-auto px-7 pb-10 pt-6">
      <div className="mx-auto w-full max-w-[920px]">
        <div
          className="rise flex items-start gap-3"
          style={{ "--rise-i": 0 } as React.CSSProperties}
        >
          <div className="mt-0.5 flex size-9 items-center justify-center rounded-lg bg-gradient-to-b from-primary to-primary/80 text-primary-foreground shadow-sm">
            <GraduationCap className="size-5" />
          </div>
          <div>
            <h1 className="text-[21px] font-semibold tracking-tight">
              The DSA Course
            </h1>
            <p className="mt-0.5 text-[13px] text-muted-foreground">
              Learn to recognize which pattern an unseen problem needs — one
              sub-pattern per lesson, solved on real problems.
            </p>
          </div>
        </div>

        {allLessonIds.length > 0 && (
          <div
            className="rise mt-4 flex items-center gap-3 rounded-lg border bg-card px-4 py-2.5 text-[12.5px]"
            style={{ "--rise-i": 1 } as React.CSSProperties}
          >
            <span className="microlabel">Progress</span>
            <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-primary transition-all"
                style={{
                  width: `${(doneCount / allLessonIds.length) * 100}%`,
                }}
              />
            </div>
            <span className="font-mono text-xs text-muted-foreground">
              {doneCount}/{allLessonIds.length} lessons
            </span>
          </div>
        )}

        <div className="mt-7 flex flex-col gap-8">
          {curriculum.stages.map((stage, si) => (
            <section
              key={stage.id}
              className="rise"
              style={{ "--rise-i": si + 2 } as React.CSSProperties}
            >
              <div className="mb-3 flex items-center gap-2.5">
                <span className="microlabel text-foreground">{stage.title}</span>
                <span className="h-px flex-1 bg-border" />
              </div>

              <div className="flex flex-col gap-3">
                {stage.units.map((uid) => {
                  const unit = units.get(uid);
                  if (!unit) return null;
                  return (
                    <UnitCard
                      key={uid}
                      unit={unit}
                      lessons={lessons}
                      progress={progress}
                    />
                  );
                })}
              </div>
            </section>
          ))}
        </div>
      </div>
    </main>
  );
}

function UnitCard({
  unit,
  lessons,
  progress,
}: {
  unit: Unit;
  lessons: Map<string, Lesson>;
  progress: Map<string, LessonStatus>;
}) {
  const hasLessons = unit.lessons.length > 0;
  return (
    <div className="rounded-lg border bg-card">
      <div className="flex items-center justify-between px-4 py-3">
        <div>
          <h2 className="text-[15px] font-semibold">{unit.title}</h2>
          <p className="mt-0.5 text-[11.5px] text-muted-foreground">
            {unit.problems.length} problems
            {unit.prereqs.length > 0 &&
              ` · builds on ${unit.prereqs.join(", ")}`}
          </p>
        </div>
      </div>

      {hasLessons ? (
        <ul className="border-t">
          {unit.lessons.map((lid) => {
            const lesson = lessons.get(lid);
            const status = progress.get(lid) ?? "not-started";
            return (
              <li key={lid} className="border-b last:border-b-0">
                <Link
                  href={`/learn?lesson=${lid}`}
                  className="flex items-center gap-3 px-4 py-2.5 transition-colors hover:bg-accent"
                >
                  <BookOpen className="size-[15px] shrink-0 text-primary" />
                  <span className="min-w-0 flex-1 truncate text-[13.5px] font-medium">
                    {lesson?.subpattern ?? prettifySlug(lid)}
                  </span>
                  <StatusPill status={status} />
                  <ArrowRight className="size-[14px] text-muted-foreground" />
                </Link>
              </li>
            );
          })}
        </ul>
      ) : (
        <div className="flex items-center gap-2 border-t px-4 py-2.5 text-[12px] text-muted-foreground">
          <Lock className="size-[13px]" />
          Lessons for this unit arrive in a later phase.
        </div>
      )}
    </div>
  );
}

/* --------------------------------------------------------------------- */
/* Single lesson                                                         */
/* --------------------------------------------------------------------- */

function LessonView({ lessonId }: { lessonId: string }) {
  const router = useRouter();
  const [lesson, setLesson] = useState<Lesson | null>(null);
  const [unitTitle, setUnitTitle] = useState<string>("");
  const [status, setStatus] = useState<LessonStatus>("not-started");
  const [problems, setProblems] = useState<Map<string, ProblemSummary>>(
    new Map()
  );
  const [notFound, setNotFound] = useState(false);
  const recordedFor = useRef<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const [l, prog, summaries] = await Promise.all([
        getLesson(lessonId),
        getLessonProgress(),
        listProblems(),
      ]);
      if (cancelled) return;
      if (!l) {
        setNotFound(true);
        return;
      }
      setLesson(l);
      setProblems(new Map(summaries.map((s) => [s.id, s])));
      setStatus(prog.find((p) => p.lessonId === lessonId)?.status ?? "not-started");

      const unit = await getUnitSafe(l.unit);
      if (!cancelled) setUnitTitle(unit?.title ?? l.unit);

      // Opening a lesson marks it in-progress (once), unless already complete.
      if (recordedFor.current !== lessonId) {
        recordedFor.current = lessonId;
        const current = prog.find((p) => p.lessonId === lessonId)?.status;
        if (current !== "complete") {
          recordLessonProgress(lessonId, "in-progress")
            .then(() => !cancelled && setStatus("in-progress"))
            .catch(() => undefined);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [lessonId]);

  const markComplete = useCallback(async () => {
    try {
      await recordLessonProgress(lessonId, "complete");
      setStatus("complete");
      toast.success("Lesson marked complete.");
      router.push("/learn");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not save progress");
    }
  }, [lessonId, router]);

  if (notFound) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3">
        <p className="text-sm text-muted-foreground">Lesson not found.</p>
        <Link href="/learn" className="text-sm font-medium text-primary">
          ← Back to the course
        </Link>
      </div>
    );
  }
  if (!lesson) {
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

  return (
    <main className="min-h-0 flex-1 overflow-auto px-7 pb-16 pt-6">
      <div className="mx-auto w-full max-w-[760px]">
        <Link
          href="/learn"
          className="rise inline-flex items-center gap-1.5 text-[12.5px] font-medium text-muted-foreground transition-colors hover:text-foreground"
          style={{ "--rise-i": 0 } as React.CSSProperties}
        >
          <ArrowLeft className="size-[14px]" />
          {unitTitle || "Course"}
        </Link>

        <div
          className="rise mt-3 flex items-start justify-between gap-4"
          style={{ "--rise-i": 1 } as React.CSSProperties}
        >
          <h1 className="text-[24px] font-semibold leading-tight tracking-tight">
            {lesson.subpattern}
          </h1>
          <div className="mt-1.5 shrink-0">
            <StatusPill status={status} />
          </div>
        </div>

        {/* Trigger signals — the recognition cue, taught explicitly. */}
        {lesson.trigger_signals.length > 0 && (
          <section
            className="rise mt-6 rounded-lg border bg-surface-2 p-4"
            style={{ "--rise-i": 2 } as React.CSSProperties}
          >
            <div className="flex items-center gap-2">
              <Target className="size-[15px] text-medium" />
              <span className="microlabel text-foreground">Trigger signals</span>
              <span className="font-mono text-[11px] text-muted-foreground">
                when to reach for this
              </span>
            </div>
            <ul className="mt-3 flex flex-col gap-2">
              {lesson.trigger_signals.map((sig, i) => (
                <li key={i} className="flex items-start gap-2.5 text-[13.5px]">
                  <span className="mt-[7px] size-1.5 shrink-0 rounded-full bg-medium" />
                  <Markdown className="[&_p]:!my-0">{sig}</Markdown>
                </li>
              ))}
            </ul>
          </section>
        )}

        {/* Explainer prose. */}
        <section
          className="rise mt-6"
          style={{ "--rise-i": 3 } as React.CSSProperties}
        >
          <Markdown>{lesson.explainer_md}</Markdown>
        </section>

        {/* Worked example — solve it in the real workspace/runner. */}
        <section
          className="rise mt-7"
          style={{ "--rise-i": 4 } as React.CSSProperties}
        >
          <div className="flex items-center gap-2">
            <Lightbulb className="size-[15px] text-primary" />
            <span className="microlabel text-foreground">Worked example</span>
          </div>
          <Link
            href={`/problem?id=${lesson.worked_example}`}
            className="mt-3 flex items-center gap-3 rounded-lg border bg-card px-4 py-3 transition-colors hover:bg-accent"
          >
            <span className="min-w-0 flex-1">
              <span className="block text-[14px] font-semibold">
                {problemLabel(lesson.worked_example)}
              </span>
              <span className="text-[11.5px] text-muted-foreground">
                Open in the workspace and solve it against the frozen tests.
              </span>
            </span>
            <span className="inline-flex shrink-0 items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-[12.5px] font-semibold text-primary-foreground">
              Solve
              <ArrowRight className="size-[14px] stroke-[2.2]" />
            </span>
          </Link>
        </section>

        {/* Practice — faded → independent. */}
        {lesson.practice.length > 0 && (
          <section
            className="rise mt-7"
            style={{ "--rise-i": 5 } as React.CSSProperties}
          >
            <div className="flex items-center gap-2">
              <Dumbbell className="size-[15px] text-primary" />
              <span className="microlabel text-foreground">Practice</span>
              <span className="font-mono text-[11px] text-muted-foreground">
                apply it yourself
              </span>
            </div>
            <ul className="mt-3 flex flex-col gap-2">
              {lesson.practice.map((slug) => (
                <li key={slug}>
                  <Link
                    href={`/problem?id=${slug}`}
                    className="flex items-center gap-3 rounded-lg border bg-card px-4 py-2.5 transition-colors hover:bg-accent"
                  >
                    <BookOpen className="size-[15px] shrink-0 text-muted-foreground" />
                    <span className="min-w-0 flex-1 truncate text-[13.5px] font-medium">
                      {problemLabel(slug)}
                    </span>
                    <ArrowRight className="size-[14px] text-muted-foreground" />
                  </Link>
                </li>
              ))}
            </ul>
          </section>
        )}

        {/* Follow-up ladder — where interview difficulty lives. */}
        {lesson.follow_up.length > 0 && (
          <section
            className="rise mt-7 rounded-lg border border-dashed p-4"
            style={{ "--rise-i": 6 } as React.CSSProperties}
          >
            <span className="microlabel text-foreground">Push further</span>
            <ul className="mt-2.5 flex flex-col gap-2">
              {lesson.follow_up.map((f, i) => (
                <li key={i} className="flex items-start gap-2.5 text-[13px]">
                  <ArrowRight className="mt-0.5 size-[14px] shrink-0 text-muted-foreground" />
                  <Markdown className="[&_p]:!my-0">{f}</Markdown>
                </li>
              ))}
            </ul>
          </section>
        )}

        {/* Complete. */}
        <div
          className="rise mt-8 flex items-center justify-between gap-4 rounded-lg border bg-card px-4 py-3.5"
          style={{ "--rise-i": 7 } as React.CSSProperties}
        >
          <p className="text-[12.5px] text-muted-foreground">
            {status === "complete"
              ? "You've completed this lesson. Revisit it any time."
              : "Solved the worked example and practice? Mark this lesson done."}
          </p>
          <button
            type="button"
            onClick={markComplete}
            disabled={status === "complete"}
            className={cn(
              "inline-flex shrink-0 items-center gap-1.5 rounded-md px-4 py-2 text-[13px] font-semibold transition-[filter,transform] active:scale-[0.98]",
              status === "complete"
                ? "cursor-default bg-pass/15 text-pass"
                : "bg-primary text-primary-foreground hover:brightness-110"
            )}
          >
            <Check className="size-[15px] stroke-[2.4]" />
            {status === "complete" ? "Completed" : "Mark complete"}
          </button>
        </div>
      </div>
    </main>
  );
}

/* --------------------------------------------------------------------- */

/** getUnit that never throws — a missing unit just yields null. */
async function getUnitSafe(id: string): Promise<Unit | null> {
  try {
    return await getUnit(id);
  } catch {
    return null;
  }
}

function LearnRouter() {
  const searchParams = useSearchParams();
  const lessonId = searchParams.get("lesson");
  return lessonId ? <LessonView lessonId={lessonId} /> : <CourseView />;
}

export default function LearnPage() {
  return (
    <AppShell>
      <Suspense
        fallback={
          <div className="flex flex-1 items-center justify-center">
            <Spinner className="size-6" />
          </div>
        }
      >
        <LearnRouter />
      </Suspense>
    </AppShell>
  );
}
