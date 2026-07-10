"use client";

import { Suspense, useCallback, useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import {
  ArrowLeft,
  ArrowRight,
  BookOpen,
  Brain,
  Check,
  CircleDot,
  Dumbbell,
  Film,
  GraduationCap,
  Lightbulb,
  ListChecks,
  Lock,
  Puzzle,
  ShieldCheck,
  Sparkles,
  Target,
  Trophy,
} from "lucide-react";
import { toast } from "sonner";
import { AppShell } from "@/src/components/anvil/app-shell";
import { DiagramPlayer } from "@/src/components/anvil/diagram-player";
import { Markdown } from "@/src/components/anvil/markdown";
import { QuizRunner } from "@/src/components/anvil/quiz-runner";
import { Spinner } from "@/src/components/anvil/spinner";
import {
  applyPlacement,
  getCapstone,
  getCurriculum,
  getLesson,
  getLessonProgress,
  getPatternPool,
  getPlacement,
  getProgression,
  getReadiness,
  getUnit,
  listProblems,
  recordLessonProgress,
} from "@/src/lib/api";
import { cn } from "@/src/lib/utils";
import { PATTERN_POOL_SOURCE } from "@/src/lib/types";
import type {
  CapstoneView,
  Curriculum,
  Lesson,
  LessonStatus,
  PlacementProbe,
  ProblemSummary,
  Quiz,
  QuizAnswer,
  Readiness,
  Unit,
  UnitProgress,
  UnitStatus,
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

const UNIT_STATUS_META: Record<
  UnitStatus,
  { label: string; className: string; icon: typeof Check }
> = {
  mastered: {
    label: "Mastered",
    className: "border-pass/40 bg-pass/10 text-pass",
    icon: Trophy,
  },
  unlocked: {
    label: "In progress",
    className: "border-primary/40 bg-primary/10 text-primary",
    icon: CircleDot,
  },
  locked: {
    label: "Locked",
    className: "border-border bg-muted text-muted-foreground",
    icon: Lock,
  },
};

function UnitStatusBadge({ status }: { status: UnitStatus }) {
  const meta = UNIT_STATUS_META[status];
  const Icon = meta.icon;
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-[3px] text-[11px] font-semibold",
        meta.className
      )}
    >
      <Icon className="size-[12px] stroke-[2.4]" />
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
  const [progress, setProgress] = useState<Map<string, UnitProgress>>(
    new Map()
  );
  const [readiness, setReadiness] = useState<Readiness | null>(null);
  const [capstone, setCapstone] = useState<CapstoneView | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const c = await getCurriculum();
      const unitIds = c.stages.flatMap((s) => s.units);
      const [unitList, prog, ready, cap] = await Promise.all([
        Promise.all(unitIds.map((id) => getUnitSafe(id))),
        getProgression(),
        getReadiness(),
        getCapstone(),
      ]);
      if (cancelled) return;
      const unitMap = new Map<string, Unit>();
      for (const u of unitList) if (u) unitMap.set(u.id, u);
      setCurriculum(c);
      setUnits(unitMap);
      setProgress(new Map(prog.map((p) => [p.unitId, p])));
      setReadiness(ready);
      setCapstone(cap);
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

  const masteredCount = [...progress.values()].filter(
    (p) => p.status === "mastered"
  ).length;
  const unitTotal = progress.size;

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
              One connected climb — each unit unlocks only when you pass the
              one before it, hint-free under a timer.
            </p>
          </div>
        </div>

        {unitTotal > 0 && (
          <div
            className="rise mt-4 flex flex-wrap items-center gap-3 rounded-lg border bg-card px-4 py-2.5 text-[12.5px]"
            style={{ "--rise-i": 1 } as React.CSSProperties}
          >
            <span className="microlabel">Readiness</span>
            <div className="h-1.5 min-w-[120px] flex-1 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-pass transition-all"
                style={{
                  width: `${readiness ? readiness.percent : (masteredCount / unitTotal) * 100}%`,
                }}
              />
            </div>
            <span className="font-mono text-xs text-muted-foreground">
              {masteredCount}/{unitTotal} units
              {readiness ? ` · ${readiness.percent}%` : ""}
            </span>
            {readiness?.ready && (
              <span className="inline-flex items-center gap-1 rounded-full bg-pass/15 px-2 py-0.5 text-[11px] font-medium text-pass">
                <Trophy className="size-3" /> Interview-ready
              </span>
            )}
            <Link
              href="/learn?placement=1"
              className="inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-accent"
            >
              <Target className="size-3" /> Placement test
            </Link>
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
                      progress={progress.get(uid)}
                      units={units}
                    />
                  );
                })}
              </div>
            </section>
          ))}

          {capstone && (
            <section
              className="rise"
              style={
                {
                  "--rise-i": curriculum.stages.length + 2,
                } as React.CSSProperties
              }
            >
              <div className="mb-3 flex items-center gap-2.5">
                <span className="microlabel text-foreground">
                  The final exam
                </span>
                <span className="h-px flex-1 bg-border" />
              </div>
              <Link
                href="/learn?capstone=1"
                className={cn(
                  "group flex items-center gap-4 rounded-lg border bg-card px-4 py-3.5 transition-colors hover:bg-accent",
                  !capstone.unlocked && "opacity-80"
                )}
              >
                <div className="mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-lg bg-gradient-to-b from-amber-500 to-amber-600 text-white shadow-sm">
                  <Trophy className="size-5" />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2.5">
                    <h2 className="text-[15px] font-semibold">
                      {capstone.title}
                    </h2>
                    {capstone.met ? (
                      <span className="inline-flex items-center gap-1 rounded-full bg-pass/15 px-2 py-0.5 text-[11px] font-medium text-pass">
                        <Check className="size-3" /> Cleared
                      </span>
                    ) : (
                      <span className="rounded-full bg-muted px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
                        {capstone.unlocked ? "Ready" : "Practice"}
                      </span>
                    )}
                  </div>
                  <p className="mt-1 text-[11.5px] text-muted-foreground">
                    {capstone.total} unlabeled problems — no pattern shown; you
                    decide which technique fits. Clear {capstone.passCount} to
                    prove you can solve the unfamiliar. ({capstone.passedCount}/
                    {capstone.passCount})
                  </p>
                </div>
                <ArrowRight className="size-[15px] shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
              </Link>
            </section>
          )}
        </div>
      </div>
    </main>
  );
}

function UnitCard({
  unit,
  progress,
  units,
}: {
  unit: Unit;
  progress?: UnitProgress;
  units: Map<string, Unit>;
}) {
  const status = progress?.status ?? "locked";
  const locked = status === "locked";
  const prereqTitles = (progress?.blockedBy ?? []).map(
    (id) => units.get(id)?.title ?? prettifySlug(id)
  );

  return (
    <Link
      href={`/learn?unit=${unit.id}`}
      className={cn(
        "group flex items-center gap-4 rounded-lg border bg-card px-4 py-3.5 transition-colors hover:bg-accent",
        locked && "opacity-80"
      )}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2.5">
          <h2 className="text-[15px] font-semibold">{unit.title}</h2>
          <UnitStatusBadge status={status} />
        </div>
        <p className="mt-1 text-[11.5px] text-muted-foreground">
          {locked && prereqTitles.length > 0 ? (
            <span className="inline-flex items-center gap-1.5">
              <Lock className="size-[11px]" />
              Pass {prereqTitles.join(" & ")} to unlock
            </span>
          ) : (
            <>
              {progress ? `${progress.lessonsTotal} lesson${progress.lessonsTotal === 1 ? "" : "s"}` : `${unit.lessons.length} lessons`}
              {" · "}
              {progress
                ? `gate ${progress.gate.passedCount}/${progress.gate.passCount}`
                : `${unit.problems.length} problems`}
              {status === "mastered" && " · complete"}
            </>
          )}
        </p>
      </div>
      <ArrowRight className="size-[15px] shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
    </Link>
  );
}

/* --------------------------------------------------------------------- */
/* Single unit (lessons + mastery gate)                                  */
/* --------------------------------------------------------------------- */

function UnitView({ unitId }: { unitId: string }) {
  const [unit, setUnit] = useState<Unit | null>(null);
  const [units, setUnits] = useState<Map<string, Unit>>(new Map());
  const [progress, setProgress] = useState<UnitProgress | null>(null);
  const [lessonStatus, setLessonStatus] = useState<Map<string, LessonStatus>>(
    new Map()
  );
  const [problems, setProblems] = useState<Map<string, ProblemSummary>>(
    new Map()
  );
  const [notFound, setNotFound] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const [u, prog, lprog, summaries, curriculum] = await Promise.all([
        getUnit(unitId),
        getProgression(),
        getLessonProgress(),
        listProblems(),
        getCurriculum(),
      ]);
      if (cancelled) return;
      if (!u) {
        setNotFound(true);
        return;
      }
      setUnit(u);
      setProgress(prog.find((p) => p.unitId === unitId) ?? null);
      setLessonStatus(new Map(lprog.map((p) => [p.lessonId, p.status])));
      setProblems(new Map(summaries.map((s) => [s.id, s])));

      // Titles for prereq / unlocked-next references.
      const allIds = curriculum.stages.flatMap((s) => s.units);
      const unitList = await Promise.all(allIds.map((id) => getUnitSafe(id)));
      if (cancelled) return;
      const map = new Map<string, Unit>();
      for (const un of unitList) if (un) map.set(un.id, un);
      setUnits(map);
    })();
    return () => {
      cancelled = true;
    };
  }, [unitId]);

  const problemLabel = useCallback(
    (slug: string) => {
      const p = problems.get(slug);
      return p ? `${p.number}. ${p.title}` : prettifySlug(slug);
    },
    [problems]
  );

  if (notFound) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3">
        <p className="text-sm text-muted-foreground">Unit not found.</p>
        <Link href="/learn" className="text-sm font-medium text-primary">
          ← Back to the course
        </Link>
      </div>
    );
  }
  if (!unit || !progress) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Spinner className="size-6" />
      </div>
    );
  }

  const locked = progress.status === "locked";
  const prereqTitles = progress.blockedBy.map(
    (id) => units.get(id)?.title ?? prettifySlug(id)
  );
  const gateProblems = unit.problems.filter((p) => p.role === "gate");

  return (
    <main className="min-h-0 flex-1 overflow-auto px-7 pb-16 pt-6">
      <div className="mx-auto w-full max-w-[760px]">
        <Link
          href="/learn"
          className="rise inline-flex items-center gap-1.5 text-[12.5px] font-medium text-muted-foreground transition-colors hover:text-foreground"
          style={{ "--rise-i": 0 } as React.CSSProperties}
        >
          <ArrowLeft className="size-[14px]" />
          The DSA Course
        </Link>

        <div
          className="rise mt-3 flex items-start justify-between gap-4"
          style={{ "--rise-i": 1 } as React.CSSProperties}
        >
          <div>
            <h1 className="text-[24px] font-semibold leading-tight tracking-tight">
              {unit.title}
            </h1>
            {unit.prereqs.length > 0 && (
              <p className="mt-1 text-[12.5px] text-muted-foreground">
                Builds on{" "}
                {unit.prereqs
                  .map((id) => units.get(id)?.title ?? prettifySlug(id))
                  .join(", ")}
              </p>
            )}
          </div>
          <div className="mt-1.5 shrink-0">
            <UnitStatusBadge status={progress.status} />
          </div>
        </div>

        {/* Locked banner — the unit stays sealed until its prereqs are mastered. */}
        {locked && (
          <div
            className="rise mt-6 flex items-start gap-3 rounded-lg border border-medium/30 bg-medium/10 p-4"
            style={{ "--rise-i": 2 } as React.CSSProperties}
          >
            <Lock className="mt-0.5 size-[16px] shrink-0 text-medium" />
            <div className="text-[13px]">
              <p className="font-semibold">This unit is locked.</p>
              <p className="mt-1 text-muted-foreground">
                Pass the mastery gate for{" "}
                <span className="font-medium text-foreground">
                  {prereqTitles.join(" and ")}
                </span>{" "}
                first. The course is a single chain — earlier patterns are the
                foundation for this one.
              </p>
              {progress.blockedBy.map((id) => (
                <Link
                  key={id}
                  href={`/learn?unit=${id}`}
                  className="mt-2 inline-flex items-center gap-1 text-[12.5px] font-medium text-primary"
                >
                  Go to {units.get(id)?.title ?? prettifySlug(id)}
                  <ArrowRight className="size-[13px]" />
                </Link>
              ))}
            </div>
          </div>
        )}

        {/* Lessons in order. */}
        <section
          className="rise mt-7"
          style={{ "--rise-i": 3 } as React.CSSProperties}
        >
          <div className="flex items-center gap-2">
            <BookOpen className="size-[15px] text-primary" />
            <span className="microlabel text-foreground">Lessons</span>
            <span className="font-mono text-[11px] text-muted-foreground">
              {progress.lessonsComplete}/{progress.lessonsTotal} complete
            </span>
          </div>
          {unit.lessons.length > 0 ? (
            <ul className="mt-3 flex flex-col gap-2">
              {unit.lessons.map((lid) => (
                <LessonRow
                  key={lid}
                  lessonId={lid}
                  status={lessonStatus.get(lid) ?? "not-started"}
                  locked={locked}
                />
              ))}
            </ul>
          ) : (
            <p className="mt-3 rounded-lg border border-dashed px-4 py-3 text-[12.5px] text-muted-foreground">
              Lessons for this unit arrive in a later phase — you can still take
              its mastery gate below.
            </p>
          )}
        </section>

        {/* Mastery gate. */}
        <GateSection
          unitId={unit.id}
          progress={progress}
          locked={locked}
          gateProblems={gateProblems.map((p) => ({
            slug: p.slug,
            novel: p.novel,
            label: problemLabel(p.slug),
            solved: progress.gate.solvedSlugs.includes(p.slug),
          }))}
          nextTitles={
            // Units this one unblocks, for the "unlocks …" line.
            [...units.values()]
              .filter((u) => u.prereqs.includes(unit.id))
              .map((u) => u.title)
          }
        />
      </div>
    </main>
  );
}

function LessonRow({
  lessonId,
  status,
  locked,
}: {
  lessonId: string;
  status: LessonStatus;
  locked: boolean;
}) {
  const [lesson, setLesson] = useState<Lesson | null>(null);
  useEffect(() => {
    let cancelled = false;
    getLesson(lessonId).then((l) => !cancelled && setLesson(l));
    return () => {
      cancelled = true;
    };
  }, [lessonId]);

  const label = lesson?.subpattern ?? prettifySlug(lessonId);
  const inner = (
    <>
      <BookOpen
        className={cn(
          "size-[15px] shrink-0",
          locked ? "text-muted-foreground" : "text-primary"
        )}
      />
      <span className="min-w-0 flex-1 truncate text-[13.5px] font-medium">
        {label}
      </span>
      {locked ? (
        <Lock className="size-[13px] text-muted-foreground" />
      ) : (
        <>
          <StatusPill status={status} />
          <ArrowRight className="size-[14px] text-muted-foreground" />
        </>
      )}
    </>
  );

  if (locked) {
    return (
      <li className="flex cursor-not-allowed items-center gap-3 rounded-lg border bg-card px-4 py-2.5 opacity-70">
        {inner}
      </li>
    );
  }
  return (
    <li>
      <Link
        href={`/learn?lesson=${lessonId}`}
        className="flex items-center gap-3 rounded-lg border bg-card px-4 py-2.5 transition-colors hover:bg-accent"
      >
        {inner}
      </Link>
    </li>
  );
}

function GateSection({
  unitId,
  progress,
  locked,
  gateProblems,
  nextTitles,
}: {
  unitId: string;
  progress: UnitProgress;
  locked: boolean;
  gateProblems: {
    slug: string;
    novel: boolean;
    label: string;
    solved: boolean;
  }[];
  nextTitles: string[];
}) {
  const { gate } = progress;
  const mastered = progress.status === "mastered";
  const target = gate.timerTargetMin;

  return (
    <section
      className="rise mt-8 rounded-xl border bg-surface-2 p-5"
      style={{ "--rise-i": 4 } as React.CSSProperties}
    >
      <div className="flex items-center gap-2">
        <ShieldCheck
          className={cn("size-[16px]", mastered ? "text-pass" : "text-medium")}
        />
        <span className="microlabel text-foreground">Mastery gate</span>
        {mastered && (
          <span className="inline-flex items-center gap-1 rounded-full bg-pass/10 px-2 py-[2px] text-[10.5px] font-semibold text-pass">
            <Trophy className="size-[11px]" />
            Passed
          </span>
        )}
      </div>

      <p className="mt-2.5 text-[13px] leading-relaxed text-muted-foreground">
        Fresh, unseen problems of this pattern. Solve{" "}
        <span className="font-semibold text-foreground">{gate.passCount}</span>
        {gate.requireNovel && (
          <>
            {" "}
            (including{" "}
            <span className="font-semibold text-foreground">≥1 novel</span>)
          </>
        )}{" "}
        <span className="font-semibold text-foreground">hint-free</span> and
        without peeking at the solution, aiming for under {target} min each.
        Passing unlocks{" "}
        {nextTitles.length > 0 ? nextTitles.join(", ") : "the next unit"}.
      </p>

      {/* progress bar */}
      <div className="mt-4 flex items-center gap-3">
        <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
          <div
            className={cn(
              "h-full rounded-full transition-all",
              gate.met ? "bg-pass" : "bg-medium"
            )}
            style={{
              width: `${Math.min(100, (gate.passedCount / Math.max(1, gate.passCount)) * 100)}%`,
            }}
          />
        </div>
        <span className="font-mono text-[11.5px] text-muted-foreground">
          {gate.passedCount}/{gate.passCount} cleared
        </span>
      </div>
      {gate.requireNovel && (
        <p className="mt-1.5 flex items-center gap-1.5 text-[11.5px]">
          <Sparkles
            className={cn(
              "size-[12px]",
              gate.passedNovel >= 1 ? "text-pass" : "text-muted-foreground"
            )}
          />
          <span
            className={cn(
              gate.passedNovel >= 1
                ? "font-medium text-pass"
                : "text-muted-foreground"
            )}
          >
            {gate.passedNovel >= 1
              ? "Novel requirement met"
              : "Needs at least one novel problem"}
          </span>
        </p>
      )}

      {/* gate problems */}
      <ul className="mt-4 flex flex-col gap-2">
        {gateProblems.map((gp) => (
          <li
            key={gp.slug}
            className="flex items-center gap-3 rounded-lg border bg-card px-4 py-2.5"
          >
            {gp.solved ? (
              <Check className="size-[15px] shrink-0 text-pass" />
            ) : (
              <Target className="size-[15px] shrink-0 text-medium" />
            )}
            <span className="min-w-0 flex-1 truncate text-[13.5px] font-medium">
              {gp.label}
            </span>
            {gp.novel && (
              <span className="inline-flex items-center gap-1 rounded-full border border-primary/30 bg-primary/5 px-2 py-[1px] text-[10px] font-semibold text-primary">
                <Sparkles className="size-[10px]" />
                Novel
              </span>
            )}
            {locked ? (
              <Lock className="size-[13px] text-muted-foreground" />
            ) : gp.solved ? (
              <span className="text-[11.5px] font-semibold text-pass">
                Cleared
              </span>
            ) : (
              <Link
                href={`/problem?id=${gp.slug}&gate=${unitId}&target=${target}`}
                className="inline-flex shrink-0 items-center gap-1.5 rounded-md bg-medium px-3 py-1.5 text-[12px] font-semibold text-white transition-[filter] hover:brightness-110"
              >
                Start gate
                <ArrowRight className="size-[13px] stroke-[2.2]" />
              </Link>
            )}
          </li>
        ))}
      </ul>

      {locked && (
        <p className="mt-3 text-[11.5px] text-muted-foreground">
          Unlock this unit to take its gate.
        </p>
      )}
    </section>
  );
}

/* --------------------------------------------------------------------- */
/* Single lesson                                                         */
/* --------------------------------------------------------------------- */

function LessonView({ lessonId }: { lessonId: string }) {
  const router = useRouter();
  const [lesson, setLesson] = useState<Lesson | null>(null);
  const [unitTitle, setUnitTitle] = useState<string>("");
  const [unitId, setUnitId] = useState<string>("");
  const [status, setStatus] = useState<LessonStatus>("not-started");
  const [problems, setProblems] = useState<Map<string, ProblemSummary>>(
    new Map()
  );
  const [pool, setPool] = useState<Quiz | null>(null);
  /** Unit id → title, for resolving a pattern-picker's revealed pattern. */
  const [unitTitles, setUnitTitles] = useState<Map<string, string>>(new Map());
  /** Patterns the learner has already met (this unit's prereqs + recap units) —
   *  what the start-of-lesson warm-up retrieves. */
  const [earlierUnitIds, setEarlierUnitIds] = useState<Set<string>>(new Set());
  const [notFound, setNotFound] = useState(false);
  const recordedFor = useRef<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const [l, prog, summaries, patternPool, curriculum] = await Promise.all([
        getLesson(lessonId),
        getLessonProgress(),
        listProblems(),
        getPatternPool(),
        getCurriculum(),
      ]);
      if (cancelled) return;
      if (!l) {
        setNotFound(true);
        return;
      }
      setLesson(l);
      setUnitId(l.unit);
      setProblems(new Map(summaries.map((s) => [s.id, s])));
      setPool(patternPool);
      setStatus(prog.find((p) => p.lessonId === lessonId)?.status ?? "not-started");

      // All unit titles (for the pattern reveal) + this lesson's unit prereqs.
      const allIds = curriculum.stages.flatMap((s) => s.units);
      const unitList = await Promise.all(allIds.map((id) => getUnitSafe(id)));
      if (cancelled) return;
      const titles = new Map<string, string>();
      for (const u of unitList) if (u) titles.set(u.id, u.title);
      setUnitTitles(titles);
      const unit = unitList.find((u) => u?.id === l.unit) ?? null;
      setUnitTitle(unit?.title ?? l.unit);

      // Earlier patterns = this unit's prereqs + the units of any recap lessons.
      const earlier = new Set<string>(unit?.prereqs ?? []);
      const recapLessons = await Promise.all(l.recap.map((id) => getLesson(id)));
      if (cancelled) return;
      for (const rl of recapLessons) if (rl) earlier.add(rl.unit);
      setEarlierUnitIds(earlier);

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
      router.push(unitId ? `/learn?unit=${unitId}` : "/learn");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not save progress");
    }
  }, [lessonId, router, unitId]);

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
  const patternLabel = (id: string) => unitTitles.get(id) ?? prettifySlug(id);

  // Split the lesson quiz for spaced placement (LESSON_COURSE_DESIGN.md §13.3):
  // concept-check/complexity land mid-lesson, pattern-picker at the end.
  const conceptItems = lesson.quiz.items.filter(
    (it) => it.type === "concept-check" || it.type === "complexity"
  );
  const lessonPickers = lesson.quiz.items.filter(
    (it) => it.type === "pattern-picker"
  );
  const poolItems = pool?.items ?? [];
  // Warm-up (start): retrieve an *earlier* pattern this lesson builds on.
  const warmupItems = poolItems.filter(
    (it) => it.correct_pattern && earlierUnitIds.has(it.correct_pattern)
  );
  // Interleaved recognition (end): unlabeled drills for this unit's pattern.
  const interleavedItems = poolItems.filter(
    (it) => it.correct_pattern === lesson.unit
  );

  return (
    <main className="min-h-0 flex-1 overflow-auto px-7 pb-16 pt-6">
      <div className="mx-auto w-full max-w-[760px]">
        <Link
          href={unitId ? `/learn?unit=${unitId}` : "/learn"}
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

        {/* Warm-up retrieval (start) — spaced recall of an earlier pattern this
            lesson builds on. Empty for the course's very first lesson. */}
        {warmupItems.length > 0 && (
          <QuizSection
            icon={Brain}
            title="Warm-up retrieval"
            hint="recall an earlier pattern first"
            riseIndex={2}
          >
            <p className="mb-3 text-[12.5px] text-muted-foreground">
              Before the new idea, retrieve a pattern you already know — spacing
              is what makes it stick. No labels: read the prompt and name it.
            </p>
            <QuizRunner
              source={PATTERN_POOL_SOURCE}
              items={warmupItems}
              patternLabel={patternLabel}
            />
          </QuizSection>
        )}

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

        {/* Prediction diagram — the pattern in motion, with a "what happens
            next?" pause. Taught during the lesson, not tacked on the end. */}
        <section
          className="rise mt-7"
          style={{ "--rise-i": 4 } as React.CSSProperties}
        >
          <div className="mb-3 flex items-center gap-2">
            <Film className="size-[15px] text-primary" />
            <span className="microlabel text-foreground">See it run</span>
            <span className="font-mono text-[11px] text-muted-foreground">
              predict, then reveal
            </span>
          </div>
          <DiagramPlayer diagram={lesson.diagram} />
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

        {/* Concept check (mid-lesson) — low-stakes retrieval on the just-taught
            idea. Never a gate. */}
        {conceptItems.length > 0 && (
          <QuizSection
            icon={ListChecks}
            title="Concept check"
            hint="retrieval, not a gate"
            riseIndex={5}
          >
            <QuizRunner
              source={lessonId}
              items={conceptItems}
              patternLabel={patternLabel}
            />
          </QuizSection>
        )}

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

        {/* Pattern-picker drills (end) — the moat: name the pattern from an
            unlabeled prompt, then read the trigger. */}
        {lessonPickers.length > 0 && (
          <QuizSection
            icon={Puzzle}
            title="Pattern-picker"
            hint="which pattern, and why?"
            riseIndex={7}
          >
            <p className="mb-3 text-[12.5px] text-muted-foreground">
              No pattern labels — decide which technique the prompt calls for.
              This is the skill that transfers to unseen problems.
            </p>
            <QuizRunner
              source={lessonId}
              items={lessonPickers}
              patternLabel={patternLabel}
            />
          </QuizSection>
        )}

        {/* Interleaved recognition — extra unlabeled drills for this pattern,
            drawn from the cross-unit pool. */}
        {interleavedItems.length > 0 && (
          <QuizSection
            icon={Sparkles}
            title="Interleaved recognition"
            hint="mixed prompts, no labels"
            riseIndex={8}
          >
            <QuizRunner
              source={PATTERN_POOL_SOURCE}
              items={interleavedItems}
              patternLabel={patternLabel}
            />
          </QuizSection>
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

/** A titled lesson section that hosts a quiz runner — matches the other
 *  lesson section headers (icon + microlabel + mono hint). */
function QuizSection({
  icon: Icon,
  title,
  hint,
  riseIndex,
  children,
}: {
  icon: typeof Check;
  title: string;
  hint?: string;
  riseIndex: number;
  children: React.ReactNode;
}) {
  return (
    <section
      className="rise mt-7"
      style={{ "--rise-i": riseIndex } as React.CSSProperties}
    >
      <div className="mb-3 flex items-center gap-2">
        <Icon className="size-[15px] text-primary" />
        <span className="microlabel text-foreground">{title}</span>
        {hint && (
          <span className="font-mono text-[11px] text-muted-foreground">
            {hint}
          </span>
        )}
      </div>
      {children}
    </section>
  );
}

/** getUnit that never throws — a missing unit just yields null. */
async function getUnitSafe(id: string): Promise<Unit | null> {
  try {
    return await getUnit(id);
  } catch {
    return null;
  }
}

/* --------------------------------------------------------------------- */
/* Stage-7 mixed capstone (unlabeled pool)                               */
/* --------------------------------------------------------------------- */

function CapstonePane() {
  const [capstone, setCapstone] = useState<CapstoneView | null | undefined>(
    undefined
  );

  useEffect(() => {
    let cancelled = false;
    getCapstone().then((c) => {
      if (!cancelled) setCapstone(c);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  if (capstone === undefined) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Spinner className="size-6" />
      </div>
    );
  }
  if (capstone === null) {
    return (
      <main className="min-h-0 flex-1 overflow-auto px-7 pb-10 pt-6">
        <div className="mx-auto w-full max-w-[820px]">
          <BackToCourse />
          <p className="mt-6 text-[13px] text-muted-foreground">
            This course has no capstone yet.
          </p>
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-0 flex-1 overflow-auto px-7 pb-10 pt-6">
      <div className="mx-auto w-full max-w-[820px]">
        <BackToCourse />
        <div className="mt-4 flex items-start gap-3">
          <div className="mt-0.5 flex size-9 items-center justify-center rounded-lg bg-gradient-to-b from-amber-500 to-amber-600 text-white shadow-sm">
            <Trophy className="size-5" />
          </div>
          <div>
            <h1 className="text-[21px] font-semibold tracking-tight">
              {capstone.title}
            </h1>
            <p className="mt-0.5 max-w-[62ch] text-[13px] text-muted-foreground">
              These problems carry <strong>no pattern label</strong>. That&apos;s
              the whole test: read the problem cold and decide which of your
              climbed patterns it needs — exactly like an unseen interview
              question. Hints and solutions are off; a peeked solve doesn&apos;t
              count. Clear <strong>{capstone.passCount}</strong> to pass.
            </p>
          </div>
        </div>

        <div className="mt-4 flex items-center gap-3 rounded-lg border bg-card px-4 py-2.5 text-[12.5px]">
          <span className="microlabel">Progress</span>
          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
            <div
              className="h-full rounded-full bg-pass transition-all"
              style={{
                width: `${Math.min(100, (capstone.passedCount / capstone.passCount) * 100)}%`,
              }}
            />
          </div>
          <span className="font-mono text-xs text-muted-foreground">
            {capstone.passedCount}/{capstone.passCount} cleared
          </span>
          <span className="text-muted-foreground">
            aim &lt; {capstone.timerTargetMin}m each
          </span>
        </div>

        {capstone.met && (
          <div className="mt-4 flex items-center gap-2 rounded-lg border border-pass/40 bg-pass/10 px-4 py-3 text-[13px] text-pass">
            <Trophy className="size-4" />
            You cleared the capstone — you can recognize and solve unfamiliar
            problems on your own.
          </div>
        )}

        <div className="mt-6 grid grid-cols-1 gap-2.5 sm:grid-cols-2">
          {capstone.problems.map((p, i) => (
            <Link
              key={p.problemId}
              href={`/problem?id=${p.problemId}&capstone=1&target=${capstone.timerTargetMin}`}
              className={cn(
                "group flex items-center gap-3 rounded-lg border bg-card px-4 py-3 transition-colors hover:bg-accent",
                p.solved && "border-pass/40"
              )}
            >
              <div
                className={cn(
                  "flex size-7 shrink-0 items-center justify-center rounded-md text-[12px] font-semibold",
                  p.solved
                    ? "bg-pass/15 text-pass"
                    : "bg-muted text-muted-foreground"
                )}
              >
                {p.solved ? <Check className="size-4" /> : i + 1}
              </div>
              <span className="min-w-0 flex-1 text-[13.5px] font-medium">
                Problem {i + 1}
                <span className="ml-2 text-[11px] font-normal text-muted-foreground">
                  unlabeled
                </span>
              </span>
              <ArrowRight className="size-[15px] shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
            </Link>
          ))}
        </div>
      </div>
    </main>
  );
}

/* --------------------------------------------------------------------- */
/* Diagnostic placement                                                  */
/* --------------------------------------------------------------------- */

function PlacementView() {
  const router = useRouter();
  const [probe, setProbe] = useState<PlacementProbe | null>(null);
  const [choices, setChoices] = useState<Record<string, string>>({});
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    let cancelled = false;
    getPlacement().then((p) => {
      if (!cancelled) setProbe(p);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const submit = useCallback(async () => {
    if (!probe) return;
    setSubmitting(true);
    try {
      const answers: QuizAnswer[] = probe.items.map((i) => ({
        itemId: i.id,
        selected: choices[i.id] ?? "",
      }));
      const out = await applyPlacement(answers);
      if (out.placed.length === 0) {
        toast("No units placed out — starting from the top. That's the honest floor.");
      } else {
        toast.success(
          `Placed out of ${out.placed.length} unit${out.placed.length === 1 ? "" : "s"}. Your frontier is ready.`
        );
      }
      router.push("/learn");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Placement failed");
    } finally {
      setSubmitting(false);
    }
  }, [probe, choices, router]);

  if (!probe) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Spinner className="size-6" />
      </div>
    );
  }

  return (
    <main className="min-h-0 flex-1 overflow-auto px-7 pb-10 pt-6">
      <div className="mx-auto w-full max-w-[720px]">
        <BackToCourse />
        <div className="mt-4 flex items-start gap-3">
          <div className="mt-0.5 flex size-9 items-center justify-center rounded-lg bg-gradient-to-b from-primary to-primary/80 text-primary-foreground shadow-sm">
            <Target className="size-5" />
          </div>
          <div>
            <h1 className="text-[21px] font-semibold tracking-tight">
              Placement test
            </h1>
            <p className="mt-0.5 max-w-[60ch] text-[13px] text-muted-foreground">
              Already know some of this? Name the pattern each unlabeled prompt
              needs. Get every prompt for a unit right (and its prerequisites)
              and we&apos;ll place you past it — starting you at your frontier
              instead of unit one. No penalty for guessing wrong; you just
              start earlier there.
            </p>
          </div>
        </div>

        {probe.items.length === 0 ? (
          <p className="mt-6 text-[13px] text-muted-foreground">
            No placement prompts are available yet.
          </p>
        ) : (
          <>
            <div className="mt-6 flex flex-col gap-4">
              {probe.items.map((item, qi) => (
                <div key={item.id} className="rounded-lg border bg-card p-4">
                  <div className="text-[13.5px]">
                    <span className="mr-2 font-mono text-[11px] text-muted-foreground">
                      {qi + 1}.
                    </span>
                    <Markdown>{item.prompt_md}</Markdown>
                  </div>
                  <div className="mt-3 flex flex-col gap-1.5">
                    {item.options.map((opt) => (
                      <label
                        key={opt}
                        className={cn(
                          "flex cursor-pointer items-center gap-2 rounded-md border px-3 py-2 text-[12.5px] transition-colors hover:bg-accent",
                          choices[item.id] === opt &&
                            "border-primary bg-primary/5"
                        )}
                      >
                        <input
                          type="radio"
                          name={item.id}
                          className="accent-primary"
                          checked={choices[item.id] === opt}
                          onChange={() =>
                            setChoices((c) => ({ ...c, [item.id]: opt }))
                          }
                        />
                        {opt}
                      </label>
                    ))}
                  </div>
                </div>
              ))}
            </div>
            <button
              type="button"
              onClick={submit}
              disabled={submitting}
              className="mt-5 inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-[13px] font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:opacity-60"
            >
              {submitting ? <Spinner className="size-4" /> : <Target className="size-4" />}
              Place me
            </button>
          </>
        )}
      </div>
    </main>
  );
}

function BackToCourse() {
  return (
    <Link
      href="/learn"
      className="inline-flex items-center gap-1.5 text-[12.5px] text-muted-foreground transition-colors hover:text-foreground"
    >
      <ArrowLeft className="size-3.5" /> The DSA Course
    </Link>
  );
}

function LearnRouter() {
  const searchParams = useSearchParams();
  const lessonId = searchParams.get("lesson");
  const unitId = searchParams.get("unit");
  if (searchParams.get("capstone")) return <CapstonePane />;
  if (searchParams.get("placement")) return <PlacementView />;
  if (lessonId) return <LessonView lessonId={lessonId} />;
  if (unitId) return <UnitView unitId={unitId} />;
  return <CourseView />;
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
