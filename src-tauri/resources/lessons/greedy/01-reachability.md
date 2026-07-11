---
id: 01-reachability
unit: greedy
subpattern: "Greedy reachability (jump game)"
trigger_signals:
  - "The question is yes/no reachability — can you get from the start to the end (or to any point) — given a per-position 'max step' capacity."
  - "Every choice only ever helps or is neutral: taking the biggest reach available at each step never rules out an option a smaller reach would have kept open."
  - "A DP that marks every index's reachability individually is O(n) memory and revisits work; you suspect one running number is enough."
worked_example: jump-game
diagram: 01-reachability.diagram.json
quiz: 01-reachability.quiz.json
practice:
  - maximum-subarray
  - jump-game-ii
recap: []
follow_up:
  - "What if the question weren't yes/no but asked for the *minimum number of jumps* to reach the end — how would you adapt the same scan? (jump-game-ii)"
  - "What if a handful of positions could be marked unusable — does tracking only the farthest reach still work?"
---

## The one idea

Walk the array once and track a single number: the **farthest index reachable
so far**. At each position `i` — as long as `i` is still within that reach —
extend it: `farthest = max(farthest, i + nums[i])`. You never need to
remember *which* earlier position produced that reach, only *how far* it
got. If you ever land on an index past the current farthest, nothing before
it could have jumped over the gap either, so the end is unreachable.

```python
def can_jump(nums: list[int]) -> bool:
    farthest = 0
    for i, step in enumerate(nums):
        if i > farthest:               # this index was never reachable
            return False
        farthest = max(farthest, i + step)
    return farthest >= len(nums) - 1
```

## Why it beats the obvious approach

A DP table `reachable[i]` — true if some earlier reachable `j` had
`j + nums[j] >= i` — works, but it stores a fact for every index and
degrades to O(n^2) if you naively check "any earlier j" instead of reusing
work. The greedy insight is an exchange argument: reachable indices always
form one unbroken prefix `[0, farthest]`. If index `k < farthest` is
reachable, so is every index between it and `farthest` — you can always
choose to under-jump from `k`, so `k` never lets you reach somewhere
`farthest` doesn't already cover. That means `k`'s exact position is
*dominated* information once you know `farthest`; keeping it around buys
nothing. Discard it, keep only the maximum, and the whole table collapses
into one running variable — O(n) time, O(1) space, single pass.

## Reading the trigger

Say the shape out loud: **"per-position max-step capacity, can you reach the
far end?"** That's the tell for greedy reachability — before writing a DP
table indexed by position, ask whether every reachable index is dominated by
the single farthest one seen so far. If yes, one forward scan replaces the
whole table. The same "track the extreme value, discard everything it
dominates" move reappears through the rest of this unit under different
names — a running max, a running deficit, a running total.
