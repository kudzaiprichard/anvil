---
id: 05-two-heaps
unit: heap
subpattern: "Two heaps (running median)"
trigger_signals:
  - "You need the median (or any middle-rank value) of a collection that keeps growing, queried between insertions."
  - "Re-sorting after every insert would cost O(n log n) per query, and you only need the middle, not the full order."
  - "The data naturally splits into a \"smaller half\" and a \"larger half\" and you only care about the border between them."
worked_example: find-median-from-data-stream
diagram: 05-two-heaps.diagram.json
quiz: 05-two-heaps.quiz.json
practice:
  - sliding-window-median
  - ipo
recap:
  - 01-top-k
  - 02-heap-basics
follow_up:
  - "What if the stream is only integers in a small known range (say 0 to 100) — is there something simpler and faster than two heaps?"
  - "What if you also had to support removing an arbitrary old value, as in a sliding window — what breaks in the two-heap design?"
---

## The one idea

The median sits on the **border between the smaller half and the larger half**
of the data. You never need the halves sorted internally; you only need the
largest value of the lower half and the smallest value of the upper half. A
heap gives you exactly one extreme in O(1), so use two heaps back to back:

- a **max-heap `low`** holding the smaller half (its top is the biggest of the small ones),
- a **min-heap `high`** holding the larger half (its top is the smallest of the big ones).

Two invariants hold after every insertion:

1. **Ordering:** every element of `low` is `<=` every element of `high`, which is the same as `max(low) <= min(high)`.
2. **Balance:** the sizes differ by at most one. We keep `low` as the possibly-larger side, so `len(low) == len(high)` or `len(low) == len(high) + 1`.

With both true, the median is read straight off the tops: the top of `low` if the
count is odd, the average of the two tops if it is even.

## Why it beats the obvious approach

The brute-force design stores a list and sorts (or inserts with `bisect.insort`)
before each query. Sorting per query is O(n log n); even keeping the list sorted
costs O(n) per insert because shifting elements is linear. Over `n` operations
that is O(n^2). All of that work maintains a full order the question never uses.

Python's `heapq` only offers a **min-heap**, so the max-heap `low` stores
**negated** values: the smallest negative is the largest real number.

```python
import heapq

class MedianFinder:
    def __init__(self):
        self.low = []    # max-heap of the smaller half, stored as negatives
        self.high = []   # min-heap of the larger half

    def add_num(self, num: int) -> None:
        heapq.heappush(self.low, -num)                      # offer to the lower half
        heapq.heappush(self.high, -heapq.heappop(self.low)) # its max must be <= all of high
        if len(self.high) > len(self.low):                  # rebalance: low may lead by one
            heapq.heappush(self.low, -heapq.heappop(self.high))

    def find_median(self) -> float:
        if len(self.low) > len(self.high):                  # odd count: middle is low's top
            return float(-self.low[0])
        return (-self.low[0] + self.high[0]) / 2            # even count: average the borders
```

The push-through trick in `add_num` is what keeps the ordering invariant without
any comparisons of our own: the new value goes into `low`, then low's current
maximum (which may be the new value itself) crosses to `high`. Everything left in
`low` is therefore `<=` everything in `high`. One conditional move back restores
the size balance.

**Time:** `add_num` does at most three heap operations, each O(log n), so it is
**O(log n)**. `find_median` only peeks at one or two tops: **O(1)**. **Space:**
every value is stored once, in one of the two heaps, so **O(n)**. Nothing else
grows.

## Reading the trigger

Say the requirement out loud: **"after each insert I may be asked for the middle
value."** A running median, a "median so far", or a rank that stays a fixed
fraction of the size (the middle, or the median of a window) all point here. The
key sign is that the answer sits at a *border*, not at an extreme.

Look-alikes: if the rank is a fixed `k` (the 3rd largest, always), one size-`k`
heap is enough, as in Top-K. If you only need the median once, on a static array,
sorting or quickselect is simpler. If values come from a tiny range, a count
array beats both. Two heaps earn their keep when the stream is unbounded and the
middle moves with the size.

## Saying it in the interview

"The naive version keeps a sorted list: O(n) per insert from shifting, or
O(n log n) if I re-sort per query. The insight is that the median is the border
between the lower and upper halves, so I only need each half's nearest element to
the middle. I keep a max-heap for the lower half and a min-heap for the upper
half, with the sizes within one of each other and every low element no larger
than every high element. Insert is O(log n) from a constant number of heap
operations, the median is O(1) from the tops, and space is O(n)." Then test: a
single element, two elements (average of both), duplicates, and negative numbers.

Pitfalls interviewers watch for: forgetting that `heapq` is a min-heap and
pushing raw values into the "max-heap"; letting the sizes drift so the median
reads from the wrong top; and using integer division for an even count when the
median can end in a half.
