---
id: 02-combinations
unit: backtracking
subpattern: "Combination sum with pruning"
trigger_signals:
  - "You need every combination of numbers (possibly reused) that sums exactly to a target — not just whether one exists."
  - "Elements can repeat within a combination, and you must avoid emitting the same multiset in a different order — sort first, then only look forward from a `start` index."
  - "The running sum lets you prune whole branches early: once it can't reach (or already exceeds) the target, stop exploring that path."
worked_example: combination-sum
diagram: 02-combinations.diagram.json
quiz: 02-combinations.quiz.json
practice:
  - combination-sum-ii
  - palindrome-partitioning
recap:
  - 01-subsets
follow_up:
  - "What if each number could only be used once — how would the recursive call change from `backtrack(i, ...)` to `backtrack(i + 1, ...)`?"
  - "What if the input had duplicate numbers and you had to skip duplicate combinations at the same recursion depth?"
---

## The one idea

Same choose -> recurse -> un-choose template as subsets, but two things
change. First, instead of a binary include/exclude decision per index, the
loop chooses *which* candidate to add next, and can choose the **same**
index again (reuse). Second, you carry a running `remaining` total so you can
**prune**: stop exploring a branch the moment sorted order tells you it can
no longer reach the target.

```python
def combination_sum(candidates: list[int], target: int) -> list[list[int]]:
    candidates.sort()                 # smallest first, so we can prune early
    result = []
    path = []

    def backtrack(start: int, remaining: int) -> None:
        if remaining == 0:
            result.append(path[:])
            return
        for i in range(start, len(candidates)):
            if candidates[i] > remaining:   # sorted -> everything after is worse too
                break
            path.append(candidates[i])
            backtrack(i, remaining - candidates[i])   # i, not i + 1: reuse allowed
            path.pop()

    backtrack(0, target)
    return result
```

## Why it beats the obvious approach

Without sorting, "this candidate is already too big" only shows up as a
wasted recursive call that fails at a deeper base case. Sorting turns that
into a `break` right where it's discovered: once `candidates[i] > remaining`,
every candidate after `i` is at least as large (the array is sorted), so the
*entire rest of the loop* can be skipped in one step, not just this one
candidate.

The `start` parameter does a second job: passing `i` (not `0`) into the
recursive call means the search only ever looks **forward** from the current
position, so `[2, 3]` is built once, never also as `[3, 2]`. Passing `i`
itself (not `i + 1`) is what allows reuse — the next pick can be the same
candidate again, which is exactly why `2 + 2 + 3 = 7` and `2 + 2 + 2 + 1 = 7`
are both reachable from `2`.

Compare to the subsets lesson: there, every level makes exactly one
include/exclude call each way, so the tree's shape only depends on `n`. Here
the branching factor and depth both depend on `target` and the candidate
values, so the honest complexity claim is that the search is exponential in
the worst case — pruning shrinks it a lot in practice, but it doesn't change
the asymptotic bound.

## Reading the trigger

Say it out loud: **"every combination of numbers, values may repeat, that
sums to exactly a target."** Whenever "combination" and "target sum" appear
together with the option to reuse values, backtracking with a running-sum
prune is the play: sort first, choose an index, recurse either at the same
index (reuse) or the next one (no reuse), and `break` out of the loop the
instant sorted order proves the rest can't fit.
