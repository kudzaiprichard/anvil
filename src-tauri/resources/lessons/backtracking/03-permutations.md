---
id: 03-permutations
unit: backtracking
subpattern: "Permutations & the used-set"
trigger_signals:
  - "You need every ordering (arrangement) of a set of distinct elements — order matters, unlike subsets or combinations."
  - "Each element must appear exactly once per output, so you need to track which elements are already placed."
  - "n is small (roughly n <= 10) because the answer has n! entries — this is exhaustive search, not a polynomial algorithm."
worked_example: permutations
diagram: 03-permutations.diagram.json
quiz: 03-permutations.quiz.json
practice:
  - subsets-ii
  - combination-sum-ii
recap:
  - 01-subsets
follow_up:
  - "What if the input has duplicate values — how would you avoid emitting the same permutation twice?"
  - "What if you only wanted permutations of a fixed length k < n, not a full arrangement of everything?"
---

## The one idea

Subsets and combination sum both decide, index by index, whether to include
a value going *forward*. Permutations flips that: at each position of the
output, the loop asks *which still-unused element* goes there, trying every
candidate fresh at every level. That "which one is free?" question is
tracked with a `used` set: choose an unused element, recurse, then un-choose
it by popping it off `path` and removing it from `used`, so a sibling branch
can try it next.

```python
def permute(nums: list[int]) -> list[list[int]]:
    result = []
    path = []
    used = set()

    def backtrack() -> None:
        if len(path) == len(nums):
            result.append(path[:])
            return
        for x in nums:
            if x in used:
                continue
            used.add(x)               # choose
            path.append(x)
            backtrack()
            path.pop()                 # un-choose
            used.remove(x)

    backtrack()
    return result
```

## Why it beats the obvious approach

Subsets makes one binary decision per index, giving `2^n` leaves.
Permutations makes `n` decisions, and at *each* of those `n` positions there
are up to `n` candidates to try — that's `n!` leaves, not `2^n`. The `used`
set is what keeps each of those position-level checks cheap: without it,
"has this value already been placed?" would mean scanning `path` — an
`O(n)` check repeated for every candidate at every level. A set makes that
check `O(1)`, so the search does `O(n)` work per leaf just to build and copy
`path`, for a total of `O(n * n!)`.

The other detail worth noticing: unlike combination sum, this loop always
starts over from the beginning of `nums` at every level (`for x in nums`,
not `for i in range(start, ...)`). Order genuinely matters here, so there's
no `start` index to keep the search moving forward — every still-unused
value is a legal next choice, from any position.

## Reading the trigger

Say it out loud: **"every arrangement of a set of distinct elements — order
matters, each value used exactly once."** Whenever a problem says
"permutation" or "arrangement" (not "subset" or "combination") and elements
can't repeat within a single answer, that's the signal to backtrack with a
used-set: loop over all candidates fresh at each level, skip the ones
already placed, and undo both `path` and `used` together when backing out.
