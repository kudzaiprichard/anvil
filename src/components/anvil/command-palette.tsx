"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import {
  Hammer,
  LayoutDashboard,
  LayoutGrid,
  Library,
  Search,
  Settings,
  SunMedium,
} from "lucide-react";
import { useTheme } from "next-themes";
import { toast } from "sonner";
import { DifficultyBadge } from "@/src/components/anvil/difficulty-badge";
import { StatusIndicator } from "@/src/components/anvil/status-indicator";
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from "@/src/components/shadcn/dialog";
import { listProblems } from "@/src/lib/api";
import { setEditorPrefs, WORKSPACE_LAYOUTS } from "@/src/lib/settings";
import type { ProblemSummary } from "@/src/lib/types";
import { cn } from "@/src/lib/utils";

type Item =
  | { kind: "action"; id: string; label: string; icon: React.ElementType; run: () => void }
  | { kind: "problem"; id: string; summary: ProblemSummary };

const EMPTY_PROBLEM_ROWS = 10;
const MAX_PROBLEM_ROWS = 50;

/**
 * Ctrl+K palette (owned by the shell): type-ahead jump to any problem by
 * number or title, plus the handful of app actions. Hand-rolled on the
 * dialog primitive so it stays in the forged-iron design language.
 */
export function CommandPalette({
  open,
  onOpenChange,
  onOpenSettings,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onOpenSettings: () => void;
}) {
  const router = useRouter();
  const { resolvedTheme, setTheme } = useTheme();
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const [summaries, setSummaries] = useState<ProblemSummary[] | null>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Load the catalog once, on first open.
  useEffect(() => {
    if (open && summaries === null) listProblems().then(setSummaries);
  }, [open, summaries]);

  // Fresh palette every time it opens (state adjusted during render — the
  // open flag can flip from either the dialog or the Ctrl+K toggle).
  const [prevOpen, setPrevOpen] = useState(open);
  if (open !== prevOpen) {
    setPrevOpen(open);
    if (open) {
      setQuery("");
      setActive(0);
    }
  }

  const actions = useMemo<Item[]>(() => {
    const nav = (href: string) => () => router.push(href);
    const base: Item[] = [
      { kind: "action", id: "nav-dashboard", label: "Go to Dashboard", icon: LayoutDashboard, run: nav("/") },
      { kind: "action", id: "nav-library", label: "Go to Library", icon: Library, run: nav("/problems") },
      { kind: "action", id: "nav-forge", label: "Forge a problem", icon: Hammer, run: nav("/create") },
      { kind: "action", id: "settings", label: "Open Settings", icon: Settings, run: onOpenSettings },
      {
        kind: "action",
        id: "theme",
        label: "Toggle light / dark theme",
        icon: SunMedium,
        run: () => setTheme(resolvedTheme === "dark" ? "light" : "dark"),
      },
    ];
    for (const layout of WORKSPACE_LAYOUTS) {
      base.push({
        kind: "action",
        id: `layout-${layout.id}`,
        label: `Workspace layout: ${layout.label}`,
        icon: LayoutGrid,
        run: () => {
          setEditorPrefs({ workspaceLayout: layout.id });
          toast(`Workspace layout: ${layout.label}`);
        },
      });
    }
    return base;
  }, [router, onOpenSettings, resolvedTheme, setTheme]);

  const { groups, flat } = useMemo(() => {
    const q = query.trim().toLowerCase();
    const matchedActions = q
      ? actions.filter((a) => a.kind === "action" && a.label.toLowerCase().includes(q))
      : actions;
    let matchedProblems: Item[] = [];
    if (summaries) {
      const pool = q
        ? summaries.filter(
            (s) =>
              String(s.number).startsWith(q) ||
              s.title.toLowerCase().includes(q)
          )
        : summaries.slice(0, EMPTY_PROBLEM_ROWS);
      matchedProblems = pool
        .slice(0, MAX_PROBLEM_ROWS)
        .map((summary) => ({ kind: "problem" as const, id: summary.id, summary }));
    }
    const groups: { title: string; items: Item[] }[] = [];
    if (matchedActions.length) groups.push({ title: "Actions", items: matchedActions });
    if (matchedProblems.length) groups.push({ title: "Problems", items: matchedProblems });
    return { groups, flat: groups.flatMap((g) => g.items) };
  }, [query, actions, summaries]);

  // Keep the active row visible while arrowing through the list.
  useEffect(() => {
    listRef.current
      ?.querySelector(`[data-idx="${active}"]`)
      ?.scrollIntoView({ block: "nearest" });
  }, [active]);

  const perform = (item: Item | undefined) => {
    if (!item) return;
    onOpenChange(false);
    if (item.kind === "action") item.run();
    else router.push(`/problem?id=${item.id}`);
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((i) => (flat.length ? (i + 1) % flat.length : 0));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((i) => (flat.length ? (i - 1 + flat.length) % flat.length : 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      perform(flat[active]);
    }
  };

  let idx = -1;
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        showCloseButton={false}
        aria-describedby={undefined}
        className="top-[14%] w-[560px] max-w-[560px] translate-y-0 gap-0 overflow-hidden rounded-xl p-0 sm:max-w-[560px]"
      >
        <DialogTitle className="sr-only">Command palette</DialogTitle>

        {/* query row */}
        <div className="flex items-center gap-2.5 border-b px-3.5">
          <Search className="size-4 shrink-0 text-muted-foreground" />
          <input
            autoFocus
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setActive(0);
            }}
            onKeyDown={onKeyDown}
            placeholder="Jump to a problem by number or name, or run a command…"
            className="h-[46px] w-full bg-transparent text-[13.5px] outline-none placeholder:text-muted-foreground"
          />
          <kbd className="shrink-0 rounded-md border bg-muted px-1.5 py-0.5 font-mono text-[10.5px] font-medium text-muted-foreground">
            esc
          </kbd>
        </div>

        {/* results */}
        <div ref={listRef} className="max-h-[380px] min-h-[120px] overflow-y-auto p-1.5">
          {flat.length === 0 ? (
            <div className="flex h-[120px] items-center justify-center text-[13px] text-muted-foreground">
              {summaries === null ? "Loading catalog…" : "No matches."}
            </div>
          ) : (
            groups.map((group) => (
              <div key={group.title}>
                <div className="microlabel px-2.5 pb-1 pt-2.5">{group.title}</div>
                {group.items.map((item) => {
                  idx += 1;
                  const i = idx;
                  const isActive = i === active;
                  return (
                    <button
                      key={`${item.kind}-${item.id}`}
                      type="button"
                      data-idx={i}
                      onClick={() => perform(item)}
                      onMouseMove={() => setActive(i)}
                      className={cn(
                        "flex w-full items-center gap-2.5 rounded-[9px] px-2.5 py-[7px] text-left",
                        isActive ? "bg-accent" : undefined
                      )}
                    >
                      {item.kind === "action" ? (
                        <>
                          <item.icon className="size-[15px] shrink-0 text-muted-foreground" />
                          <span className="min-w-0 flex-1 truncate text-[13px] font-medium">
                            {item.label}
                          </span>
                        </>
                      ) : (
                        <>
                          <span className="flex w-[18px] shrink-0 items-center justify-center">
                            <StatusIndicator status={item.summary.status} />
                          </span>
                          <span className="w-[38px] shrink-0 text-right font-mono text-xs text-muted-foreground">
                            {item.summary.number}.
                          </span>
                          <span className="min-w-0 flex-1 truncate text-[13px] font-medium">
                            {item.summary.title}
                          </span>
                          <DifficultyBadge
                            difficulty={item.summary.difficulty}
                            className="shrink-0"
                          />
                        </>
                      )}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>

        {/* hint bar */}
        <div className="flex items-center gap-3 border-t bg-sidebar px-3.5 py-2 font-mono text-[10.5px] text-muted-foreground">
          <span>↑↓ navigate</span>
          <span>↵ open</span>
          <span className="flex-1" />
          <span>{summaries ? `${summaries.length} problems` : ""}</span>
        </div>
      </DialogContent>
    </Dialog>
  );
}
