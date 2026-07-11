---
id: 03-rotated-2d
unit: binary-search
subpattern: "Searching rotated & 2-D arrays"
trigger_signals:
  - "The array was sorted but then rotated at an unknown pivot — it's no longer globally sorted, but whichever half you're looking at, at least one side of it is still cleanly sorted."
  - "You're searching a 2-D matrix where every row is sorted left-to-right and every row starts where the previous one ended — the whole grid behaves like one long sorted sequence."
  - "A linear scan still works but is O(n) (or O(rows * cols) for a grid); you can identify which half of the current range is genuinely sorted and use its endpoints to decide which half to keep."
worked_example: find-minimum-in-rotated-sorted-array
diagram: 03-rotated-2d.diagram.json
quiz: 03-rotated-2d.quiz.json
practice:
  - search-a-2d-matrix
  - koko-eating-bananas
recap:
  - 01-sorted-boundary
follow_up:
  - "What if you needed to find a specific *target* value in the rotated array, not just the minimum — how would you combine 'which half is sorted' with a normal target comparison?"
  - "What if the array contained duplicate values — could nums[mid] == nums[hi] ever make it impossible to tell which half is sorted?"
---

## The one idea

Rotating a sorted array breaks the *whole-array* sort, but it can't break
every half at once: split at `mid`, and **at least one of the two halves is
still internally sorted**, end to end. Comparing `nums[mid]` against an
endpoint tells you which half that is — and the rotation point (the
minimum) must live in the *other*, still-broken half.

```python
def find_min(nums: list[int]) -> int:
    lo, hi = 0, len(nums) - 1
    while lo < hi:
        mid = (lo + hi) // 2
        if nums[mid] > nums[hi]:
            lo = mid + 1        # rotation point is to the right of mid
        else:
            hi = mid             # mid..hi is already sorted; keep looking left
    return nums[lo]
```

## Why it beats the obvious approach

A linear scan for the smallest element is O(n) — it has no way to skip
anything, since it doesn't know where the rotation happened. But checking
`nums[mid]` against `nums[hi]` costs one comparison and tells you which
half is sorted: if `nums[mid] > nums[hi]`, the drop-off (and the minimum)
is somewhere in `mid+1..hi`; otherwise `mid..hi` is already increasing, so
the minimum is at `mid` or to its left. Either way, half the range is
eliminated, giving O(log n) — the same halving guarantee from lesson one,
just driven by "which side is sorted" instead of "which side holds the
target."

The 2-D matrix case (row-sorted, and each row continues from the last) is
the same idea one layer up: treat the grid as one flattened sorted array of
length `rows * cols`, map `mid -> (mid // cols, mid % cols)`, and binary
search it exactly as in lesson one.

## Reading the trigger

Say it out loud: **"sorted, but rotated"** or **"sorted rows stacked into a
grid."** Either phrase means you still have order to exploit — you just
have to identify *which* half of the current range is the clean, sorted
one before deciding where the answer must be hiding.
