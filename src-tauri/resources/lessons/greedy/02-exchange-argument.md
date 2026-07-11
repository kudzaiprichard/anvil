---
id: 02-exchange-argument
unit: greedy
subpattern: "Exchange-argument greedy"
trigger_signals:
  - "You must pick one starting point / threshold / item order, and a naive check would test each candidate independently in O(n^2)."
  - "You can prove that if candidate A fails, every candidate 'dominated' by A (weaker or later, by the same measure) fails too — so a whole batch can be skipped at once, not tested one by one."
  - "A single left-to-right scan with a running total lets you compare 'the greedy choice' against 'swapping to any other choice' and show the swap never helps."
worked_example: gas-station
diagram: 02-exchange-argument.diagram.json
quiz: 02-exchange-argument.quiz.json
practice:
  - jump-game
  - hand-of-straights
recap:
  - 01-reachability
follow_up:
  - "What if gas and cost arrived as a stream and you couldn't store the whole array — can you still find the start in one pass?"
  - "How would you prove that at most one valid starting station exists whenever a solution does?"
---

## The one idea

Scan once, keeping a running `tank` (surplus since the current candidate
start) and a running `total` (surplus over the whole trip). Whenever `tank`
drops below zero, the current candidate start is dead — and so is every
station between it and here — so jump the candidate straight to the next
index and reset `tank` to 0. At the end, `total >= 0` tells you a full
circuit exists at all, and `start` tells you where.

```python
def can_complete_circuit(gas: list[int], cost: list[int]) -> int:
    total = tank = start = 0
    for i, (g, c) in enumerate(zip(gas, cost)):
        diff = g - c
        total += diff
        tank += diff
        if tank < 0:            # every station in [start, i] is dead
            start = i + 1
            tank = 0
    return start if total >= 0 else -1
```

## Why it beats the obvious approach

Testing each of the `n` stations by simulating the full circuit from
scratch is O(n^2). The **exchange argument** cuts that: suppose starting at
`start` first goes negative at station `i`. Every station `k` strictly
between `start` and `i` had a *nonnegative* running total measured from
`start` (otherwise the tank would already have failed at `k`). Starting at
`k` instead throws away that nonnegative head start — by the time you reach
`i`, a start at `k` has strictly less accumulated surplus than a start at
`start` did, so it fails too, or no later than `start` failed. Swapping to
any station in `[start, i]` never helps; they're all dominated by `start`
having already been tried. That licenses skipping the whole block in one
motion instead of retrying each station — one O(n) pass, no revisits.

This is the same move as the last lesson's farthest-reach tracking: discard
whatever's provably dominated, keep only the one running number that
matters.

## Reading the trigger

Say it out loud: **"pick a starting point so a running balance never goes
negative."** Whenever failing at one candidate lets you prove a whole
contiguous block of candidates fails identically — not "probably," but
*provably*, by comparing accumulated surplus — reach for a single scan with
a reset-on-negative running total instead of re-simulating from every
candidate.
