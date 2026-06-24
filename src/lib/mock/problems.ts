import type { Problem, ProblemStatus } from "@/src/lib/types";

/**
 * Mock built-in problem library. Every statement, example, constraint, hint,
 * and test case here is ORIGINAL content authored for Anvil (PROJECT_SPEC §4)
 * — classic concepts, our own expression.
 */
export const MOCK_PROBLEMS: Problem[] = [
  {
    id: "pair-with-target-sum",
    number: 1,
    title: "Pair With Target Sum",
    pattern: "Arrays & Hashing",
    difficulty: "Easy",
    source: "built-in",
    description_md:
      "Given an array of integers `nums` and an integer `target`, return the **indices** of the two numbers that add up to `target`.\n\nYou may assume that each input has *exactly one* solution, and you may not use the same element twice. You can return the answer in any order.",
    constraints: [
      "`2 <= nums.length <= 10^4`",
      "`-10^9 <= nums[i] <= 10^9`",
      "Only one valid answer exists.",
    ],
    examples: [
      {
        input: "nums = [2,7,11,15], target = 9",
        output: "[0,1]",
        explanation_md: "Because nums[0] + nums[1] == 9, we return [0, 1].",
      },
      { input: "nums = [3,2,4], target = 6", output: "[1,2]" },
    ],
    function_signature: {
      python: "def solve(nums, target):\n    # write your solution\n    pass",
      javascript: "function solve(nums, target) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [[2, 7, 11, 15], 9], expected: [0, 1], hidden: false },
      { input: [[3, 2, 4], 6], expected: [1, 2], hidden: false },
      { input: [[3, 3], 6], expected: [0, 1], hidden: true },
    ],
    hints: [
      "A brute-force pass over every pair works but is O(n²).",
      "Store each value's index in a hash map as you scan.",
      "For each element, check whether `target - value` is already in the map; if so you have your pair, otherwise record the current value and move on.",
    ],
    reference_solution: {
      python:
        "def solve(nums, target):\n    seen = {}\n    for i, n in enumerate(nums):\n        if target - n in seen:\n            return [seen[target - n], i]\n        seen[n] = i\n    return []",
      javascript:
        "function solve(nums, target) {\n  const seen = new Map();\n  for (let i = 0; i < nums.length; i++) {\n    const need = target - nums[i];\n    if (seen.has(need)) return [seen.get(need), i];\n    seen.set(nums[i], i);\n  }\n  return [];\n}",
      complexity: { time: "O(n)", space: "O(n)" },
    },
    explanation_md:
      "Scan the array once while remembering, for every value already visited, the index where it lives. At each element you only need to ask one question: *has the complementary value (`target - current`) appeared before?* A hash map answers that in O(1), so the whole scan is linear.\n\nThe brute-force alternative compares every pair, which is O(n²) — fine for tiny arrays, painful at 10⁴ elements.",
    follow_up: "Can you devise an algorithm that runs in O(n) time?",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "balanced-bracket-check",
    number: 2,
    title: "Balanced Bracket Check",
    pattern: "Stack",
    difficulty: "Easy",
    source: "built-in",
    description_md:
      "You are given a string `s` made up only of the characters `(`, `)`, `[`, `]`, `{`, and `}`.\n\nReturn `true` when the string is **well-nested**: every opener has a matching closer of the same kind, and pairs close in the reverse order they were opened.",
    constraints: ["`1 <= s.length <= 10^4`", "`s` contains only the six bracket characters."],
    examples: [
      { input: 's = "([])"', output: "true" },
      {
        input: 's = "(]"',
        output: "false",
        explanation_md: "The `(` is closed by `]`, which is the wrong kind.",
      },
    ],
    function_signature: {
      python: "def solve(s):\n    # write your solution\n    pass",
      javascript: "function solve(s) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: ["([])"], expected: true, hidden: false },
      { input: ["(]"], expected: false, hidden: false },
      { input: ["{[()]}"], expected: true, hidden: true },
      { input: ["((("], expected: false, hidden: true },
    ],
    hints: [
      "Think about the order pairs must close in — last opened, first closed.",
      "A stack mirrors that order exactly: push openers, pop on closers.",
      "Push every opener. On a closer, the stack top must hold the matching opener; pop it. The string is balanced when the stack ends empty.",
    ],
    reference_solution: {
      python:
        "def solve(s):\n    pairs = {')': '(', ']': '[', '}': '{'}\n    stack = []\n    for ch in s:\n        if ch in pairs:\n            if not stack or stack.pop() != pairs[ch]:\n                return False\n        else:\n            stack.append(ch)\n    return not stack",
      javascript:
        "function solve(s) {\n  const pairs = { ')': '(', ']': '[', '}': '{' };\n  const stack = [];\n  for (const ch of s) {\n    if (ch in pairs) {\n      if (stack.pop() !== pairs[ch]) return false;\n    } else {\n      stack.push(ch);\n    }\n  }\n  return stack.length === 0;\n}",
      complexity: { time: "O(n)", space: "O(n)" },
    },
    explanation_md:
      "Brackets close in last-in-first-out order, which is precisely what a stack models. Walk the string once: openers go on the stack; each closer must match the most recent unmatched opener, so compare it with the popped top. Any mismatch — or leftovers at the end — means the string isn't balanced.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "longest-stretch-without-repeats",
    number: 3,
    title: "Longest Stretch Without Repeats",
    pattern: "Sliding Window",
    difficulty: "Medium",
    source: "built-in",
    description_md:
      "Given a string `s`, return the length of the **longest contiguous stretch** of characters in which no character appears more than once.",
    constraints: ["`0 <= s.length <= 5 * 10^4`", "`s` consists of printable ASCII characters."],
    examples: [
      {
        input: 's = "tracetop"',
        output: "7",
        explanation_md: 'The longest run with no repeated character is `"racetop"` — seven characters. The only duplicate in the string is `t`; dropping the leading one leaves every remaining character distinct.',
      },
      { input: 's = "zzzz"', output: "1" },
    ],
    function_signature: {
      python: "def solve(s):\n    # write your solution\n    pass",
      javascript: "function solve(s) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: ["tracetop"], expected: 7, hidden: false },
      { input: ["zzzz"], expected: 1, hidden: false },
      { input: [""], expected: 0, hidden: true },
      { input: ["abrupt"], expected: 6, hidden: true },
    ],
    hints: [
      "When a repeat appears, the stretch can't simply restart from scratch — only its left edge must move.",
      "Maintain a window [left, right] and a map of each character's most recent position.",
      "Advance `right` one step at a time; when s[right] was seen inside the window, jump `left` past its previous position. Track the best window length seen.",
    ],
    reference_solution: {
      python:
        "def solve(s):\n    last = {}\n    best = left = 0\n    for right, ch in enumerate(s):\n        if ch in last and last[ch] >= left:\n            left = last[ch] + 1\n        last[ch] = right\n        best = max(best, right - left + 1)\n    return best",
      javascript:
        "function solve(s) {\n  const last = new Map();\n  let best = 0, left = 0;\n  for (let right = 0; right < s.length; right++) {\n    const ch = s[right];\n    if (last.has(ch) && last.get(ch) >= left) left = last.get(ch) + 1;\n    last.set(ch, right);\n    best = Math.max(best, right - left + 1);\n  }\n  return best;\n}",
      complexity: { time: "O(n)", space: "O(min(n, alphabet))" },
    },
    explanation_md:
      "Grow a window rightward one character at a time. The window stays valid as long as the incoming character hasn't been seen *inside* it; if it has, slide the left edge to just past that earlier occurrence. Each edge only ever moves forward, so the total work is linear.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "shifted-array-pivot",
    number: 4,
    title: "Find the Pivot in a Shifted Array",
    pattern: "Binary Search",
    difficulty: "Medium",
    source: "built-in",
    description_md:
      "A strictly increasing array was **rotated** at some unknown index: a block from the front was moved, in order, to the back. Given the rotated array `nums`, return the **index of the smallest element** (the pivot). If the array was not rotated at all, return `0`.\n\nYour algorithm must run in `O(log n)` time.",
    constraints: ["`1 <= nums.length <= 10^5`", "All values in `nums` are unique.", "`nums` was strictly increasing before the rotation."],
    examples: [
      {
        input: "nums = [15,18,2,3,6,12]",
        output: "2",
        explanation_md: "The smallest value, 2, sits at index 2.",
      },
      { input: "nums = [1,2,3,4]", output: "0" },
    ],
    function_signature: {
      python: "def solve(nums):\n    # write your solution\n    pass",
      javascript: "function solve(nums) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [[15, 18, 2, 3, 6, 12]], expected: 2, hidden: false },
      { input: [[1, 2, 3, 4]], expected: 0, hidden: false },
      { input: [[2, 1]], expected: 1, hidden: true },
      { input: [[7]], expected: 0, hidden: true },
    ],
    hints: [
      "Comparing the middle element to the *last* element tells you which half holds the pivot.",
      "If nums[mid] > nums[hi], the drop is to the right of mid; otherwise it's at mid or to the left.",
      "Shrink [lo, hi] with that rule until lo == hi — that index is the pivot.",
    ],
    reference_solution: {
      python:
        "def solve(nums):\n    lo, hi = 0, len(nums) - 1\n    while lo < hi:\n        mid = (lo + hi) // 2\n        if nums[mid] > nums[hi]:\n            lo = mid + 1\n        else:\n            hi = mid\n    return lo",
      javascript:
        "function solve(nums) {\n  let lo = 0, hi = nums.length - 1;\n  while (lo < hi) {\n    const mid = (lo + hi) >> 1;\n    if (nums[mid] > nums[hi]) lo = mid + 1;\n    else hi = mid;\n  }\n  return lo;\n}",
      complexity: { time: "O(log n)", space: "O(1)" },
    },
    explanation_md:
      "The rotated array is two sorted runs glued together, and the pivot is where the value *drops*. Compare the midpoint against the right end: if the midpoint is larger, the drop must be to its right; otherwise the drop is at the midpoint or to its left. Either way half the range is discarded each step — classic binary search on a condition rather than a target value.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "k-way-list-weave",
    number: 5,
    title: "K-Way List Weave",
    pattern: "Heap / Priority Queue",
    difficulty: "Hard",
    source: "built-in",
    description_md:
      "You are given `lists`, an array of `k` arrays, each already sorted in ascending order. Weave them into a **single sorted array** containing every element, and return it.\n\nAim for a solution that beats concatenating and re-sorting.",
    constraints: ["`0 <= k <= 10^4`", "`0 <= lists[i].length <= 500`", "The total number of elements does not exceed 10^5."],
    examples: [
      {
        input: "lists = [[1,4,5],[1,3,4],[2,6]]",
        output: "[1,1,2,3,4,4,5,6]",
        explanation_md: "All eight values, merged into one ascending run.",
      },
      { input: "lists = []", output: "[]" },
    ],
    function_signature: {
      python: "def solve(lists):\n    # write your solution\n    pass",
      javascript: "function solve(lists) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [[[1, 4, 5], [1, 3, 4], [2, 6]]], expected: [1, 1, 2, 3, 4, 4, 5, 6], hidden: false },
      { input: [[]], expected: [], hidden: false },
      { input: [[[], [0]]], expected: [0], hidden: true },
      { input: [[[5], [4], [3]]], expected: [3, 4, 5], hidden: true },
    ],
    hints: [
      "At any moment, the next output value is the smallest of the k current 'front' elements.",
      "A min-heap keyed on those front values gives you that smallest element in O(log k).",
      "Push (value, listIndex, elementIndex) triples; after popping one, push the next element from the same list. Alternatively, merge lists pairwise like merge sort.",
    ],
    reference_solution: {
      python:
        "import heapq\n\ndef solve(lists):\n    heap = [(lst[0], i, 0) for i, lst in enumerate(lists) if lst]\n    heapq.heapify(heap)\n    out = []\n    while heap:\n        val, i, j = heapq.heappop(heap)\n        out.append(val)\n        if j + 1 < len(lists[i]):\n            heapq.heappush(heap, (lists[i][j + 1], i, j + 1))\n    return out",
      javascript:
        "function solve(lists) {\n  // pairwise merge, O(N log k) overall\n  const merge = (a, b) => {\n    const out = [];\n    let i = 0, j = 0;\n    while (i < a.length || j < b.length) {\n      if (j >= b.length || (i < a.length && a[i] <= b[j])) out.push(a[i++]);\n      else out.push(b[j++]);\n    }\n    return out;\n  };\n  let queue = lists.slice();\n  if (queue.length === 0) return [];\n  while (queue.length > 1) {\n    const next = [];\n    for (let i = 0; i < queue.length; i += 2) {\n      next.push(i + 1 < queue.length ? merge(queue[i], queue[i + 1]) : queue[i]);\n    }\n    queue = next;\n  }\n  return queue[0];\n}",
      complexity: { time: "O(N log k)", space: "O(k)" },
    },
    explanation_md:
      "Two equivalent strategies hit O(N log k):\n\n1. **Min-heap of fronts.** Keep the current head of each list in a heap. Pop the global minimum, append it to the output, and feed in that list's next element. Every element passes through the heap once at O(log k) each.\n2. **Pairwise merging.** Merge lists two at a time, halving the count per round — log k rounds over N total elements.\n\nBoth dominate the naive concatenate-and-sort, which costs O(N log N).",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "course-unlock-order",
    number: 6,
    title: "Course Unlock Order",
    pattern: "Graphs",
    difficulty: "Medium",
    source: "built-in",
    description_md:
      "A training program has `n` modules labeled `0` to `n - 1`. The array `prereqs` holds pairs `[a, b]` meaning module `b` must be completed **before** module `a` unlocks.\n\nReturn `true` if every module can eventually be completed, or `false` if the prerequisites contradict each other.",
    constraints: ["`1 <= n <= 2000`", "`0 <= prereqs.length <= 5000`", "All pairs are distinct."],
    examples: [
      { input: "n = 2, prereqs = [[1,0]]", output: "true" },
      {
        input: "n = 2, prereqs = [[1,0],[0,1]]",
        output: "false",
        explanation_md: "Each module waits on the other — a deadlock.",
      },
    ],
    function_signature: {
      python: "def solve(n, prereqs):\n    # write your solution\n    pass",
      javascript: "function solve(n, prereqs) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [2, [[1, 0]]], expected: true, hidden: false },
      { input: [2, [[1, 0], [0, 1]]], expected: false, hidden: false },
      { input: [5, [[1, 0], [2, 1], [3, 2], [4, 3]]], expected: true, hidden: true },
      { input: [3, [[0, 1], [1, 2], [2, 0]]], expected: false, hidden: true },
    ],
    hints: [
      "Rephrase it as a graph question: modules are nodes, prerequisites are directed edges.",
      "The schedule is impossible exactly when the graph contains a directed cycle.",
      "Count incoming edges per node and repeatedly remove nodes with zero in-degree (Kahn's algorithm). If you can't remove all n, there's a cycle.",
    ],
    reference_solution: {
      python:
        "from collections import deque\n\ndef solve(n, prereqs):\n    adj = [[] for _ in range(n)]\n    indeg = [0] * n\n    for a, b in prereqs:\n        adj[b].append(a)\n        indeg[a] += 1\n    q = deque(i for i in range(n) if indeg[i] == 0)\n    done = 0\n    while q:\n        node = q.popleft()\n        done += 1\n        for nxt in adj[node]:\n            indeg[nxt] -= 1\n            if indeg[nxt] == 0:\n                q.append(nxt)\n    return done == n",
      javascript:
        "function solve(n, prereqs) {\n  const adj = Array.from({ length: n }, () => []);\n  const indeg = new Array(n).fill(0);\n  for (const [a, b] of prereqs) {\n    adj[b].push(a);\n    indeg[a]++;\n  }\n  const q = [];\n  for (let i = 0; i < n; i++) if (indeg[i] === 0) q.push(i);\n  let done = 0;\n  while (q.length) {\n    const node = q.shift();\n    done++;\n    for (const nxt of adj[node]) {\n      if (--indeg[nxt] === 0) q.push(nxt);\n    }\n  }\n  return done === n;\n}",
      complexity: { time: "O(n + e)", space: "O(n + e)" },
    },
    explanation_md:
      "Model the prerequisites as a directed graph. A valid completion order exists iff the graph is acyclic, and Kahn's algorithm checks that constructively: peel off nodes with no unmet prerequisites, decrementing the in-degree of their dependents. If the peeling stalls before all n nodes are gone, the leftover nodes form a cycle.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "connected-plot-counter",
    number: 7,
    title: "Connected Plot Counter",
    pattern: "Graphs",
    difficulty: "Medium",
    source: "built-in",
    description_md:
      "A survey drone maps farmland as a 2-D grid of characters: `'1'` marks a planted plot and `'0'` marks empty soil. Plots belong to the same **field** when they touch horizontally or vertically.\n\nGiven the grid, return the number of distinct fields.",
    constraints: ["`1 <= rows, cols <= 300`", "Each cell is `'1'` or `'0'`."],
    examples: [
      {
        input: 'grid = [["1","1","0"],["0","1","0"],["0","0","1"]]',
        output: "2",
        explanation_md: "The three connected 1s form one field; the bottom-right 1 is its own field.",
      },
    ],
    function_signature: {
      python: "def solve(grid):\n    # write your solution\n    pass",
      javascript: "function solve(grid) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [[["1", "1", "0"], ["0", "1", "0"], ["0", "0", "1"]]], expected: 2, hidden: false },
      { input: [[["0"]]], expected: 0, hidden: false },
      { input: [[["1", "0", "1"], ["0", "1", "0"], ["1", "0", "1"]]], expected: 5, hidden: true },
    ],
    hints: [
      "Each unvisited planted cell you encounter starts a brand-new field.",
      "From that cell, flood outward to absorb every reachable planted neighbour.",
      "Depth-first search (or BFS) marking cells as visited gives an O(rows × cols) sweep; the answer is how many floods you started.",
    ],
    reference_solution: {
      python:
        "def solve(grid):\n    rows, cols = len(grid), len(grid[0])\n    def flood(r, c):\n        if r < 0 or r >= rows or c < 0 or c >= cols or grid[r][c] != '1':\n            return\n        grid[r][c] = '0'\n        flood(r + 1, c); flood(r - 1, c); flood(r, c + 1); flood(r, c - 1)\n    count = 0\n    for r in range(rows):\n        for c in range(cols):\n            if grid[r][c] == '1':\n                count += 1\n                flood(r, c)\n    return count",
      javascript:
        "function solve(grid) {\n  const rows = grid.length, cols = grid[0].length;\n  const flood = (r, c) => {\n    if (r < 0 || r >= rows || c < 0 || c >= cols || grid[r][c] !== '1') return;\n    grid[r][c] = '0';\n    flood(r + 1, c); flood(r - 1, c); flood(r, c + 1); flood(r, c - 1);\n  };\n  let count = 0;\n  for (let r = 0; r < rows; r++) {\n    for (let c = 0; c < cols; c++) {\n      if (grid[r][c] === '1') {\n        count++;\n        flood(r, c);\n      }\n    }\n  }\n  return count;\n}",
      complexity: { time: "O(rows × cols)", space: "O(rows × cols)" },
    },
    explanation_md:
      "Sweep the grid cell by cell. Whenever you hit a planted cell that hasn't been absorbed yet, that's a new field: flood-fill from it, erasing (or marking) every connected planted cell so it's never counted again. Every cell is visited a constant number of times, so the sweep is linear in the grid size.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "cheapest-staircase-climb",
    number: 8,
    title: "Cheapest Staircase Climb",
    pattern: "1-D DP",
    difficulty: "Easy",
    source: "built-in",
    description_md:
      "A staircase charges a toll per step: `cost[i]` is the price of standing on step `i`. After paying for a step you may climb **one or two** steps further. You may start on step `0` or step `1`.\n\nReturn the minimum total cost to reach the landing just past the final step.",
    constraints: ["`2 <= cost.length <= 1000`", "`0 <= cost[i] <= 999`"],
    examples: [
      {
        input: "cost = [10,15,20]",
        output: "15",
        explanation_md: "Start on step 1 (pay 15) and jump two steps straight to the landing.",
      },
      { input: "cost = [1,100,1,1,1,100,1,1,100,1]", output: "6" },
    ],
    function_signature: {
      python: "def solve(cost):\n    # write your solution\n    pass",
      javascript: "function solve(cost) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [[10, 15, 20]], expected: 15, hidden: false },
      { input: [[1, 100, 1, 1, 1, 100, 1, 1, 100, 1]], expected: 6, hidden: false },
      { input: [[5, 5]], expected: 5, hidden: true },
    ],
    hints: [
      "The cheapest way to reach step i only depends on the two steps below it.",
      "Let best[i] be the minimum cost to *leave* step i; build it bottom-up.",
      "best[i] = cost[i] + min(best[i-1], best[i-2]); the answer is min of the last two. Two rolling variables suffice.",
    ],
    reference_solution: {
      python:
        "def solve(cost):\n    a, b = cost[0], cost[1]\n    for c in cost[2:]:\n        a, b = b, c + min(a, b)\n    return min(a, b)",
      javascript:
        "function solve(cost) {\n  let a = cost[0], b = cost[1];\n  for (let i = 2; i < cost.length; i++) {\n    [a, b] = [b, cost[i] + Math.min(a, b)];\n  }\n  return Math.min(a, b);\n}",
      complexity: { time: "O(n)", space: "O(1)" },
    },
    explanation_md:
      "Because each move covers one or two steps, the cheapest cost of standing on any step is its own toll plus the cheaper of the two steps beneath it. Rolling that recurrence forward with two variables avoids any array, and the landing is reachable from either of the last two steps — take the cheaper.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "meeting-slot-merger",
    number: 9,
    title: "Meeting Slot Merger",
    pattern: "Intervals",
    difficulty: "Medium",
    source: "built-in",
    description_md:
      "A calendar export lists busy slots as `[start, end]` pairs, possibly overlapping and in no particular order. Merge every group of overlapping slots and return the consolidated list, sorted by start time.\n\nSlots that merely touch (one ends exactly when another begins) count as overlapping.",
    constraints: ["`1 <= slots.length <= 10^4`", "`0 <= start <= end <= 10^6`"],
    examples: [
      {
        input: "slots = [[1,3],[8,10],[2,6]]",
        output: "[[1,6],[8,10]]",
        explanation_md: "[1,3] and [2,6] overlap, collapsing into [1,6].",
      },
      { input: "slots = [[1,4],[4,5]]", output: "[[1,5]]" },
    ],
    function_signature: {
      python: "def solve(slots):\n    # write your solution\n    pass",
      javascript: "function solve(slots) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [[[1, 3], [8, 10], [2, 6]]], expected: [[1, 6], [8, 10]], hidden: false },
      { input: [[[1, 4], [4, 5]]], expected: [[1, 5]], hidden: false },
      { input: [[[3, 3]]], expected: [[3, 3]], hidden: true },
    ],
    hints: [
      "Order matters: sort the slots by start time first.",
      "After sorting, a slot either extends the last merged slot or starts a new one.",
      "Keep the output's last interval; if the next start ≤ its end, raise its end as needed, otherwise append.",
    ],
    reference_solution: {
      python:
        "def solve(slots):\n    slots.sort()\n    out = []\n    for s, e in slots:\n        if out and s <= out[-1][1]:\n            out[-1][1] = max(out[-1][1], e)\n        else:\n            out.append([s, e])\n    return out",
      javascript:
        "function solve(slots) {\n  slots.sort((a, b) => a[0] - b[0]);\n  const out = [];\n  for (const [s, e] of slots) {\n    const last = out[out.length - 1];\n    if (last && s <= last[1]) last[1] = Math.max(last[1], e);\n    else out.push([s, e]);\n  }\n  return out;\n}",
      complexity: { time: "O(n log n)", space: "O(n)" },
    },
    explanation_md:
      "Sorting by start time guarantees that overlaps are always with the most recently merged interval, never an earlier one. One linear pass then suffices: extend the current merged slot while starts keep landing inside it, and emit a fresh slot the moment a gap appears. The sort dominates the cost.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "single-lonely-number",
    number: 10,
    title: "Single Lonely Number",
    pattern: "Bit Manipulation",
    difficulty: "Easy",
    source: "built-in",
    description_md:
      "Every value in the array `nums` appears exactly **twice** — except one value, which appears once. Return that lonely value.\n\nDo it in linear time using **constant** extra space.",
    constraints: ["`1 <= nums.length <= 3 * 10^4`", "`nums.length` is odd.", "Every element appears twice except one."],
    examples: [
      { input: "nums = [4,1,2,1,2]", output: "4" },
      { input: "nums = [7]", output: "7" },
    ],
    function_signature: {
      python: "def solve(nums):\n    # write your solution\n    pass",
      javascript: "function solve(nums) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [[4, 1, 2, 1, 2]], expected: 4, hidden: false },
      { input: [[7]], expected: 7, hidden: false },
      { input: [[0, 1, 0]], expected: 1, hidden: true },
    ],
    hints: [
      "A hash map works, but it spends O(n) memory — the constraint forbids that.",
      "XOR of a value with itself is 0, and XOR with 0 is the value itself.",
      "XOR the whole array together: the pairs annihilate, leaving exactly the lonely number.",
    ],
    reference_solution: {
      python: "from functools import reduce\nfrom operator import xor\n\ndef solve(nums):\n    return reduce(xor, nums)",
      javascript: "function solve(nums) {\n  return nums.reduce((acc, n) => acc ^ n, 0);\n}",
      complexity: { time: "O(n)", space: "O(1)" },
    },
    explanation_md:
      "XOR is associative, commutative, and self-cancelling (`x ^ x == 0`). Folding it across the array makes every duplicated value vanish in pairs regardless of order, so the running result ends as the single unpaired value — no extra memory needed.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "subset-builder",
    number: 11,
    title: "Subset Builder",
    pattern: "Backtracking",
    difficulty: "Medium",
    source: "built-in",
    description_md:
      "Given an array `nums` of **distinct** integers, return every possible subset (the power set). The subsets may be returned in any order, but no subset may appear twice. Within each subset, keep elements in their original array order.",
    constraints: ["`1 <= nums.length <= 10`", "All elements of `nums` are distinct."],
    examples: [
      { input: "nums = [1,2]", output: "[[],[1],[2],[1,2]]" },
      { input: "nums = [5]", output: "[[],[5]]" },
    ],
    function_signature: {
      python: "def solve(nums):\n    # write your solution\n    pass",
      javascript: "function solve(nums) {\n  // write your solution\n}",
    },
    checker: "unordered",
    test_cases: [
      { input: [[1, 2]], expected: [[], [1], [2], [1, 2]], hidden: false },
      { input: [[5]], expected: [[], [5]], hidden: false },
      { input: [[1, 2, 3]], expected: [[], [1], [1, 2], [1, 2, 3], [1, 3], [2], [2, 3], [3]], hidden: true },
    ],
    hints: [
      "Every element makes one binary choice: in the subset or not.",
      "Recursion over the index naturally enumerates both branches of each choice.",
      "Carry a growing path; at each index either skip or take nums[i], and record the path at every node (not just the leaves).",
    ],
    reference_solution: {
      python:
        "def solve(nums):\n    out = []\n    def walk(i, path):\n        out.append(path[:])\n        for j in range(i, len(nums)):\n            path.append(nums[j])\n            walk(j + 1, path)\n            path.pop()\n    walk(0, [])\n    return out",
      javascript:
        "function solve(nums) {\n  const out = [];\n  const walk = (i, path) => {\n    out.push([...path]);\n    for (let j = i; j < nums.length; j++) {\n      path.push(nums[j]);\n      walk(j + 1, path);\n      path.pop();\n    }\n  };\n  walk(0, []);\n  return out;\n}",
      complexity: { time: "O(n · 2^n)", space: "O(n)" },
    },
    explanation_md:
      "Each of the n elements independently joins or skips a subset, giving 2ⁿ subsets total. Backtracking enumerates them by extending a shared path: record the path at every recursion node, try appending each later element, recurse, then undo the append. Copying the path on record is what costs the extra factor of n.",
    license: "project-default",
    author: "built-in",
  },
  {
    id: "warehouse-pair-distance",
    number: 12,
    title: "Closest Warehouse Pair",
    pattern: "Two Pointers",
    difficulty: "Easy",
    source: "built-in",
    description_md:
      "Mile markers of warehouses along a highway are given in the **sorted** array `markers`. A dispatcher wants two distinct warehouses whose marker values **sum** to exactly `limit`.\n\nReturn their indices `[i, j]` with `i < j`, or `[-1, -1]` if no such pair exists. Use the sorted order — aim for constant extra space.",
    constraints: ["`2 <= markers.length <= 10^5`", "`markers` is sorted ascending.", "At most one valid pair exists."],
    examples: [
      { input: "markers = [1,4,5,9], limit = 13", output: "[1,3]" },
      { input: "markers = [2,5,8], limit = 6", output: "[-1,-1]" },
    ],
    function_signature: {
      python: "def solve(markers, limit):\n    # write your solution\n    pass",
      javascript: "function solve(markers, limit) {\n  // write your solution\n}",
    },
    test_cases: [
      { input: [[1, 4, 5, 9], 13], expected: [1, 3], hidden: false },
      { input: [[2, 5, 8], 6], expected: [-1, -1], hidden: false },
      { input: [[1, 2], 3], expected: [0, 1], hidden: true },
    ],
    hints: [
      "The array is already sorted — exploit that instead of hashing.",
      "Put one pointer at each end and look at their sum.",
      "Sum too small → advance the left pointer; too large → retreat the right one. They meet in O(n).",
    ],
    reference_solution: {
      python:
        "def solve(markers, limit):\n    i, j = 0, len(markers) - 1\n    while i < j:\n        s = markers[i] + markers[j]\n        if s == limit:\n            return [i, j]\n        if s < limit:\n            i += 1\n        else:\n            j -= 1\n    return [-1, -1]",
      javascript:
        "function solve(markers, limit) {\n  let i = 0, j = markers.length - 1;\n  while (i < j) {\n    const s = markers[i] + markers[j];\n    if (s === limit) return [i, j];\n    if (s < limit) i++;\n    else j--;\n  }\n  return [-1, -1];\n}",
      complexity: { time: "O(n)", space: "O(1)" },
    },
    explanation_md:
      "With sorted input, two pointers squeeze toward each other: a sum that's too small can only be fixed by a larger left value, and one that's too large only by a smaller right value. Each step eliminates one candidate index permanently, so the scan is linear with no extra memory.",
    license: "project-default",
    author: "built-in",
  },
];

/** Per-problem progress used to derive statuses in lists. */
export const MOCK_STATUSES: Record<string, { status: ProblemStatus; lastAttempted?: string }> = {
  "pair-with-target-sum": { status: "solved", lastAttempted: "2d ago" },
  "balanced-bracket-check": { status: "solved", lastAttempted: "5d ago" },
  "longest-stretch-without-repeats": { status: "needs-review", lastAttempted: "1w ago" },
  "shifted-array-pivot": { status: "todo" },
  "k-way-list-weave": { status: "in-progress", lastAttempted: "3h ago" },
  "course-unlock-order": { status: "todo" },
  "connected-plot-counter": { status: "in-progress", lastAttempted: "yesterday" },
  "cheapest-staircase-climb": { status: "solved", lastAttempted: "4d ago" },
  "meeting-slot-merger": { status: "todo" },
  "single-lonely-number": { status: "solved", lastAttempted: "6d ago" },
  "subset-builder": { status: "todo" },
  "warehouse-pair-distance": { status: "solved", lastAttempted: "1w ago" },
};
