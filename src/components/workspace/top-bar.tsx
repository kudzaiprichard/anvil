"use client";

import {
  ArrowUp,
  ChevronLeft,
  ChevronRight,
  List,
  Play,
  Shuffle,
} from "lucide-react";
import { Spinner } from "@/src/components/anvil/spinner";

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="rounded-[4px] bg-black/10 px-1 font-mono text-[10px] font-medium opacity-80 dark:bg-black/20">
      {children}
    </kbd>
  );
}

/**
 * Workspace toolbar (UI_SPEC §5.1): problem navigation on the left, Run /
 * Submit in the center. App-level chrome (logo, theme, settings) lives in
 * the shell rail.
 */
export function TopBar({
  running,
  onOpenList,
  onPrev,
  onNext,
  onShuffle,
  onRun,
  onSubmit,
}: {
  running: boolean;
  onOpenList: () => void;
  onPrev: () => void;
  onNext: () => void;
  onShuffle: () => void;
  onRun: () => void;
  onSubmit: () => void;
}) {
  const iconBtn =
    "flex size-[28px] items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground";

  return (
    <div className="flex h-[42px] shrink-0 items-center gap-2 border-b bg-card px-2.5">
      {/* left zone */}
      <div className="flex items-center gap-1">
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

      {/* right zone: Run / Submit */}
      <div className="flex items-center gap-1.5">
        <button
          type="button"
          onClick={onRun}
          disabled={running}
          className="flex items-center gap-2 rounded-md border border-primary/30 bg-primary/10 px-3 py-[5.5px] text-[12.5px] font-semibold text-primary transition-[filter,transform] hover:brightness-105 active:scale-[0.98] disabled:cursor-progress dark:border-primary/40 dark:bg-primary/15"
        >
          {running ? (
            <>
              <Spinner className="size-[13px] border-primary/40 border-t-primary" />
              Running…
            </>
          ) : (
            <>
              <Play className="size-[12px] fill-current stroke-none" />
              Run
              <Kbd>Ctrl+&apos;</Kbd>
            </>
          )}
        </button>
        <button
          type="button"
          onClick={onSubmit}
          disabled={running}
          className="flex items-center gap-2 rounded-md bg-primary px-3.5 py-[5.5px] text-[12.5px] font-semibold text-primary-foreground shadow-sm transition-[filter,transform] hover:brightness-110 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-45"
        >
          <ArrowUp className="size-[13px] stroke-[2.4]" />
          Submit
          <Kbd>Ctrl+↵</Kbd>
        </button>
      </div>
    </div>
  );
}
