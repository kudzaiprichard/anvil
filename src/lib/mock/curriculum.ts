import type { Curriculum, Lesson, Quiz, Unit } from "@/src/lib/types";

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
          predict: {
            prompt_md:
              "At index 1 we hold **7** and need **2**. What happens next?",
            choices: [
              {
                id: "lookup",
                label_md:
                  "Check the map for `2` — it's there at index 0 — and return `[0, 1]`.",
              },
              {
                id: "store",
                label_md:
                  "Store `7 → 1` in the map, then keep scanning to index 2.",
              },
              {
                id: "scan",
                label_md:
                  "Compare `7` against every earlier value one by one to find its partner.",
              },
            ],
            answer: "lookup",
            explanation_md:
              "Before storing, we ask the map for the complement. `2` is already there at index 0, so a single **O(1)** lookup ends it — no nested scan.",
          },
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

/**
 * The interleaved, cross-unit pattern-picker pool (Phase 4) — mirrors
 * `resources/curriculum/pattern-pool.json` so browser dev drills the same
 * unlabeled recognition prompts the Rust loader serves.
 */
export const MOCK_PATTERN_POOL: Quiz = {
  items: [
    {
      id: "pp-unsorted-pair-sum",
      type: "pattern-picker",
      prompt_md:
        "You get an **unsorted** array of integers and must return the **positions** of two entries that add up to a target. What do you reach for first?",
      options: [
        "Walk two pointers inward from both ends",
        "Keep a hash map of values seen so far and look up each complement",
        "Slide a variable-width window across the array",
        "Sort the array, then binary-search for each complement",
      ],
      answer: "Keep a hash map of values seen so far and look up each complement",
      correct_pattern: "arrays-hashing",
      explanation_md:
        "**Unsorted + return original indices + O(n)** is the hash-map complement fingerprint. Sorting would scramble the positions you must report.",
    },
    {
      id: "pp-sorted-pair-sum",
      type: "pattern-picker",
      prompt_md:
        "An array is **already sorted ascending**. Find two values that sum to a target, using only **O(1)** extra space. Which technique fits best?",
      options: [
        "Keep a hash map of values seen so far and look up each complement",
        "Walk one pointer from each end, moving them based on the sum",
        "Expand and shrink a window over the array",
        "Recurse with memoization over index and remaining target",
      ],
      answer: "Walk one pointer from each end, moving them based on the sum",
      correct_pattern: "two-pointers",
      explanation_md:
        "**Sorted + O(1) space** is the tell for opposite-ends two pointers: too-small sum → advance left, too-large → retreat right.",
    },
    {
      id: "pp-longest-unique-substring",
      type: "pattern-picker",
      prompt_md:
        "Find the length of the **longest substring with no repeating characters** in a string. What shape does the solution take?",
      options: [
        "A fixed-size window swept once across the string",
        "A variable-size window that grows, and shrinks from the left on a repeat",
        "Sort the characters and scan for runs",
        "A hash map from each character to its global frequency",
      ],
      answer:
        "A variable-size window that grows, and shrinks from the left on a repeat",
      correct_pattern: "sliding-window",
      explanation_md:
        "**\"Longest substring such that a condition holds\"** is a dynamic sliding window: extend right, and shrink from the left when the invariant breaks.",
    },
    {
      id: "pp-container-most-water",
      type: "pattern-picker",
      prompt_md:
        "Given heights of vertical lines, pick two lines that with the x-axis hold the **most water**. Best approach?",
      options: [
        "Two pointers at the ends, always moving the shorter line inward",
        "A sliding window of fixed width",
        "A hash map of height to index",
        "Dynamic programming over subranges",
      ],
      answer: "Two pointers at the ends, always moving the shorter line inward",
      correct_pattern: "two-pointers",
      explanation_md:
        "Area is bounded by the **shorter** line, so moving the taller one can never help — advance the shorter pointer.",
    },
    {
      id: "pp-group-anagrams",
      type: "pattern-picker",
      prompt_md:
        "Group a list of words so that all **anagrams of each other** land in the same bucket. What organizes the buckets?",
      options: [
        "A hash map keyed by each word's canonical form (e.g. its sorted letters)",
        "Two pointers comparing words end to end",
        "A sliding window over each word",
        "Binary search after sorting the whole list",
      ],
      answer:
        "A hash map keyed by each word's canonical form (e.g. its sorted letters)",
      correct_pattern: "arrays-hashing",
      explanation_md:
        "Anagrams share one invariant — the same multiset of letters. Reduce each word to a **canonical key** and bucket by it. \"Group by an equivalence\" ⇒ hashing.",
    },
    {
      id: "pp-max-sum-k-window",
      type: "pattern-picker",
      prompt_md:
        "Find the maximum sum of any **exactly k consecutive** elements in an array of numbers. Which is the O(n) way?",
      options: [
        "A fixed-size window of width k, adding the entering and dropping the leaving element",
        "A variable-size window that grows while the sum is small",
        "A hash map of prefix sums",
        "Two pointers from both ends",
      ],
      answer:
        "A fixed-size window of width k, adding the entering and dropping the leaving element",
      correct_pattern: "sliding-window",
      explanation_md:
        "A **fixed** length ‘k consecutive’ is the fixed-size sliding window: slide by one, add the new element, subtract the one that fell off.",
    },
    {
      id: "pp-contains-duplicate",
      type: "pattern-picker",
      prompt_md:
        "Decide whether an array contains **any duplicate value at all** (just yes/no). Cheapest reliable approach?",
      options: [
        "Insert into a hash set and report a collision",
        "Two pointers scanning inward",
        "A sliding window of growing size",
        "Sort, then two pointers",
      ],
      answer: "Insert into a hash set and report a collision",
      correct_pattern: "arrays-hashing",
      explanation_md:
        "\"**Have I seen this before?**\" over an unordered collection is a hash-set membership check — one pass, O(n).",
    },
    {
      id: "pp-min-subarray-sum",
      type: "pattern-picker",
      prompt_md:
        "In an array of **positive** integers, find the length of the **shortest contiguous subarray whose sum is ≥ target**. What fits?",
      options: [
        "A variable-size window that grows to reach the target, then shrinks from the left",
        "A hash map from value to index",
        "Two pointers from opposite ends of the array",
        "Sort the array first, then scan",
      ],
      answer:
        "A variable-size window that grows to reach the target, then shrinks from the left",
      correct_pattern: "sliding-window",
      explanation_md:
        "Positive values make the window sum monotonic — growing raises it, shrinking lowers it — which a dynamic window exploits. Sorting would destroy contiguity.",
    },
  ],
};
