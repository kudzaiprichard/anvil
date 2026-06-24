"use client";

import { useState } from "react";
import { ChevronRight, SearchX, X } from "lucide-react";
import { DifficultyBadge } from "@/src/components/anvil/difficulty-badge";
import { EmptyState } from "@/src/components/anvil/empty-state";
import { Spinner } from "@/src/components/anvil/spinner";
import { StatusIndicator } from "@/src/components/anvil/status-indicator";
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetTitle,
} from "@/src/components/shadcn/sheet";
import { cn } from "@/src/lib/utils";
import {
  ActiveFilterChips,
  FilterRow,
  SearchInput,
} from "./filter-controls";
import {
  DEFAULT_FILTERS,
  useProblemList,
  type ProblemFilters,
} from "./use-problem-filters";

/**
 * Slide-over quick-nav inside the workspace (UI_SPEC §6.3). Shares the
 * filter model with the library page.
 */
export function ProblemSheet({
  open,
  onOpenChange,
  currentId,
  onSelect,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  currentId?: string;
  onSelect: (id: string) => void;
}) {
  const [filters, setFilters] = useState<ProblemFilters>(DEFAULT_FILTERS);
  const { rows, allRows, solvedCount, loading } = useProblemList(filters);

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        side="left"
        showCloseButton={false}
        className="w-[430px] gap-0 bg-card sm:max-w-[430px]"
      >
        {/* header */}
        <div className="shrink-0 border-b px-4 pb-[13px] pt-4">
          <div className="flex items-center gap-2">
            <SheetTitle className="text-[15px] font-semibold tracking-tight">
              Problem List
            </SheetTitle>
            <ChevronRight className="size-[15px] text-muted-foreground" />
            <div className="flex-1" />
            <span className="font-mono text-[12.5px] text-muted-foreground">
              {solvedCount} / {allRows.length} solved
            </span>
            <SheetClose className="flex size-[30px] items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-muted hover:text-foreground">
              <X className="size-4" />
              <span className="sr-only">Close</span>
            </SheetClose>
          </div>
          <SearchInput
            className="mt-[13px]"
            value={filters.search}
            onChange={(search) => setFilters({ ...filters, search })}
          />
          <div className="mt-[9px]">
            <FilterRow filters={filters} onChange={setFilters} />
          </div>
          <div className="mt-[9px] empty:hidden">
            <ActiveFilterChips filters={filters} onChange={setFilters} />
          </div>
        </div>

        {/* rows */}
        <div className="min-h-0 flex-1 overflow-auto p-1.5">
          {loading ? (
            <div className="flex h-32 items-center justify-center">
              <Spinner />
            </div>
          ) : rows.length === 0 ? (
            <EmptyState
              icon={SearchX}
              title="No problems match"
              description="Try widening your search or clearing a filter."
              className="min-h-[200px]"
            />
          ) : (
            rows.map((row) => {
              const current = row.id === currentId;
              return (
                <button
                  key={row.id}
                  type="button"
                  onClick={() => onSelect(row.id)}
                  className={cn(
                    "flex w-full items-center gap-2.5 rounded-[9px] px-2.5 py-[9px] text-left transition-colors",
                    current
                      ? "bg-primary/10 dark:bg-primary/15"
                      : "hover:bg-muted"
                  )}
                >
                  <span className="flex w-[18px] shrink-0 items-center justify-center">
                    <StatusIndicator
                      status={current ? "in-progress" : row.status}
                    />
                  </span>
                  <span className="w-[34px] shrink-0 text-right font-mono text-xs text-muted-foreground">
                    {row.number}.
                  </span>
                  <span
                    className={cn(
                      "min-w-0 flex-1 truncate text-[13px]",
                      current ? "font-semibold" : "font-medium"
                    )}
                  >
                    {row.title}
                  </span>
                  <DifficultyBadge
                    difficulty={row.difficulty}
                    className="shrink-0"
                  />
                </button>
              );
            })
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
