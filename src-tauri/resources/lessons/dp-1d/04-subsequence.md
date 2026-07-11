---
id: 04-subsequence
unit: dp-1d
subpattern: "Longest increasing subsequence"
trigger_signals:
  - "You need the longest (or best-scoring) subsequence of a sequence that keeps elements in relative order but need not be contiguous."
  - "The condition to extend depends on comparing the current element to *every* earlier element, not just its immediate neighbor."
  - "dp[i] means 'the best subsequence ending exactly at index i', and the final answer is the max over all dp[i], not dp[n-1] alone."
worked_example: longest-increasing-subsequence
diagram: 04-subsequence.diagram.json
quiz: 04-subsequence.quiz.json
practice:
  - coin-change
  - maximum-product-subarray
recap:
  - 01-fibonacci
follow_up:
  - "Could you track, for each subsequence length, the smallest possible tail value — and binary search into that instead of scanning all of dp? That's the O(n log n) patience-sorting trick."
  - "What if you needed the actual subsequence, not just its length — how would you reconstruct it from the dp table?"
---

## The one idea

`dp[i]` here means **the length of the longest increasing subsequence that
ends exactly at index i** — not "the best answer using the first i
elements." That distinction matters: to extend a subsequence ending at some
earlier `j`, we need `nums[j] < nums[i]`, so
`dp[i] = 1 + max(dp[j] for j < i if nums[j] < nums[i])`, and the final
answer is `max(dp)` over the whole table, because the longest subsequence
overall might not end at the last index.

## Why it beats the obvious approach

Brute force tries every subsequence — 2^n of them — and checks whether each
is increasing. That's wasteful because the value of extending at index `i`
only depends on which earlier index you extend *from*, not on the rest of
the subsequence's history. So for each `i`, look back over every `j < i`: if
`nums[j] < nums[i]`, index `i` could extend whatever subsequence ends at
`j`.

```python
def length_of_lis(nums: list[int]) -> int:
    dp = [1] * len(nums)                 # every element is an LIS of length 1 alone
    for i in range(len(nums)):
        for j in range(i):
            if nums[j] < nums[i]:
                dp[i] = max(dp[i], dp[j] + 1)
    return max(dp)
```

This is O(n²) time, O(n) space — a real improvement over exponential brute
force, though not the asymptotic floor. (A patience-sorting technique with
binary search reaches O(n log n), but the O(n²) table is the one to master
first: it's the direct DP recurrence, and the trigger to recognize is the
same either way.) Note the shape difference from Fibonacci and House Robber:
here `dp[i]` scans *all* earlier indices, not a fixed lookback of one or two
— closer to Coin Change's "check every valid predecessor" flavor, but keyed
by an ordering condition instead of a reachability one.

## Reading the trigger

Ask: *"do I need the best subsequence (order preserved, gaps allowed) ending
at each position, where extending requires a comparison against every
earlier element?"* Longest increasing subsequence, longest chain of pairs,
longest sequence of nested envelopes — anything phrased as "keep relative
order, skip freely, extend when a condition holds against a prior element" —
is this shape. The giveaway over House Robber is that the lookback isn't
fixed at one or two positions; it's a full scan of everything before `i`.
