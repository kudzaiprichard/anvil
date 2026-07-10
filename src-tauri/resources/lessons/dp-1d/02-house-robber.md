---
id: 02-house-robber
unit: dp-1d
subpattern: "Non-adjacent selection (house robber)"
trigger_signals:
  - "You must pick a subset of a sequence to maximize (or minimize) a sum, but picking two adjacent elements is forbidden."
  - "The recurrence needs a 'take it or skip it' decision at each position, not just a running total."
  - "dp[i] depends on dp[i-1] (skip current) and dp[i-2] + value[i] (take current) — Fibonacci's lookback with a choice layered on top."
worked_example: house-robber
diagram: 02-house-robber.diagram.json
quiz: 02-house-robber.quiz.json
practice:
  - house-robber-ii
  - coin-change
recap:
  - 01-fibonacci
follow_up:
  - "The houses are now arranged in a circle, so the first and last are adjacent — how would you adapt the recurrence?"
  - "What if you needed to also return *which* houses were robbed, not just the total?"
---

## The one idea

Where Fibonacci-style DP just adds the last two answers, house robber adds a
**decision**: at each house you either skip it (carry forward `dp[i-1]`) or
rob it (take `dp[i-2]` plus its value, because robbing means the previous
house is off-limits). `dp[i]` is the best of those two choices:
`dp[i] = max(dp[i-1], dp[i-2] + nums[i])`.

## Why it beats the obvious approach

Brute force tries every subset that avoids adjacent picks — up to 2^n
subsets to check for the adjacency constraint. But whether you should rob
house `i` never depends on *which* subset of earlier houses you picked, only
on the best totals achievable stopping at `i-1` and `i-2`. That's the
"optimal substructure + overlapping subproblems" signature of DP, so we
tabulate:

```python
def rob(nums: list[int]) -> int:
    prev2, prev1 = 0, 0          # dp[i-2], dp[i-1]
    for x in nums:
        prev2, prev1 = prev1, max(prev1, prev2 + x)
    return prev1
```

Same O(n) time as Fibonacci, and because each `dp[i]` only reads the last
two entries, we skip the array entirely and roll two variables forward —
O(1) space. The memoized top-down version
(`rob(i) = max(rob(i-1), rob(i-2) + nums[i])` behind an `@lru_cache`)
computes the identical values in a different order; tabulation is simpler to
shrink to O(1) space.

## Reading the trigger

Whenever a problem says "pick a subset that maximizes a value, but no two
picks may be adjacent," reach for this shape. The tell is a "take or skip"
choice at each position combined with a lookback of exactly two — the
Fibonacci recurrence with a `max()` wrapped around it instead of a plain
`+`. A greedy "always take the bigger one" instinct fails here: a locally
large value might block two smaller neighbors that together outscore it, so
the recurrence has to weigh both branches explicitly.
