"use client";

import { ChevronDown, Search, X } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/src/components/shadcn/dropdown-menu";
import { cn } from "@/src/lib/utils";
import type { Difficulty, Pattern, ProblemStatus } from "@/src/lib/types";
import { PATTERNS } from "@/src/lib/types";
import {
  STATUS_LABELS,
  type ProblemFilters,
  hasActiveFilters,
} from "./use-problem-filters";

const DIFFICULTIES: Difficulty[] = ["Easy", "Medium", "Hard"];
const STATUSES: ProblemStatus[] = [
  "todo",
  "in-progress",
  "solved",
  "needs-review",
];

export function FilterDropdown<T extends string>({
  label,
  value,
  options,
  optionLabel,
  onChange,
}: {
  label: string;
  value: T | undefined;
  options: readonly T[];
  optionLabel?: (v: T) => string;
  onChange: (v: T | undefined) => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className={cn(
            "flex items-center gap-1.5 whitespace-nowrap rounded-md border px-2.5 py-1.5 text-[12.5px] transition-colors",
            value
              ? "border-primary/35 bg-primary/10 font-semibold text-primary dark:bg-primary/15"
              : "bg-card font-medium hover:bg-accent"
          )}
        >
          {value ? (optionLabel ? optionLabel(value) : value) : label}
          <ChevronDown
            className={cn(
              "size-3",
              value ? "text-primary" : "text-muted-foreground"
            )}
          />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        {options.map((opt) => (
          <DropdownMenuItem key={opt} onClick={() => onChange(opt)}>
            {optionLabel ? optionLabel(opt) : opt}
          </DropdownMenuItem>
        ))}
        {value && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={() => onChange(undefined)}>
              Clear
            </DropdownMenuItem>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function SearchInput({
  value,
  onChange,
  className,
  inputRef,
}: {
  value: string;
  onChange: (v: string) => void;
  className?: string;
  /** Focus target for the library's "/" shortcut. */
  inputRef?: React.Ref<HTMLInputElement>;
}) {
  return (
    <div
      className={cn(
        "flex items-center gap-2 rounded-md border bg-editor px-[11px] py-2 transition-colors focus-within:border-ring",
        className
      )}
    >
      <Search className="size-[15px] shrink-0 text-muted-foreground" />
      <input
        ref={inputRef}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Search problems…"
        className="w-full bg-transparent text-[13px] outline-none placeholder:text-muted-foreground"
      />
    </div>
  );
}

/** The three filter dropdowns, shared by the sheet and the library. */
export function FilterRow({
  filters,
  onChange,
  showPattern = true,
}: {
  filters: ProblemFilters;
  onChange: (next: ProblemFilters) => void;
  showPattern?: boolean;
}) {
  return (
    <div className="flex gap-[7px]">
      {showPattern && (
        <FilterDropdown<Pattern>
          label="Pattern"
          value={filters.pattern}
          options={PATTERNS}
          onChange={(pattern) => onChange({ ...filters, pattern })}
        />
      )}
      <FilterDropdown<Difficulty>
        label="Difficulty"
        value={filters.difficulty}
        options={DIFFICULTIES}
        onChange={(difficulty) => onChange({ ...filters, difficulty })}
      />
      <FilterDropdown<ProblemStatus>
        label="Status"
        value={filters.status}
        options={STATUSES}
        optionLabel={(s) => STATUS_LABELS[s]}
        onChange={(status) => onChange({ ...filters, status })}
      />
    </div>
  );
}

/** Active-filter chips with remove buttons + "Clear" (sheet mockup). */
export function ActiveFilterChips({
  filters,
  onChange,
}: {
  filters: ProblemFilters;
  onChange: (next: ProblemFilters) => void;
}) {
  if (!hasActiveFilters(filters)) return null;
  const chips: { label: string; clear: () => void }[] = [];
  if (filters.pattern)
    chips.push({
      label: filters.pattern,
      clear: () => onChange({ ...filters, pattern: undefined }),
    });
  if (filters.difficulty)
    chips.push({
      label: filters.difficulty,
      clear: () => onChange({ ...filters, difficulty: undefined }),
    });
  if (filters.status)
    chips.push({
      label: STATUS_LABELS[filters.status],
      clear: () => onChange({ ...filters, status: undefined }),
    });

  return (
    <div className="flex items-center gap-[7px]">
      {chips.map((chip) => (
        <span
          key={chip.label}
          className="inline-flex items-center gap-1.5 rounded-md bg-primary/10 py-1 pl-2.5 pr-2 text-[11.5px] font-semibold text-primary dark:bg-primary/15"
        >
          {chip.label}
          <button type="button" onClick={chip.clear} aria-label={`Remove ${chip.label} filter`}>
            <X className="size-3 stroke-[2.2]" />
          </button>
        </span>
      ))}
      <button
        type="button"
        onClick={() =>
          onChange({ search: filters.search, sort: filters.sort })
        }
        className="text-[11.5px] text-muted-foreground hover:text-foreground"
      >
        Clear
      </button>
    </div>
  );
}
