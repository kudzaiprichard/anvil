---
id: 02-answer-space
unit: binary-search
subpattern: "Binary search on the answer space"
trigger_signals:
  - "You're asked for the minimum (or maximum) feasible value of some parameter — a speed, a capacity, a duration — not an index into an array."
  - "There's a monotonic feasibility check: if some value k works, every larger (or every smaller) k also works, so the yes/no answers form a sorted boundary even though nothing is stored in an actual array."
  - "Brute force would try every candidate value one at a time and re-check feasibility from scratch; you'd rather prune half the candidate range on each check."
worked_example: koko-eating-bananas
diagram: 02-answer-space.diagram.json
quiz: 02-answer-space.quiz.json
practice:
  - find-minimum-in-rotated-sorted-array
  - search-a-2d-matrix
recap:
  - 01-sorted-boundary
follow_up:
  - "What if the feasibility check itself were expensive (say, O(n log n)) — would binary searching the answer still beat trying every candidate value linearly?"
  - "What if you needed the *maximum* feasible value instead of the minimum — which comparison in the loop would flip?"
---

## The one idea

Binary search doesn't need a physical array at all — it needs a **range of
candidate answers** and a feasibility check that flips from "no" to "yes"
(or vice versa) exactly once as the candidate increases. That monotonic
flip is a sorted boundary in disguise, so you can hunt it with the same
midpoint-and-discard-half move from the last lesson, just applied to values
instead of positions.

```python
def min_eating_speed(piles: list[int], h: int) -> int:
    def hours_needed(k: int) -> int:
        return sum((p + k - 1) // k for p in piles)   # ceil(p / k) per pile

    lo, hi = 1, max(piles)
    while lo < hi:
        mid = (lo + hi) // 2
        if hours_needed(mid) <= h:   # mid is fast enough -> maybe go slower
            hi = mid
        else:                        # mid is too slow -> must go faster
            lo = mid + 1
    return lo
```

## Why it beats the obvious approach

Trying every speed `k = 1, 2, 3, ...` and re-checking feasibility each time
costs O(n * max(piles)) in the worst case. But "can Koko finish in time at
speed k?" is **monotonic**: any speed faster than a working speed also
works. That's the same property sortedness gives an array — a clean
boundary between "no" and "yes" — so binary searching the range
`[1, max(piles)]` finds the smallest working speed in O(log(max(piles)))
checks, each costing O(n), for O(n log(max(piles))) overall.

This is the generalization the last lesson was building toward: whenever a
problem asks for the smallest (or largest) value satisfying some
monotonic condition, that condition — not a sorted array — is what you
binary search.

## Reading the trigger

Say it out loud: **"minimum (or maximum) value such that some condition
holds."** Whenever increasing a candidate can only turn a "no" into a "yes"
(never back into a "no"), stop thinking about trying candidates one at a
time — binary search the range of candidates using that feasibility check
as the comparison.
