"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import {
  ArrowRight,
  CircleCheckBig,
  Eye,
  Flame,
  Plus,
  Star,
  Target,
  TrendingUp,
} from "lucide-react";
import { AppShell } from "@/src/components/anvil/app-shell";
import { Spinner } from "@/src/components/anvil/spinner";
import { ActivityHeatmap } from "@/src/components/dashboard/activity-heatmap";
import { ProgressChart } from "@/src/components/dashboard/progress-chart";
import { getDashboard } from "@/src/lib/api";
import { cn } from "@/src/lib/utils";
import type { DashboardData, Difficulty, PatternStat } from "@/src/lib/types";

const DIFFICULTY_TEXT: Record<Difficulty, string> = {
  Easy: "text-easy",
  Medium: "text-medium",
  Hard: "text-hard",
};

/** Section card with the shared chrome: hairline border, panel surface. */
function Panel({
  className,
  riseIndex,
  children,
}: {
  className?: string;
  riseIndex?: number;
  children: React.ReactNode;
}) {
  return (
    <div
      className={cn("rise rounded-lg border bg-card", className)}
      style={
        riseIndex !== undefined
          ? ({ "--rise-i": riseIndex } as React.CSSProperties)
          : undefined
      }
    >
      {children}
    </div>
  );
}

function StatTile({
  label,
  icon,
  riseIndex,
  children,
  sub,
}: {
  label: string;
  icon: React.ReactNode;
  riseIndex: number;
  children: React.ReactNode;
  sub?: React.ReactNode;
}) {
  return (
    <Panel className="p-4" riseIndex={riseIndex}>
      <div className="flex items-center justify-between">
        <span className="microlabel">{label}</span>
        {icon}
      </div>
      <div className="mt-2.5">{children}</div>
      {sub && <div className="mt-3 text-xs text-muted-foreground">{sub}</div>}
    </Panel>
  );
}

function BigNumber({
  value,
  className,
  suffix,
}: {
  value: React.ReactNode;
  className?: string;
  suffix?: React.ReactNode;
}) {
  return (
    <div className="flex items-baseline gap-1.5">
      <span
        className={cn(
          "font-mono text-[27px] font-bold leading-none tracking-tight",
          className
        )}
      >
        {value}
      </span>
      {suffix}
    </div>
  );
}

function PatternRows({
  rows,
  barClass,
}: {
  rows: PatternStat[];
  barClass: string;
}) {
  return (
    <div className="mt-3.5 flex flex-col gap-3.5">
      {rows.map((row) => (
        <div key={row.pattern}>
          <div className="flex items-center justify-between text-[13px]">
            <span className="font-medium">{row.pattern}</span>
            <span className="font-mono text-xs text-muted-foreground">
              {row.solved} / {row.total}
            </span>
          </div>
          <div className="mt-[7px] h-1 overflow-hidden rounded-full bg-muted">
            <div
              className={cn("h-full rounded-full", barClass)}
              style={{
                width: `${Math.max(
                  row.total ? (row.solved / row.total) * 100 : 0,
                  2
                )}%`,
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

export default function DashboardPage() {
  const [data, setData] = useState<DashboardData | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = () =>
      getDashboard().then((d) => {
        if (!cancelled) setData(d);
      });
    load();
    // Refetch whenever the window/tab regains focus so the dashboard always
    // reflects the latest solves the runner recorded (e.g. after solving a
    // problem and returning) — no stale stats within a session.
    const refresh = () => {
      if (document.visibilityState === "visible") load();
    };
    window.addEventListener("focus", refresh);
    document.addEventListener("visibilitychange", refresh);
    return () => {
      cancelled = true;
      window.removeEventListener("focus", refresh);
      document.removeEventListener("visibilitychange", refresh);
    };
  }, []);

  if (!data) {
    return (
      <AppShell>
        <div className="flex flex-1 items-center justify-center">
          <Spinner className="size-6" />
        </div>
      </AppShell>
    );
  }

  const { progress } = data;
  const isEmpty = progress.attempted === 0 && progress.solved === 0;
  const solvedPct = progress.total
    ? (progress.solved / progress.total) * 100
    : 0;
  const totalSolves = data.activity.reduce((acc, d) => acc + d.count, 0);

  return (
    <AppShell
      status={
        <span>
          {progress.solved}/{progress.total} solved
          {progress.streakDays > 0 && ` · ${progress.streakDays}d streak`}
        </span>
      }
    >
      <main className="min-h-0 flex-1 overflow-auto px-7 pb-8 pt-6">
        <div className="mx-auto w-full max-w-[1060px]">
        <div
          className="rise flex items-start justify-between gap-4"
          style={{ "--rise-i": 0 } as React.CSSProperties}
        >
          <div>
            <h1 className="text-[21px] font-semibold tracking-tight">
              {isEmpty ? "Welcome to Anvil" : "Welcome back."}
            </h1>
            <p className="mt-0.5 text-[13px] text-muted-foreground">
              {isEmpty
                ? "Solve your first problem to start tracking your progress."
                : "Here's where your practice stands today."}
            </p>
          </div>
          <Link
            href="/create"
            className="flex shrink-0 items-center gap-1.5 rounded-md bg-primary px-3.5 py-2 text-[13px] font-semibold text-primary-foreground shadow-sm transition-[filter,transform] hover:brightness-110 active:scale-[0.98]"
          >
            <Plus className="size-[15px] stroke-[2.4]" />
            New problem
          </Link>
        </div>

        {/* stat tiles */}
        <div className="mt-5 grid grid-cols-4 gap-3.5">
          <StatTile
            label="Solved"
            riseIndex={1}
            icon={<CircleCheckBig className="size-4 text-primary" />}
          >
            <BigNumber
              value={progress.solved}
              suffix={
                <span className="font-mono text-sm font-medium text-muted-foreground">
                  / {progress.total}
                </span>
              }
            />
            <div className="mt-3 h-1 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-primary"
                style={{ width: `${solvedPct}%` }}
              />
            </div>
          </StatTile>
          <StatTile
            label="Attempted"
            riseIndex={2}
            icon={<Eye className="size-4 text-chart-3" />}
            sub={
              isEmpty
                ? "No attempts yet"
                : `${progress.needsReview} flagged for review`
            }
          >
            <BigNumber value={progress.attempted} />
          </StatTile>
          <StatTile
            label="Streak"
            riseIndex={3}
            icon={<Flame className="size-4 text-medium" />}
            sub={
              isEmpty
                ? "Start today"
                : `Personal best · ${progress.bestStreakDays} days`
            }
          >
            <BigNumber
              value={progress.streakDays}
              className="text-medium"
              suffix={
                <span className="text-sm font-medium text-muted-foreground">
                  days
                </span>
              }
            />
          </StatTile>
          <StatTile
            label="Mastered"
            riseIndex={4}
            icon={<Star className="size-4 text-easy" />}
            sub={isEmpty ? "—" : `of ${progress.solved} solved, fully mastered`}
          >
            <BigNumber value={progress.mastered} />
          </StatTile>
        </div>

        {/* charts row */}
        <div className="mt-3.5 grid grid-cols-[1.25fr_1fr] gap-3.5">
          <Panel className="px-[18px] py-4" riseIndex={5}>
            <div className="flex items-center justify-between">
              <span className="microlabel">Activity</span>
              <span className="font-mono text-[11px] text-muted-foreground">
                {isEmpty
                  ? "last 26 weeks"
                  : `${totalSolves} solves · last 26 weeks`}
              </span>
            </div>
            <ActivityHeatmap
              activity={
                isEmpty
                  ? data.activity.map((d) => ({ ...d, count: 0 }))
                  : data.activity
              }
            />
          </Panel>
          <Panel className="flex flex-col px-[18px] py-4" riseIndex={6}>
            <div className="flex items-center justify-between">
              <span className="microlabel">Progress over time</span>
              <span className="font-mono text-[11px] text-muted-foreground">
                cumulative solved
              </span>
            </div>
            <div className="relative mt-2.5 min-h-[150px] flex-1">
              {isEmpty ? (
                <div className="flex h-40 items-center justify-center rounded-md border border-dashed text-[13px] text-muted-foreground">
                  No activity yet — your progress will chart here.
                </div>
              ) : (
                <ProgressChart series={data.cumulative} />
              )}
            </div>
            <div className="mt-1 flex justify-between font-mono text-[10.5px] text-muted-foreground">
              <span>{isEmpty ? "" : data.axisLabels[0]}</span>
              <span>{isEmpty ? "" : data.axisLabels[1]}</span>
              <span>Now</span>
            </div>
          </Panel>
        </div>

        {/* focus / doing well */}
        <div className="mt-3.5 grid grid-cols-2 gap-3.5">
          <Panel className="px-[18px] py-4" riseIndex={7}>
            <div className="flex items-center gap-2">
              <Target className="size-[15px] text-medium" />
              <span className="microlabel text-foreground">Focus here</span>
              <span className="font-mono text-[11px] text-muted-foreground">
                weakest patterns
              </span>
            </div>
            {isEmpty ? (
              <p className="mt-4 text-[13px] leading-relaxed text-muted-foreground">
                Solve a few problems and Anvil will surface the patterns that
                need the most work.
              </p>
            ) : (
              <PatternRows rows={data.focus} barClass="bg-medium" />
            )}
          </Panel>
          <Panel className="px-[18px] py-4" riseIndex={8}>
            <div className="flex items-center gap-2">
              <TrendingUp className="size-[15px] text-primary" />
              <span className="microlabel text-foreground">Doing well</span>
              <span className="font-mono text-[11px] text-muted-foreground">
                strong patterns
              </span>
            </div>
            {isEmpty ? (
              <p className="mt-4 text-[13px] leading-relaxed text-muted-foreground">
                Your strongest patterns will appear here as you solve more.
              </p>
            ) : (
              <PatternRows rows={data.strong} barClass="bg-primary" />
            )}
          </Panel>
        </div>

        {/* continue / first-run CTA */}
        {!isEmpty && data.continueProblem ? (
          <Panel
            className="mt-3.5 flex items-center gap-4 px-[18px] py-[15px]"
            riseIndex={9}
          >
            <div className="min-w-0 flex-1">
              <div className="microlabel">Continue where you left off</div>
              <div className="mt-[7px] flex items-center gap-3">
                <span className="text-[15px] font-semibold">
                  {data.continueProblem.number}. {data.continueProblem.title}
                </span>
                <span
                  className={cn(
                    "inline-flex items-center gap-1.5 text-xs font-semibold",
                    DIFFICULTY_TEXT[data.continueProblem.difficulty]
                  )}
                >
                  <span className="size-1.5 rounded-full bg-current" />
                  {data.continueProblem.difficulty}
                </span>
                <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
                  <span className="size-1.5 rounded-full bg-medium" />
                  In progress
                </span>
              </div>
            </div>
            <Link
              href={`/problem?id=${data.continueProblem.id}`}
              className="flex items-center gap-1.5 rounded-md bg-primary px-4 py-[9px] text-[13px] font-semibold text-primary-foreground transition-[filter,transform] hover:brightness-110 active:scale-[0.98]"
            >
              Resume
              <ArrowRight className="size-[15px] stroke-[2.2]" />
            </Link>
          </Panel>
        ) : (
          <Panel
            className="mt-3.5 flex items-center gap-[18px] bg-surface-2 p-[22px]"
            riseIndex={9}
          >
            <div className="flex-1">
              <div className="text-base font-semibold">
                Solve your first problem
              </div>
              <p className="mt-1 text-[13px] leading-relaxed text-muted-foreground">
                Pick a pattern below or browse the built-in library to get
                started. Everything runs locally — no account needed.
              </p>
            </div>
            <Link
              href="/problems"
              className="flex items-center gap-1.5 rounded-md bg-primary px-[18px] py-2.5 text-[13.5px] font-semibold text-primary-foreground transition-[filter,transform] hover:brightness-110 active:scale-[0.98]"
            >
              Browse library
              <ArrowRight className="size-[15px] stroke-[2.2]" />
            </Link>
          </Panel>
        )}

        {/* pattern chips */}
        <div
          className="rise mt-6"
          style={{ "--rise-i": 10 } as React.CSSProperties}
        >
          <div className="microlabel">Patterns</div>
          <div className="mt-[11px] flex flex-wrap gap-2">
            {data.patternStats.map((stat, i) => (
              <Link
                key={stat.pattern}
                href={`/problems?pattern=${encodeURIComponent(stat.pattern)}`}
                className={cn(
                  "inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-xs transition-colors",
                  i === 0 && !isEmpty
                    ? "border border-primary/30 bg-primary/10 font-semibold text-primary dark:bg-primary/15"
                    : "border bg-card font-medium hover:bg-accent"
                )}
              >
                {stat.pattern}
                <span className="font-mono text-[10.5px] text-muted-foreground">
                  {stat.solved}/{stat.total}
                </span>
              </Link>
            ))}
          </div>
        </div>
        </div>
      </main>
    </AppShell>
  );
}
