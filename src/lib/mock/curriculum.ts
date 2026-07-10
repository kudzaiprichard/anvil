import type { Curriculum, Lesson, Unit } from "@/src/lib/types";

/**
 * Mock course content — mirrors the shipped Stage-1 vertical slice
 * (`src-tauri/resources/curriculum/`) so browser-dev has the same shape the
 * real backend loads. Phase 2 authors the first lesson
 * (`arrays-hashing/01-hashmap-lookup`); the rest are still stubs.
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
    lessons: ["01-hashmap-lookup"],
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

/**
 * Authored lessons, keyed by id — mirrors `resources/lessons/**` so browser
 * dev renders the same lesson the Rust loader serves. Only the Phase-2 lesson
 * exists so far.
 */
export const MOCK_LESSONS: Record<string, Lesson> = {
  "01-hashmap-lookup": {
    id: "01-hashmap-lookup",
    unit: "arrays-hashing",
    subpattern: "Hash-map complement lookup",
    explainer_md: [
      "## The one idea",
      "",
      'A hash map turns the question **"have I seen the number I need?"** from a',
      "scan into a single O(1) lookup. That one trade — spend memory to remember",
      'what you\'ve passed — collapses a whole family of "find the pair / find the',
      'duplicate / match by key" problems from O(n²) down to O(n).',
      "",
      "In Python the map is a plain `dict`. As you walk the array once, you record",
      "each value you've passed; before recording, you first *ask the map* whether",
      "the partner you're looking for is already there.",
      "",
      "```python",
      "def two_sum(nums: list[int], target: int) -> list[int]:",
      "    seen: dict[int, int] = {}          # value -> the index we saw it at",
      "    for i, x in enumerate(nums):",
      "        need = target - x              # the one partner that completes x",
      "        if need in seen:               # O(1): have we passed it already?",
      "            return [seen[need], i]",
      "        seen[x] = i                    # remember x for a later element",
      "    return []",
      "```",
      "",
      "Notice we never sort. Sorting is O(n log n) and scrambles the original",
      "indices the problem asks us to return. The hash map keeps input order intact",
      "and still runs in one pass.",
    ].join("\n"),
    trigger_signals: [
      'You need to answer "have I already seen X?" in O(1) — membership, not order.',
      "You're pairing elements (a + b = target) and the array is unsorted.",
      "A brute-force scan is O(n²) and you want to trade memory for a single pass.",
    ],
    worked_example: "two-sum",
    diagram: {
      id: "hashmap-lookup",
      algorithm: "two-sum-hashmap",
      for_problem: "two-sum",
      mode: "view",
      steps: [
        {
          state: { i: 0, num: 2, need: 7, seen: {}, answer: null },
          caption_md:
            "At index 0 the value is **2**, so its partner would be `9 - 2 = 7`. The map is empty — record `2 → 0` and move on.",
        },
        {
          state: { i: 1, num: 7, need: 2, seen: { "2": 0 }, answer: null },
          caption_md:
            "Index 1 holds **7**, whose partner is `9 - 7 = 2`. What is the one lookup that decides this whole problem?",
        },
        {
          state: { i: 1, num: 7, need: 2, seen: { "2": 0 }, answer: [0, 1] },
          caption_md:
            "The partner **2** is already in the map at index 0 — return `[0, 1]`. One pass, each lookup O(1).",
        },
      ],
      predict_at: [1],
    },
    quiz: {
      items: [
        {
          id: "q1",
          type: "concept-check",
          prompt_md:
            "In the one-pass hash-map solution, what do you store as the map's **key**?",
          options: [
            "The array index",
            "The value at the current position",
            "The complement (target minus the value)",
            "A running sum",
          ],
          answer: "The value at the current position",
          explanation_md:
            'You store each value so a later element can ask "has my complement been seen?"',
        },
        {
          id: "q2",
          type: "pattern-picker",
          prompt_md:
            "You are given an **unsorted** array and must decide whether any two elements add up to a target `k`, returning their positions. Which approach fits with no sorting and a single pass?",
          options: [
            "Sort the array, then walk two pointers inward",
            "Slide a window and shrink it when the sum is too large",
            "Keep a hash map of values seen so far and look up each complement",
            "Try every pair with two nested loops",
          ],
          answer:
            "Keep a hash map of values seen so far and look up each complement",
          correct_pattern: "arrays-hashing",
          explanation_md:
            'Unsorted + "have I already seen the partner I need?" + O(n) is the fingerprint for a hash-map complement lookup.',
        },
        {
          id: "q3",
          type: "complexity",
          prompt_md:
            "What is the time complexity of the one-pass hash-map solution to Two Sum?",
          options: ["O(1)", "O(n)", "O(n log n)", "O(n^2)"],
          answer: "O(n)",
          explanation_md:
            "One scan, each map insert/lookup expected O(1) → O(n) time, O(n) space.",
        },
      ],
    },
    practice: ["contains-duplicate", "valid-anagram"],
    recap: [],
    follow_up: [
      "What if the array were already sorted — could you drop the hash map and use two pointers instead?",
      "What if you had to return *all* pairs that sum to the target, not just one?",
    ],
  },
};
