---
id: 04-set-membership
unit: arrays-hashing
subpattern: "Set membership & deduplication"
trigger_signals:
  - "The question only cares WHETHER something exists or repeats — not what value goes with it, not its position."
  - "You catch yourself wanting to say \"if I've seen this before\" with no need to remember anything else about it."
  - "You need to deduplicate a collection, or check a batch of \"no two X can collide\" constraints (rows, columns, boxes)."
worked_example: contains-duplicate
diagram: 04-set-membership.diagram.json
quiz: 04-set-membership.quiz.json
practice:
  - valid-sudoku
  - two-sum
follow_up:
  - "What if you needed to know *how many times* each value repeats, not just whether it repeats — does a set still suffice?"
  - "What if duplicates only count within a sliding window of size k — how would you shrink the set as the window moves?"
---

## The one idea

When a problem only asks a **yes/no membership** question — "does this value
already exist?", "have any two collided?" — you don't need a hash map's
key-to-value pairing at all. A `set` is a hash map stripped down to just the
keys: O(1) average `in` checks and O(1) inserts, with none of the bookkeeping
for values you'll never use.

## Why it beats the obvious approach

The brute-force way to check "does this array contain a duplicate?" compares
every pair — two nested loops, O(n²). A sorting-based fix gets you to
O(n log n) by putting equal values next to each other, but that still costs
more than necessary and reorders the input. A `set` gets there in one linear
pass:

```python
def contains_duplicate(nums: list[int]) -> bool:
    seen: set[int] = set()
    for x in nums:
        if x in seen:        # O(1): have we already placed this value?
            return True
        seen.add(x)
    return False
```

Compare this to `set(nums)` and checking `len(set(nums)) != len(nums)` — same
idea, expressed as a single deduplication instead of an early-exit scan. The
set is doing exactly one job: remembering *which* values have been placed,
with no index, count, or complement attached to them. That's the tell that
distinguishes this from the hash-map-lookup pattern — there, you stored a
value *because* you'd need to report something else about it (an index, a
count). Here, existence is the entire answer.

The same trick scales to "no two entries may collide" constraints across
multiple groups at once — for instance, checking that no row, column, or
3x3 box repeats a digit. You keep one set per group, insert as you scan, and
reject the moment an `in` check succeeds where it shouldn't.

## Reading the trigger

Ask: *"if I answer 'seen it' or 'not seen it,' am I done?"* If the problem
never needs to look anything up **by** the value afterward — no index to
return, no paired data to fetch — a `set` is leaner than a `dict` and signals
your intent more clearly to a reader. Reach for it whenever the task reduces
to "no duplicates allowed" or "have I already placed this."
