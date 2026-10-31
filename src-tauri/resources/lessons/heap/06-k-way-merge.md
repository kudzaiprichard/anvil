---
id: 06-k-way-merge
unit: heap
subpattern: "K-way merge with a heap"
trigger_signals:
  - "You have several sorted sequences (lists, rows, or two sorted arrays whose pair-sums form implicit rows) and need the smallest few items across all of them in order."
  - "The full space is huge (n times m combinations, or k lists) but you only need the first k results, so you must never materialise everything."
  - "Each item has an obvious \"next\" successor in its own sorted sequence, so the next global smallest is always the successor of something you already took."
worked_example: find-k-pairs-with-smallest-sums
diagram: 06-k-way-merge.diagram.json
quiz: 06-k-way-merge.quiz.json
practice:
  - smallest-range-covering-elements-from-k-lists
  - ugly-number-ii
recap:
  - 01-top-k
  - 02-heap-basics
follow_up:
  - "What if the two arrays were not sorted — would you sort them first, and what does that add to the cost?"
  - "What if you had three sorted arrays and wanted the k smallest triple sums — how does the frontier change?"
---

## The one idea

Picture the sums as a grid: row `i` holds `nums1[i] + nums2[0]`, `nums1[i] + nums2[1]`, and so on.
Because both inputs are sorted, **every row is sorted** — it is just a sorted list you never wrote down.
Finding the `k` smallest pairs is then the same as merging these rows and stopping after `k` items.

The key invariant: the next smallest unseen sum is always the **head of some row**. So you never need
the whole grid, only a **frontier** with one candidate per row, kept in a min-heap. Pop the smallest,
output it, and push its successor from the same row. The popped item's row now offers its next
candidate, and the frontier again contains the minimum of everything not yet taken.

## Why it beats the obvious approach

Brute force builds all `n * m` sums, sorts them, and slices `k`: O(n m log(n m)) time and O(n m) space.
That throws away the sortedness, and `k` may be tiny next to `n m` (a million combinations, five wanted).

```python
import heapq

def k_smallest_pairs(nums1: list[int], nums2: list[int], k: int) -> list[list[int]]:
    if not nums1 or not nums2 or k <= 0:
        return []
    # frontier: the first pair of each row (only the first k rows can matter)
    heap = [(nums1[i] + nums2[0], i, 0) for i in range(min(k, len(nums1)))]
    heapq.heapify(heap)                      # (sum, row index, column index)
    out = []
    while heap and len(out) < k:
        _, i, j = heapq.heappop(heap)        # smallest remaining sum overall
        out.append([nums1[i], nums2[j]])
        if j + 1 < len(nums2):               # successor: same row, next column
            heapq.heappush(heap, (nums1[i] + nums2[j + 1], i, j + 1))
    return out
```

Why there are no duplicates: each row only ever advances **one column at a time**, so a cell can be
pushed only by its left neighbour (or seeded as column 0). If you instead seeded just `(0, 0)` and pushed
both "right" and "down", the same cell could be reached twice, and you would need a `seen` set to
dedupe. Seeding the whole first column makes the visited set unnecessary.

**Time:** let `m = min(k, len(nums1))` be the number of rows seeded. Heapify costs O(m). Each of at most
`k` pops and pushes costs O(log m), so the total is **O(m + k log m)**, often written O(k log min(k, n)).
**Space:** the heap never holds more than one entry per row, so **O(m)**, plus O(k) for the output list.
The grid itself is never stored.

## Reading the trigger

Listen for **"sorted" together with "smallest k"** over a large implicit space: sums of two sorted arrays,
the k-th smallest in a sorted matrix, merging `k` sorted lists, or generating numbers in order from a few
multiplicative sources. In each, every source is already ordered, and you only ever need the front of each.

Look-alikes: a single sorted array needs no heap (just index into it); **unsorted** input with a top-k need
is a size-k heap over a scan (Top-K); and if you must output the whole merge of `k` lists, the heap still
works at O(N log k) for `N` total items, but a simple pairwise merge may be easier to explain when `k` is 2.

## Saying it in the interview

"Brute force enumerates all `n * m` pair sums and sorts, so O(n m log(n m)) time and O(n m) space, which is
wasteful when `k` is small. Since both arrays are sorted, each row of the sum grid is itself sorted, so the
problem is a k-way merge. I keep a min-heap with one candidate per row, seeded with each row's first pair.
I pop the smallest, record it, and push the next column of that row. That is `k` pops at O(log rows), so
O(k log min(k, n)) time and O(min(k, n)) heap space. I only advance one dimension so I never push a cell twice."
Edge cases to test: `k` larger than `n * m` (return everything), an empty array, duplicates in the inputs,
and `k = 1`.

Pitfalls: seeding the entire first column even when `k` is small (wasteful, though still correct), and
pushing both neighbours without a `seen` set, which produces duplicate pairs.
