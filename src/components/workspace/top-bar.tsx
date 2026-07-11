"use client";

import { ArrowLeft, ChevronLeft, ChevronRight, List, Play, Shuffle } from "lucide-react";
import Link from "next/link";
import { Spinner } from "@/src/components/anvil/spinner";

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="rounded-[4px] bg-black/10 px-1 font-mono text-[10px] font-medium opacity-80 dark:bg-black/20">
      {children}
    </kbd>
  );
}

/**
 * Workspace toolbar: problem navigation on the left, Run at the far right.
 * There is no separate Submit — the app is fully offline, so the one Run
 * action executes the full test suite (visible + hidden) and records the
 * attempt. App-level chrome (logo, theme, settings) lives in the shell rail.
 */
export function TopBar({
  running,
  onOpenList,
  onPrev,
  onNext,
  onShuffle,
  onRun,
  timer,
  back,
}: {
  running: boolean;
  onOpenList: () => void;
  onPrev: () => void;
  onNext: () => void;
  onShuffle: () => void;
  onRun: () => void;
  /** Optional practice-timer chip, rendered left of Run. */
  timer?: React.ReactNode;
  /** Where the far-left back button returns to (the lesson/unit/capstone the
   *  learner launched this problem from). Omit for a standalone workspace. */
  back?: { href: string; label: string };
}) {
  const iconBtn =
    "flex size-[28px] items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground";

  return (
    <div className="flex h-[42px] shrink-0 items-center gap-2 border-b bg-card px-2.5">
      {/* left zone */}
      <div className="flex items-center gap-1">
        {back && (
          <Link
            href={back.href}
            title={back.label}
            className="mr-0.5 flex items-center gap-1.5 rounded-md border bg-card px-2.5 py-[5px] text-[12.5px] font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          >
            <ArrowLeft className="size-[15px]" />
            <span className="max-[560px]:hidden">{back.label}</span>
          </Link>
        )}
        <button
          type="button"
          onClick={onOpenList}
          className="flex items-center gap-[7px] rounded-md border bg-card px-2.5 py-[5px] text-[12.5px] font-medium transition-colors hover:bg-accent"
        >
          <List className="size-[14px]" />
          Problem List
        </button>
        <div className="ml-0.5 flex items-center gap-0.5">
          <button type="button" onClick={onPrev} title="Previous (Ctrl+[)" className={iconBtn}>
            <ChevronLeft className="size-4" />
          </button>
          <button type="button" onClick={onNext} title="Next (Ctrl+])" className={iconBtn}>
            <ChevronRight className="size-4" />
          </button>
          <button type="button" onClick={onShuffle} title="Shuffle" className={iconBtn}>
            <Shuffle className="size-[14px]" />
          </button>
        </div>
      </div>

      <div className="flex-1" />

      {/* right zone: timer + Run (full suite) */}
      {timer}
      <button
        type="button"
        onClick={onRun}
        disabled={running}
        className="flex items-center gap-2 rounded-md bg-primary px-3.5 py-[5.5px] text-[12.5px] font-semibold text-primary-foreground shadow-sm transition-[filter,transform] hover:brightness-110 active:scale-[0.98] disabled:cursor-progress disabled:opacity-60"
      >
        {running ? (
          <>
            <Spinner className="size-[13px] border-primary-foreground/40 border-t-primary-foreground" />
            Running…
          </>
        ) : (
          <>
            <Play className="size-[12px] fill-current stroke-none" />
            Run
            <Kbd>Ctrl+↵</Kbd>
          </>
        )}
      </button>
    </div>
  );
}
