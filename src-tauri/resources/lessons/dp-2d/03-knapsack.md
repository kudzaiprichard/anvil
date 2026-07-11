---
id: 03-knapsack
unit: dp-2d
subpattern: "0/1 knapsack subset-sum"
trigger_signals:
  - "You're deciding, for each item, whether to include it or not, subject to a capacity/target constraint (weight, sum, count)."
  - "The question asks \"can you partition / select a subset that hits an exact sum or capacity,\" not \"in how many orders.\""
  - "Each item can be used at most once, and using it changes the remaining capacity by a fixed amount — the signature of 0/1 knapsack."
worked_example: partition-equal-subset-sum
diagram: 03-knapsack.diagram.json
quiz: 03-knapsack.quiz.json
practice:
  - coin-change-ii
  - target-sum
recap:
  - 01-grid-paths
follow_up:
  - "What if each item could be used unlimited times instead of once — how does the recurrence and the fill order change?"
  - "Could you collapse the 2-D table to a single rolling array of size target+1, and if so, why must you iterate s from high to low?"
---

## The one idea

0/1 knapsack DP builds a 2-D table `dp[i][s]` = "using only the first `i`
items, can I hit sum (or capacity) `s`?" For every item you have exactly two
choices — skip it, or take it — and the table lets you try both without
re-deriving the same (items-so-far, remaining-capacity) state twice.

## Why it beats the obvious approach

Trying every subset of n items is 2^n. But the *decision* for item i only
depends on two numbers: how many items you've considered so far, and how
much capacity/sum remains — not which specific items you picked to get
there. That collapses the state space from 2^n subsets down to n * target
reachable (items, sum) pairs.

```python
def can_partition(nums: list[int]) -> bool:
    total = sum(nums)
    if total % 2:
        return False
    target = total // 2
    n = len(nums)
    dp = [[False] * (target + 1) for _ in range(n + 1)]
    for i in range(n + 1):
        dp[i][0] = True                     # sum 0 is always reachable (take nothing)
    for i in range(1, n + 1):
        for s in range(1, target + 1):
            dp[i][s] = dp[i - 1][s]                     # skip nums[i-1]
            if nums[i - 1] <= s:
                dp[i][s] |= dp[i - 1][s - nums[i - 1]]   # or take it
    return dp[n][target]
```

`dp[i][s]` is true if it was already reachable without item `i-1` (skip), or
if `s - nums[i-1]` was reachable with the first `i-1` items and we add this
one (take). Filling row by row (item by item) guarantees `dp[i-1][*]` is
complete before you need it for row `i`. O(n·target) time and space —
pseudo-polynomial, not exponential — and the same 2-D table shape you've
already seen twice: rows index "how far into the input," columns index "how
much of the resource is spent."

## Reading the trigger

Ask: "for each item, am I choosing include-or-exclude toward a target
sum/capacity, and is each item usable at most once?" That's 0/1 knapsack.
Compare it to the grid-path table from the first lesson: rows are still "items
considered so far," but columns are now "target remaining" instead of "column
position" — same discipline (seed the base row, fill in dependency order),
different axis meaning.
