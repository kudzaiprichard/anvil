"use client";

import {
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
  type Ref,
} from "react";
import { Pause, Play, RotateCcw, Timer } from "lucide-react";
import { loadSolveTime, saveSolveTime } from "@/src/lib/solve-times";
import { cn } from "@/src/lib/utils";

function format(totalSeconds: number): string {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  const mm = String(m).padStart(2, "0");
  const ss = String(s).padStart(2, "0");
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

export interface PracticeTimerHandle {
  /** The run just passed every test: stop the clock and record the solve
   *  time for this problem. Returns the formatted time, or null when the
   *  clock wasn't running (already solved, paused, or never started). */
  solvedNow: () => string | null;
}

/**
 * Per-problem practice stopwatch (workspace toolbar). Interview practice is
 * time-boxed by nature: the clock starts when a problem opens (or by hand,
 * per the auto-start setting), freezes the moment a run passes the full
 * suite, and records that time as the problem's latest solve. Mount with
 * `key={problemId}` — switching problems remounts it, which resets the
 * clock.
 */
export function PracticeTimer({
  problemId,
  autoStart,
  targetMinutes,
  ref,
}: {
  problemId: string;
  autoStart: boolean;
  /** Soft per-problem target (mastery gate, COURSE_BLUEPRINT.md §6). Shows an
   *  "aim < Nm" goal and turns amber once passed — it never stops the clock or
   *  fails the attempt. */
  targetMinutes?: number;
  ref?: Ref<PracticeTimerHandle>;
}) {
  const [seconds, setSeconds] = useState(0);
  const [running, setRunning] = useState(autoStart);
  const [solved, setSolved] = useState(false);
  const [lastSolve, setLastSolve] = useState<number | null>(() =>
    loadSolveTime(problemId)
  );
  const overTarget =
    targetMinutes !== undefined && !solved && seconds > targetMinutes * 60;

  // Mirrors for the imperative handle (fresh values without re-binding).
  const secondsRef = useRef(seconds);
  const runningRef = useRef(running);
  useEffect(() => {
    secondsRef.current = seconds;
    runningRef.current = running;
  });

  useImperativeHandle(
    ref,
    () => ({
      solvedNow: () => {
        if (!runningRef.current || secondsRef.current === 0) return null;
        const s = secondsRef.current;
        setRunning(false);
        setSolved(true);
        setLastSolve(s);
        saveSolveTime(problemId, s);
        return format(s);
      },
    }),
    [problemId]
  );

  useEffect(() => {
    if (!running) return;
    const t = setInterval(() => setSeconds((s) => s + 1), 1000);
    return () => clearInterval(t);
  }, [running]);

  const ctlBtn =
    "flex size-[22px] items-center justify-center rounded-[5px] text-muted-foreground/70 transition-colors hover:bg-accent hover:text-foreground";

  return (
    <div
      title={
        lastSolve !== null
          ? `Last solve: ${format(lastSolve)}`
          : "Practice timer"
      }
      className={cn(
        "flex items-center gap-0.5 rounded-md border bg-editor py-[3px] pl-2.5 pr-1",
        solved && "border-pass/40"
      )}
    >
      <Timer
        className={cn(
          "mr-1.5 size-[13px]",
          solved
            ? "text-pass"
            : running
              ? "text-primary"
              : "text-muted-foreground/60"
        )}
      />
      <span
        className={cn(
          "min-w-[42px] font-mono text-[12px] font-medium tabular-nums",
          solved
            ? "text-pass"
            : overTarget
              ? "text-medium"
              : running
                ? "text-foreground"
                : "text-muted-foreground"
        )}
      >
        {format(seconds)}
      </span>
      {targetMinutes !== undefined && !solved && (
        <span
          className={cn(
            "mr-0.5 font-mono text-[10.5px] tabular-nums",
            overTarget ? "text-medium" : "text-muted-foreground/70"
          )}
        >
          / aim&nbsp;{targetMinutes}m
        </span>
      )}
      <button
        type="button"
        title={running ? "Pause timer" : "Start timer"}
        onClick={() => {
          setSolved(false);
          setRunning((r) => !r);
        }}
        className={ctlBtn}
      >
        {running ? (
          <Pause className="size-[11px] fill-current stroke-none" />
        ) : (
          <Play className="size-[11px] fill-current stroke-none" />
        )}
      </button>
      <button
        type="button"
        title="Reset timer"
        onClick={() => {
          setSeconds(0);
          setSolved(false);
          setRunning(true);
        }}
        className={ctlBtn}
      >
        <RotateCcw className="size-[11px]" />
      </button>
    </div>
  );
}
