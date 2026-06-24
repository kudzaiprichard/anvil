"use client";

import { Suspense, useMemo, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { FileDown, FileUp, Search, X } from "lucide-react";
import { toast } from "sonner";
import { AppShell } from "@/src/components/anvil/app-shell";
import { DifficultyBadge } from "@/src/components/anvil/difficulty-badge";
import { EmptyState } from "@/src/components/anvil/empty-state";
import { Spinner } from "@/src/components/anvil/spinner";
import { StatusIndicator } from "@/src/components/anvil/status-indicator";
import {
  FilterDropdown,
  FilterRow,
  SearchInput,
} from "@/src/components/problems/filter-controls";
import {
  DEFAULT_FILTERS,
  useProblemList,
  type ProblemFilters,
} from "@/src/components/problems/use-problem-filters";
import { exportPack, importProblems } from "@/src/lib/api";
import { cn } from "@/src/lib/utils";
import type { Pattern, ProblemSort } from "@/src/lib/types";
import { PATTERNS } from "@/src/lib/types";

const SORT_LABELS: Record<ProblemSort, string> = {
  number: "Number",
  difficulty: "Difficulty",
  recent: "Recently attempted",
};

const GRID = "grid grid-cols-[32px_44px_minmax(0,1fr)_170px_88px_100px] gap-3.5";

function LibraryContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const urlPattern = searchParams.get("pattern");

  // The pattern filter lives in the URL (dashboard chips deep-link here);
  // everything else is local state.
  const pattern = PATTERNS.find((p) => p === urlPattern);
  const [base, setBase] = useState<ProblemFilters>(DEFAULT_FILTERS);
  const filters = useMemo<ProblemFilters>(
    () => ({ ...base, pattern }),
    [base, pattern]
  );

  const setFilters = (next: ProblemFilters) =>
    setBase((b) => ({
      ...b,
      search: next.search,
      difficulty: next.difficulty,
      status: next.status,
      sort: next.sort,
    }));

  const setPattern = (next: Pattern | undefined) => {
    router.replace(
      next ? `/problems?pattern=${encodeURIComponent(next)}` : "/problems"
    );
  };

  const { rows, allRows, solvedCount, loading, reload } = useProblemList(filters);

  const handleImport = async () => {
    try {
      const imported = await importProblems();
      if (!imported || imported.length === 0) return; // cancelled (or browser dev)
      if (imported.length === 1) {
        toast.success(`Imported "${imported[0].title}".`);
        router.push(`/problem?id=${imported[0].id}`);
      } else {
        toast.success(`Imported ${imported.length} problems.`);
        reload();
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Import failed");
    }
  };

  const handleExportPack = async () => {
    try {
      if (await exportPack()) toast.success("Problem pack exported.");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Export failed");
    }
  };

  const railStats = useMemo(
    () =>
      PATTERNS.map((pattern) => {
        const inPattern = allRows.filter((r) => r.pattern === pattern);
        return {
          pattern,
          solved: inPattern.filter((r) => r.status === "solved").length,
          total: inPattern.length,
        };
      }),
    [allRows]
  );

  return (
    <AppShell
      status={
        <span>
          {loading
            ? "loading…"
            : `${rows.length}/${allRows.length} shown · ${solvedCount} solved`}
        </span>
      }
    >
      <div className="flex min-h-0 flex-1">
        {/* pattern rail */}
        <aside className="flex w-[224px] shrink-0 flex-col border-r bg-sidebar">
          <div className="microlabel px-4 pb-2 pt-4">Patterns</div>
          <div className="flex min-h-0 flex-1 flex-col gap-px overflow-auto px-2 pb-3.5">
            <button
              type="button"
              onClick={() => setPattern(undefined)}
              className={cn(
                "flex items-center gap-2 rounded-md px-2.5 py-[7px] text-left text-[13px] transition-colors",
                !filters.pattern
                  ? "bg-sidebar-accent font-semibold text-primary"
                  : "font-medium text-sidebar-foreground/85 hover:bg-sidebar-accent hover:text-foreground"
              )}
            >
              <span className="min-w-0 flex-1 truncate">All problems</span>
              <span
                className={cn(
                  "shrink-0 font-mono text-[11px]",
                  !filters.pattern ? "text-primary" : "text-muted-foreground"
                )}
              >
                {allRows.length}
              </span>
            </button>
            {railStats.map(({ pattern, solved, total }) => {
              const active = filters.pattern === pattern;
              return (
                <button
                  key={pattern}
                  type="button"
                  onClick={() => setPattern(pattern)}
                  className={cn(
                    "flex items-center gap-2 rounded-md px-2.5 py-[7px] text-left text-[13px] transition-colors",
                    active
                      ? "bg-sidebar-accent font-semibold text-primary"
                      : "font-medium text-sidebar-foreground/85 hover:bg-sidebar-accent hover:text-foreground"
                  )}
                >
                  <span className="min-w-0 flex-1 truncate">{pattern}</span>
                  <span
                    className={cn(
                      "shrink-0 font-mono text-[11px]",
                      active ? "text-primary" : "text-muted-foreground"
                    )}
                  >
                    {solved}/{total}
                  </span>
                </button>
              );
            })}
          </div>
        </aside>

        {/* main */}
        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          <div className="shrink-0 px-6 pt-5">
            <div className="flex items-baseline gap-2.5">
              <h1 className="text-lg font-semibold tracking-tight">Problems</h1>
              <span className="font-mono text-xs text-muted-foreground">
                {loading
                  ? "…"
                  : `${rows.length} of ${allRows.length} · ${solvedCount} solved`}
              </span>
            </div>
            <div className="mt-3 flex items-center gap-2">
              <SearchInput
                className="flex-1"
                value={filters.search}
                onChange={(search) => setFilters({ ...filters, search })}
              />
              <FilterRow
                filters={filters}
                onChange={setFilters}
                showPattern={false}
              />
              <FilterDropdown<ProblemSort>
                label="Sort"
                value={filters.sort}
                options={["number", "difficulty", "recent"] as const}
                optionLabel={(s) => `Sort: ${SORT_LABELS[s]}`}
                onChange={(sort) =>
                  setFilters({ ...filters, sort: sort ?? "number" })
                }
              />
              <button
                type="button"
                onClick={handleImport}
                title="Import a problem file or a .anvilpack of many problems"
                className="flex shrink-0 items-center gap-1.5 rounded-md border bg-card px-2.5 py-[7px] text-xs font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              >
                <FileDown className="size-[13px]" />
                Import…
              </button>
              <button
                type="button"
                onClick={handleExportPack}
                title="Export all your problems as a shareable pack"
                className="flex shrink-0 items-center gap-1.5 rounded-md border bg-card px-2.5 py-[7px] text-xs font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              >
                <FileUp className="size-[13px]" />
                Export pack
              </button>
            </div>
          </div>

          {/* table header */}
          <div
            className={cn(
              GRID,
              "microlabel mx-6 mt-4 shrink-0 items-center border-b px-3 pb-2"
            )}
          >
            <span />
            <span>#</span>
            <span>Title</span>
            <span>Pattern</span>
            <span>Difficulty</span>
            <span>Last</span>
          </div>

          {/* rows / empty */}
          <div className="min-h-0 flex-1 overflow-auto px-6 pb-6">
            {loading ? (
              <div className="flex h-40 items-center justify-center">
                <Spinner className="size-5" />
              </div>
            ) : rows.length === 0 ? (
              <EmptyState
                icon={Search}
                title="No problems match your filters"
                description={
                  <>
                    Nothing in the library matches the current search and
                    filters. Try widening your search.
                  </>
                }
                action={
                  <button
                    type="button"
                    onClick={() => {
                      setFilters(DEFAULT_FILTERS);
                      router.replace("/problems");
                    }}
                    className="flex items-center gap-1.5 rounded-md border bg-card px-3.5 py-2 text-[13px] font-semibold transition-colors hover:bg-accent"
                  >
                    <X className="size-3.5" />
                    Clear filters
                  </button>
                }
                className="h-full"
              />
            ) : (
              rows.map((row) => (
                <button
                  key={row.id}
                  type="button"
                  onClick={() => router.push(`/problem?id=${row.id}`)}
                  className={cn(
                    GRID,
                    "group w-full items-center border-b border-border/60 px-3 py-[9px] text-left transition-colors last:border-b-0 hover:bg-accent/60"
                  )}
                >
                  <span className="flex items-center justify-center">
                    <StatusIndicator status={row.status} />
                  </span>
                  <span className="font-mono text-xs text-muted-foreground">
                    {row.number}
                  </span>
                  <span className="flex min-w-0 items-center gap-2">
                    <span className="truncate text-[13px] font-medium group-hover:text-foreground">
                      {row.title}
                    </span>
                    {row.source === "imported" && (
                      <span className="shrink-0 rounded-full border border-primary/40 px-1.5 py-px text-[10px] font-semibold uppercase tracking-wide text-primary">
                        imported
                      </span>
                    )}
                    {row.source === "user" && (
                      <span className="shrink-0 rounded-full border px-1.5 py-px text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
                        yours
                      </span>
                    )}
                  </span>
                  <span className="truncate text-[12.5px] text-muted-foreground">
                    {row.pattern}
                  </span>
                  <DifficultyBadge difficulty={row.difficulty} />
                  <span className="font-mono text-[11.5px] text-muted-foreground">
                    {row.lastAttempted ?? "—"}
                  </span>
                </button>
              ))
            )}
          </div>
        </div>
      </div>
    </AppShell>
  );
}

export default function ProblemsPage() {
  return (
    <Suspense
      fallback={
        <AppShell>
          <div className="flex flex-1 items-center justify-center">
            <Spinner className="size-6" />
          </div>
        </AppShell>
      }
    >
      <LibraryContent />
    </Suspense>
  );
}
