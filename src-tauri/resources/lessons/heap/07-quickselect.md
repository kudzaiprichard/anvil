---
id: 07-quickselect
unit: heap
subpattern: "Quickselect & sorting (kth without a heap)"
trigger_signals:
  - "You need the k-th smallest or largest of a static array, not the whole ordering and not a live stream."
  - "An interviewer asks you to beat O(n log n), or says \"can you do it with O(1) extra space?\" after you gave the heap solution."
  - "The prompt says to implement a sort, or asks you to compare sorting strategies by time, space and stability."
worked_example: kth-largest-element-in-an-array
diagram: 07-quickselect.diagram.json
quiz: 07-quickselect.quiz.json
practice:
  - sort-an-array
  - k-closest-points-to-origin
recap:
  - 01-top-k
  - 03-k-closest
follow_up:
  - "What if the array were far too large for memory and arrived as a stream — which of the three approaches survives?"
  - "What if you needed the k largest elements in sorted order, not just the k-th — how does the cost change?"
---

## The one idea

Sorting finds every element's final position, but the k-th largest needs only **one** position.
Quickselect takes quicksort's partition step and throws half the work away: pick a pivot, rearrange
so that smaller values sit left of it and larger values sit right, and then recurse into **only the
side that contains the target index**. The other side is never touched.

The invariant after a partition is what makes the discard safe: the pivot (and any equal values) are
already in their **final sorted positions**, everything to the left is no larger, and everything to the
right is no smaller. If the target index falls inside the pivot's block, you are done; otherwise you
know exactly which side holds it.

For the k-th **largest** in an array of length `n`, the target is index `n - k` in ascending order.

## Why it beats the obvious approach

Interviewers like a ladder, and you should be able to climb it out loud:

1. **Sort, then index:** O(n log n) time, whatever the space of your sort. Correct but does more than asked.
2. **Size-k min-heap:** O(n log k) time, O(k) space, and it works on streams. Good when `k` is small.
3. **Quickselect:** O(n) time on average, worst case O(n^2) if pivots are consistently terrible, and
   O(1) extra space when written as a loop. The average is n + n/2 + n/4 + ... which sums to 2n.

```python
import random

def find_kth_largest(nums: list[int], k: int) -> int:
    target = len(nums) - k                  # index of the answer in ascending order
    lo, hi = 0, len(nums) - 1
    while True:
        pivot = nums[random.randint(lo, hi)]        # random pivot defeats adversarial inputs
        lt, i, gt = lo, lo, hi                      # [lo,lt) < pivot, [lt,i) == pivot, (gt,hi] > pivot
        while i <= gt:
            if nums[i] < pivot:
                nums[lt], nums[i] = nums[i], nums[lt]; lt += 1; i += 1
            elif nums[i] > pivot:
                nums[i], nums[gt] = nums[gt], nums[i]; gt -= 1   # do not advance i: new value unseen
            else:
                i += 1
        if target < lt:                             # answer is among the smaller values
            hi = lt - 1
        elif target > gt:                           # answer is among the larger values
            lo = gt + 1
        else:                                       # target lies in the equal block: done
            return pivot
```

This is a **3-way (Dutch flag) partition**. The simpler Lomuto partition splits into only "less than"
and "at least"; it is easier to write but degrades to O(n^2) when many elements equal the pivot, because
the equal block is never excluded. Three regions fix that: all copies of the pivot are settled at once,
so an array of identical values finishes in one pass.

Correctness never depends on randomness: any pivot value gives a valid partition and the loop shrinks
`[lo, hi]` each round (the pivot block is removed). Randomness only protects the running time.

**Time:** each round scans the current range once, O(range). A random pivot lands in the middle half
with probability about one half, so the expected range sizes shrink geometrically: O(n) expected. An
unlucky run is O(n^2) but vanishingly unlikely. **Space:** the loop uses a few indices, so **O(1)** extra.
(A recursive version would add recursion depth, expected O(log n), worst O(n).) Note that the input
array is **rearranged in place**; copy it first if the caller needs it untouched.

## Reading the trigger

**"k-th largest/smallest of a fixed array"** is the signal, and the follow-up pressure ("faster than
sorting", "constant extra memory") is what pushes you from the heap to quickselect. If the data is a
stream, or `k` is tiny and you must not mutate the input, the size-k heap wins. If you need the top `k`
**in order**, sort or heap-then-sort, since quickselect leaves the left part unordered.

When the task is simply "implement sort", know the two main answers. **Merge sort:** O(n log n) always,
stable, but O(n) extra space for merging. **Quicksort:** O(n log n) average, O(n^2) worst, in place
(O(log n) stack), usually faster in practice because of cache locality, and not stable. Heap sort gives
O(n log n) worst case and O(1) space but is not stable and has poor locality.

## Saying it in the interview

"Sorting works in O(n log n), but we only need one rank. A size-k min-heap drops that to O(n log k) time
and O(k) space and also handles streams. For a static array I can do better with quickselect: partition
around a random pivot so smaller values are left and larger values right, then continue into only the
side containing index `n - k`. Expected time is O(n) because the range roughly halves each round, worst
case O(n^2) if pivots are bad, which a random pivot makes very unlikely. It is iterative, so O(1) extra
space, though it mutates the array." Mention the 3-way partition for duplicates, and test: `k = 1`,
`k = n`, one element, and all-equal values.

Pitfalls: converting `k` to the wrong index (for largest it is `n - k`, not `k - 1`), using a plain
two-way partition on a heavily duplicated array and silently degrading to quadratic time, and advancing
`i` after swapping with `gt`, which skips an unexamined value.
