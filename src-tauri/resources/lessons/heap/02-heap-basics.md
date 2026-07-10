---
id: 02-heap-basics
unit: heap
subpattern: "Heap as a running extremum"
trigger_signals:
  - "You repeatedly need \"the current max (or min)\" after removals and insertions — not a one-time max, a *running* one."
  - "The two largest (or smallest) elements interact each round, and the result feeds back into the same pool (smash, merge, pick again)."
  - "A plain scan for the max each round is O(n) per round, O(n^2) total; you want O(log n) per round instead."
worked_example: last-stone-weight
diagram: 02-heap-basics.diagram.json
quiz: 02-heap-basics.quiz.json
practice:
  - k-closest-points-to-origin
  - kth-largest-element-in-an-array
recap:
  - 01-top-k
follow_up:
  - "What if you needed the running *minimum* instead of the maximum — does negating still work cleanly?"
  - "What if ties should break by insertion order — does the heap need extra bookkeeping?"
---

## The one idea

Python's `heapq` is a min-heap only. To track a running **maximum** — as in
"keep smashing the two heaviest stones together" — negate every value on
the way in and negate it again on the way out. The heap root is then always
the current largest.

```python
import heapq

def last_stone_weight(stones: list[int]) -> int:
    heap = [-s for s in stones]
    heapq.heapify(heap)
    while len(heap) > 1:
        a = -heapq.heappop(heap)   # heaviest
        b = -heapq.heappop(heap)   # second heaviest
        if a != b:
            heapq.heappush(heap, -(a - b))
    return -heap[0] if heap else 0
```

## Why it beats the obvious approach

The naive version re-sorts (or re-scans for the two largest) on every
round — O(n log n) or O(n) work, repeated up to n times, so O(n^2) overall
in the worst case. A heap turns "give me the current max, remove it, and
maybe re-insert a derived value" into two O(log n) operations per round.
Because a heap re-balances itself after every push/pop, it never needs to
know anything about the *rest* of the elements — finding the max is a
structural property of the heap, not something you compute by scanning it.

This is the same trick from the last lesson, generalized: there you fixed
the heap's *size* at k to track a boundary value; here you let the heap
hold everything and use it purely as a fast "pop the extremum, push a
derived value back" engine.

## Reading the trigger

Say it: **"repeatedly combine or compare the two biggest (or smallest),
feed the result back in."** Whenever a round-based process keeps asking for
the current extremum and then mutates the pool — smash two stones, merge
two runs, pick and requeue — a heap gives you that extremum in O(log n)
instead of an O(n) rescan every single round.
