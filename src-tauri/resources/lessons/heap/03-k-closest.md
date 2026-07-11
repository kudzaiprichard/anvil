---
id: 03-k-closest
unit: heap
subpattern: "K-closest / K-smallest by heap"
trigger_signals:
  - "You need the k elements closest to (or farthest from) a target — by some computed distance or score — not the full sorted order."
  - "Ranking every element and slicing the top k with a full sort is wasteful when k is much smaller than n."
  - "You want O(n log k) using a bounded heap instead of O(n log n) for a full sort."
worked_example: k-closest-points-to-origin
diagram: 03-k-closest.diagram.json
quiz: 03-k-closest.quiz.json
practice:
  - kth-largest-element-in-an-array
  - top-k-frequent-elements
recap:
  - 01-top-k
follow_up:
  - "What if k were close to n — is a full sort ever cheaper than the bounded heap?"
  - "What if the distance metric were expensive to compute — would you cache it inside the heap entries?"
---

## The one idea

Finding the k closest points is the k-smallest twin of "top-k with a
heap": keep a heap bounded to size k, but this time keyed by *distance*
rather than raw value, and evict the farthest point whenever the heap
grows past k. Because you want the k *smallest* distances and `heapq` is a
min-heap, negate the distance so the farthest survivor sits at the root —
exactly the one you want to evict.

```python
import heapq

def k_closest(points: list[list[int]], k: int) -> list[list[int]]:
    heap = []  # max-heap of (-distance, x, y), bounded to size k
    for x, y in points:
        dist = x * x + y * y
        heapq.heappush(heap, (-dist, x, y))
        if len(heap) > k:
            heapq.heappop(heap)   # evict the current farthest
    return [[x, y] for _, x, y in heap]
```

## Why it beats the obvious approach

Sorting all n points by distance and slicing the first k is O(n log n),
and it computes an ordering you don't actually need — you only care *which*
k are closest, not their relative order among themselves. A size-k heap
tracks exactly the boundary: while fewer than k points have been seen,
everything stays; once you're over k, only a point closer than the current
worst survivor is worth keeping, and the heap reports that worst survivor
in O(1) (its root) and evicts it in O(log k). Total cost is O(n log k) — a
real win whenever k is much smaller than n, which is exactly when
"k closest / k smallest" problems show up.

## Reading the trigger

Say it: **"k elements closest to a target by some distance or score, k much
smaller than n."** Whenever the question asks for a bounded best-k subset
ranked by a *computed* key — distance to the origin, difference from a
value, custom score — reuse the size-k heap idea from the top-k lesson;
just swap the key you push on.
