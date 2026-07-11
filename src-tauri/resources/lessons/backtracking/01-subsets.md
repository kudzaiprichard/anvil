---
id: 01-subsets
unit: backtracking
subpattern: "Subsets: the include/exclude choice tree"
trigger_signals:
  - "You need every subset (or combination) of a set, not just one answer — the output is a collection of collections."
  - "Order doesn't matter within a chosen group, but for each element you must decide: include it or don't."
  - "n is small (roughly n <= 20) because the answer has up to 2^n entries — this is exhaustive search, not a polynomial algorithm."
worked_example: subsets
diagram: 01-subsets.diagram.json
quiz: 01-subsets.quiz.json
practice:
  - subsets-ii
  - letter-combinations-of-a-phone-number
recap: []
follow_up:
  - "What if the input has duplicate values — how would you avoid emitting the same subset twice?"
  - "What if you only wanted subsets of a fixed size k, not every size at once?"
---

## The one idea

Backtracking builds every answer by walking a choice tree one element at a
time, using the same three-step template at every level: **choose**, recurse
into everything that follows, then **un-choose** (undo) and try the other
branch. Subsets is the simplest instance of that template: each element gets
one independent include/exclude decision, so `n` elements produce `2^n`
leaves — one for every possible subset.

```python
def subsets(nums: list[int]) -> list[list[int]]:
    result = []
    path = []

    def backtrack(i: int) -> None:
        if i == len(nums):
            result.append(path[:])   # snapshot — path keeps mutating
            return
        path.append(nums[i])         # choose: include nums[i]
        backtrack(i + 1)
        path.pop()                   # un-choose: try excluding nums[i]
        backtrack(i + 1)

    backtrack(0)
    return result
```

## Why it beats the obvious approach

There's no clever pruning here — the point of this lesson is the *shape* of
the recursion, not a shortcut. What makes it work is discipline about state:
`path` is a single mutable list that's shared across the whole recursion tree,
not a fresh list per branch. That's cheap (`O(n)` extra memory instead of
copying a list at every call), but it means every leaf must be recorded as a
**copy** (`path[:]`), because `path` keeps changing after you record it — if
you appended `path` itself, every entry in `result` would end up pointing at
the same, eventually-empty list.

The `backtrack(i + 1)` call appears twice: once right after `append` (the
"include" branch) and once right after `pop` (the "exclude" branch). Two
calls per level, `n` levels deep, is exactly `2^n` leaves — so the whole walk
is `O(n * 2^n)`: `2^n` subsets, each costing `O(n)` to copy into `result`.

## Reading the trigger

Say it out loud: **"every subset of a set, not just one — and each element is
either in or out."** Whenever the answer is a *collection of collections*
built from independent inclusion decisions over a small input, backtracking
with an include/exclude choice per index is the first instinct — before you
reach for loops that try to enumerate subsets iteratively (which gets messy
fast) or a DP table (which doesn't apply — there's no optimal substructure to
exploit, just an exhaustive listing).
