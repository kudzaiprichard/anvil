---
id: 02-sorted-pair-sum
unit: two-pointers
subpattern: "Two pointers on a sorted array"
trigger_signals:
  - "The array is already sorted (or cheap to sort) and you need to find/verify a pair matching a sum or comparison target."
  - "You know the opposite-ends convergence idea; here each pointer's move is decided by comparing the current pair against the target, not by symmetry."
  - "Brute force tries every pair in O(n^2); you want O(n) after at most an O(n log n) sort, with O(1) extra space instead of a hash map."
worked_example: two-sum-ii-input-array-is-sorted
diagram: 02-sorted-pair-sum.diagram.json
quiz: 02-sorted-pair-sum.quiz.json
practice:
  - valid-palindrome
  - contains-duplicate
recap:
  - 01-opposite-ends
follow_up:
  - "What if you needed all pairs that sum to the target, not just one — how would you keep the pointers moving without missing or repeating a pair?"
  - "What if the array had duplicate values — does the two-pointer scan still visit every distinct pair exactly once?"
---

## The one idea

Same two starting positions as the last lesson — `lo` at the front, `hi` at
the back — but now the direction each pointer moves isn't decided by
symmetry, it's decided by **comparing the current pair's sum to the
target**. Because the array is sorted, that comparison tells you exactly
which whole range of pairs you can safely discard.

```python
def two_sum_sorted(numbers: list[int], target: int) -> list[int]:
    lo, hi = 0, len(numbers) - 1
    while lo < hi:
        total = numbers[lo] + numbers[hi]
        if total == target:
            return [lo + 1, hi + 1]      # 1-indexed positions
        if total < target:
            lo += 1                      # sum too small -> drop the smallest value
        else:
            hi -= 1                      # sum too big -> drop the largest value
    return []
```

## Why it beats the obvious approach

If the sum is too small, raising `hi` can't help — `numbers[hi]` is already
the biggest value left, so pairing it with anything smaller than
`numbers[lo]` only makes the sum smaller. The only pointer that can possibly
fix an undersized sum is `lo`, moving right to a bigger value. Symmetric
logic prunes the other direction. Every pointer move eliminates one entire
candidate pair from further consideration, so the whole scan is O(n) time
and **O(1) extra space** — no hash map needed.

That's the trade this lesson makes explicit: without sortedness, "find a
pair summing to target" reaches for a hash map (recall the arrays-hashing
unit) at O(n) space. *With* sortedness, the same question is answered by two
pointers at O(1) space — the order in the array does the work the hash map
used to do.

## Reading the trigger

Say it out loud: **"sorted array, find a pair matching a target."** Sorted
plus pair-target is the tell — before reaching for a hash map, check whether
the array is already sorted (or cheap to sort); if so, two converging
pointers driven by the sum comparison will get you there in one pass with no
extra memory.
