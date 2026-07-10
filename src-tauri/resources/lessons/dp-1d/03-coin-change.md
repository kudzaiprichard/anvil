---
id: 03-coin-change
unit: dp-1d
subpattern: "Unbounded knapsack (coin change)"
trigger_signals:
  - "You need the minimum (or count of) ways to build a target total using unlimited copies of a fixed set of building blocks."
  - "Each choice can be reused any number of times — an 'unbounded' knapsack, not a 0/1 one where each item is used at most once."
  - "dp[amount] depends on dp[amount - choice] for every available choice, not just a fixed lookback of one or two indices."
worked_example: coin-change
diagram: 03-coin-change.diagram.json
quiz: 03-coin-change.quiz.json
practice:
  - min-cost-climbing-stairs
  - house-robber-ii
recap:
  - 01-fibonacci
follow_up:
  - "What if the question asked for the *number of combinations* that make the amount, rather than the fewest coins — does the loop order (coins outer vs. amount outer) start to matter?"
  - "What if coins could only be used once each (0/1, not unbounded) — how would the recurrence change?"
---

## The one idea

Coin change asks for the fewest coins that sum to a target amount, where
each coin denomination can be used **any number of times**. That "reuse
freely" property makes it an *unbounded* knapsack: `dp[a]`, the best answer
for amount `a`, is `1 + dp[a - c]` minimized over every coin `c` you're
allowed to use — and `a - c` can be revisited by a different choice of coin
later, because nothing is ever consumed.

## Why it beats the obvious approach

Trying every combination of coins that sums to the target is exponential —
for each amount you branch over every coin, and those branches constantly
overlap (many coin sequences reach the same intermediate amount). Tabulating
over amounts from 0 up removes the duplicate work: by the time we ask
"what's the best way to make `a`," every smaller amount is already solved.

```python
def coin_change(coins: list[int], amount: int) -> int:
    INF = amount + 1
    dp = [0] + [INF] * amount        # dp[a] = fewest coins to make amount a
    for a in range(1, amount + 1):
        for c in coins:
            if c <= a:
                dp[a] = min(dp[a], dp[a - c] + 1)
    return dp[amount] if dp[amount] != INF else -1
```

This is O(amount · len(coins)) time, O(amount) space. Notice the inner loop
runs over *coins*, not over a lookback of one or two indices — unlike
Fibonacci-shaped DP, `dp[a]` can depend on any `dp[a - c]`, so the full array
is needed, not just a couple of rolling variables. The same recurrence
memoizes cleanly top-down (`make(a) = 1 + min(make(a - c) for c in coins)`),
but bottom-up naturally guarantees every `dp[a - c]` is ready before `dp[a]`
needs it.

## Reading the trigger

Look for "unlimited supply" language — coins, tile pieces, ingredient units
— combined with "reach exactly this total" or "count the ways to reach this
total." If items can repeat and the target is a *sum* rather than a fixed
count of picks, you're filling a 1-D table indexed by amount, and each cell
pulls from *every* choice that fits, not a fixed lookback window.
