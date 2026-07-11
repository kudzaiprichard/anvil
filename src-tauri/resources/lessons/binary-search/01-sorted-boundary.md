---
id: 01-sorted-boundary
unit: binary-search
subpattern: "Binary search on a sorted array"
trigger_signals:
  - "The array (or search range) is already sorted, so comparing against the midpoint tells you which half can be discarded outright."
  - "A linear scan would be O(n) and you can afford at most a handful of comparisons — you want O(log n)."
  - "You're asked for the index of a target value (or the boundary where a condition flips) inside sorted data."
worked_example: binary-search
diagram: 01-sorted-boundary.diagram.json
quiz: 01-sorted-boundary.quiz.json
practice:
  - search-a-2d-matrix
  - contains-duplicate
recap: []
follow_up:
  - "What if you needed the *first* occurrence of a repeated target instead of any match — how would the branch on nums[mid] == target need to change?"
  - "What if the sorted data lived in a 2-D grid instead of a flat array — could you still binary search it as one long sorted sequence?"
---

## The one idea

Comparing the middle element of a sorted range against your target answers one
question — *is the target to the left, to the right, or here?* — and that
single comparison lets you throw away **half the remaining range** every
time. You never have to look at most of the array.

```python
def search(nums: list[int], target: int) -> int:
    lo, hi = 0, len(nums) - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        if nums[mid] == target:
            return mid
        if nums[mid] < target:
            lo = mid + 1        # target must be to the right of mid
        else:
            hi = mid - 1        # target must be to the left of mid
    return -1
```

## Why it beats the obvious approach

A linear scan checks every element until it happens to find the target —
O(n), and in the worst case (target absent, or at the very end) it touches
the whole array. Binary search instead maintains an invariant: **if the
target is in the array, it is always inside `nums[lo..hi]`**. Each
comparison against `nums[mid]` halves that range, so after `k` steps only
`n / 2^k` candidates remain. That shrinks to a single element in about
`log2(n)` steps — 30 comparisons is enough for an array of a billion
elements.

The only requirement the technique needs is order: as long as "is this
value too small or too big?" is a well-defined question across the range,
you can binary search it — which is exactly why the *next* lesson applies
the same halving idea to something that isn't even an array.

## Reading the trigger

Say it out loud: **"sorted array, find the position of a value."** Sorted
plus "find a specific spot" is the tell. Before writing a loop that checks
every element, ask whether the data is sorted (or can cheaply be treated as
sorted) — if so, reach for the midpoint-and-discard-half move first.
