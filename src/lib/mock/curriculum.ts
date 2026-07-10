import type { Curriculum, Unit } from "@/src/lib/types";

/**
 * Mock course content — mirrors the shipped Phase-1 Stage-1 vertical slice
 * (`src-tauri/resources/curriculum/`) so browser-dev has the same shape the
 * real backend loads. No lessons yet (Phase 2 authors those).
 */
export const MOCK_CURRICULUM: Curriculum = {
  id: "dsa-track",
  stages: [
    {
      id: "s1",
      title: "Array Fundamentals",
      units: ["arrays-hashing", "two-pointers", "sliding-window"],
    },
  ],
  prereqs: {
    "two-pointers": ["arrays-hashing"],
    "sliding-window": ["arrays-hashing", "two-pointers"],
  },
  gate_defaults: {
    pass_count: 4,
    require_novel: true,
    timer_target_min: 25,
    threshold_pct: 80,
  },
};

export const MOCK_UNITS: Unit[] = [
  {
    id: "arrays-hashing",
    stage: "s1",
    title: "Arrays & Hashing",
    prereqs: [],
    lessons: [],
    problems: [
      { slug: "two-sum", role: "worked", tier: "intro", novel: false },
      { slug: "group-anagrams", role: "worked", tier: "core", novel: false },
      { slug: "contains-duplicate", role: "guided", tier: "intro", novel: false },
      { slug: "valid-anagram", role: "guided", tier: "intro", novel: false },
      { slug: "top-k-frequent-elements", role: "guided", tier: "core", novel: false },
      { slug: "product-of-array-except-self", role: "guided", tier: "core", novel: false },
      { slug: "longest-consecutive-sequence", role: "gate", tier: "stretch", novel: true },
      { slug: "subarray-sum-equals-k", role: "gate", tier: "stretch", novel: true },
    ],
    gate: { pass_count: 2, require_novel: true, timer_target_min: 25, threshold_pct: 80 },
    spiral: [],
  },
  {
    id: "two-pointers",
    stage: "s1",
    title: "Two Pointers",
    prereqs: ["arrays-hashing"],
    lessons: [],
    problems: [
      { slug: "valid-palindrome", role: "worked", tier: "intro", novel: false },
      { slug: "two-sum-ii-input-array-is-sorted", role: "worked", tier: "intro", novel: false },
      { slug: "3sum", role: "guided", tier: "core", novel: false },
      { slug: "container-with-most-water", role: "guided", tier: "core", novel: false },
      { slug: "trapping-rain-water", role: "gate", tier: "stretch", novel: true },
    ],
    gate: { pass_count: 1, require_novel: true, timer_target_min: 25, threshold_pct: 80 },
    spiral: ["arrays-hashing"],
  },
  {
    id: "sliding-window",
    stage: "s1",
    title: "Sliding Window",
    prereqs: ["arrays-hashing", "two-pointers"],
    lessons: [],
    problems: [
      { slug: "best-time-to-buy-and-sell-stock", role: "worked", tier: "intro", novel: false },
      {
        slug: "longest-substring-without-repeating-characters",
        role: "worked",
        tier: "core",
        novel: false,
      },
      {
        slug: "longest-repeating-character-replacement",
        role: "guided",
        tier: "core",
        novel: false,
      },
      { slug: "permutation-in-string", role: "guided", tier: "core", novel: false },
      { slug: "minimum-window-substring", role: "gate", tier: "stretch", novel: true },
    ],
    gate: { pass_count: 1, require_novel: true, timer_target_min: 25, threshold_pct: 80 },
    spiral: ["arrays-hashing", "two-pointers"],
  },
];
