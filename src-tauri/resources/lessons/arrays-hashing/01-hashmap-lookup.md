---
id: 01-hashmap-lookup
unit: arrays-hashing
subpattern: "Hash-map complement lookup"
trigger_signals:
  - "You need to answer \"have I already seen X?\" in O(1) — membership, not order."
  - "You're pairing elements (a + b = target) and the array is unsorted."
  - "A brute-force scan is O(n²) and you want to trade memory for a single pass."
worked_example: two-sum
diagram: 01-hashmap-lookup.diagram.json
quiz: 01-hashmap-lookup.quiz.json
practice:
  - contains-duplicate
  - valid-anagram
recap: []
follow_up:
  - "What if the array were already sorted — could you drop the hash map and use two pointers instead?"
  - "What if you had to return *all* pairs that sum to the target, not just one?"
---

## The one idea

A hash map turns the question **"have I seen the number I need?"** from a scan
into a single O(1) lookup. That one trade — spend memory to remember what
you've passed — collapses a whole family of "find the pair / find the
duplicate / match by key" problems from O(n²) down to O(n).

In Python the map is a plain `dict`. As you walk the array once, you record
each value you've passed; before recording, you first *ask the map* whether the
partner you're looking for is already there.

## Why it beats the obvious approach

The brute-force answer to *"do two numbers add up to `target`?"* is to try every
pair — two nested loops, O(n²). That work is wasted: for the value `x` at the
current index, there is exactly **one** number that completes it, its complement
`target - x`. You don't need to search for that complement; you just need to
remember every number you've already seen and check membership.

```python
def two_sum(nums: list[int], target: int) -> list[int]:
    seen: dict[int, int] = {}          # value -> the index we saw it at
    for i, x in enumerate(nums):
        need = target - x              # the one partner that completes x
        if need in seen:               # O(1): have we passed it already?
            return [seen[need], i]
        seen[x] = i                    # remember x for a later element
    return []
```

Two things make this work, and they're the same two things in every problem of
this shape:

1. **The map key is what you'll later look *up* by** — here, the value, because
   a future element asks "is my complement one of the values I've seen?"
2. **The map value is what you need to *report*** — here, the index, so the
   answer carries positions rather than the numbers themselves.

Notice we never sort. Sorting is O(n log n) and, worse, it scrambles the
original indices the problem asks us to return. The hash map keeps the input
order intact and still runs in one pass.

## Reading the trigger

The skill this lesson trains isn't the code — it's *recognizing* when to reach
for it. Say the problem out loud: **"unsorted array, find a pair that sums to a
target, return their positions."** The phrase "have I already seen the partner I
need?" is the tell. Whenever a problem reduces to fast membership or
match-by-key on an unordered collection, the hash map is your first instinct —
before you write a single nested loop.
