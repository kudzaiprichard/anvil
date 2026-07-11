"use client";

/**
 * Shared filter/search model for the problem-list sheet and the library page
 * (task 0005 — one model, two surfaces).
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import { listProblems } from "@/src/lib/api";
import type {
  Difficulty,
  Pattern,
  ProblemFilter,
  ProblemSort,
  ProblemStatus,
  ProblemSummary,
} from "@/src/lib/types";

export interface ProblemFilters {
  search: string;
  pattern?: Pattern;
  difficulty?: Difficulty;
  status?: ProblemStatus;
  sort: ProblemSort;
}

export const DEFAULT_FILTERS: ProblemFilters = { search: "", sort: "number" };

export const STATUS_LABELS: Record<ProblemStatus, string> = {
  todo: "Todo",
  "in-progress": "Attempted",
  solved: "Solved",
  "needs-review": "Needs review",
};

export function useProblemList(filters: ProblemFilters) {
  const [allRows, setAllRows] = useState<ProblemSummary[]>([]);
  // bump to refetch after a mutation (import, etc.) without remounting.
  const [reloadNonce, setReloadNonce] = useState(0);
  const reload = useCallback(() => setReloadNonce((n) => n + 1), []);

  const query = useMemo<ProblemFilter>(
    () => ({
      search: filters.search || undefined,
      pattern: filters.pattern,
      difficulty: filters.difficulty,
      status: filters.status,
      sort: filters.sort,
    }),
    [filters.search, filters.pattern, filters.difficulty, filters.status, filters.sort]
  );
  const queryKey = useMemo(() => JSON.stringify(query), [query]);

  // `loading` is derived: rows are stale until their key matches the query.
  const [result, setResult] = useState<{
    key: string;
    rows: ProblemSummary[];
  }>({ key: "", rows: [] });

  useEffect(() => {
    listProblems().then(setAllRows);
  }, [reloadNonce]);

  useEffect(() => {
    let cancelled = false;
    listProblems(query).then((rows) => {
      if (!cancelled) setResult({ key: queryKey, rows });
    });
    return () => {
      cancelled = true;
    };
  }, [query, queryKey, reloadNonce]);

  const rows = result.rows;
  const loading = result.key !== queryKey;

  const solvedCount = useMemo(
    () => allRows.filter((r) => r.status === "solved").length,
    [allRows]
  );

  return { rows, allRows, solvedCount, loading, reload };
}

export function hasActiveFilters(filters: ProblemFilters): boolean {
  return Boolean(
    filters.search || filters.pattern || filters.difficulty || filters.status
  );
}
