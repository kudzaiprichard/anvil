---
id: 01-fibonacci
unit: dp-1d
subpattern: "Fibonacci-style 1-D DP"
trigger_signals:
  - "The problem asks to count the number of ways to reach a target (steps, tilings, paths) with no lookahead beyond a couple of previous values."
  - "The recursive definition of dp[i] only refers to dp[i-1] and/or dp[i-2] — a small, fixed window into the past."
  - "A naive recursive solution recomputes the same subproblem exponentially many times — the call tree branches the way fib(n) does."
worked_example: climbing-stairs
diagram: 01-fibonacci.diagram.json
quiz: 01-fibonacci.quiz.json
practice:
  - min-cost-climbing-stairs
  - house-robber
recap: []
follow_up:
  - "What if you only cared about the final count, not any intermediate one — could you drop the array down to two variables?"
  - "What if each step could cost 1, 2, or 3 rather than just 1 or 2 — does the recurrence change shape?"
---

## The one idea

Climbing stairs and the family it represents run on a **Fibonacci-shaped
recurrence**: the number of ways to reach step `i` is entirely determined by
the ways to reach a small, fixed set of earlier steps — here, `i-1` and
`i-2`. Once you notice the answer at `i` only needs the last one or two
answers, you can build a table bottom-up instead of re-deriving it top-down
every time.

## Why it beats the obvious approach

The natural recursive definition is `ways(i) = ways(i-1) + ways(i-2)` (you
reach step `i` either from a single step at `i-1`, or a double step at
`i-2`). Written naively, that recursion calls itself twice per level, so the
call tree branches exponentially — O(2^n) — because `ways(i-2)` gets
recomputed inside *both* `ways(i-1)` and directly, over and over.

Tabulation removes the recomputation by filling answers in order, smallest
first, so every value is computed exactly once:

```python
def climb_stairs(n: int) -> int:
    if n <= 2:
        return n
    dp = [0] * (n + 1)
    dp[1], dp[2] = 1, 2
    for i in range(3, n + 1):
        dp[i] = dp[i - 1] + dp[i - 2]   # only the last two entries matter
    return dp[n]
```

This is O(n) time, O(n) space. Because `dp[i]` only ever reads `dp[i-1]` and
`dp[i-2]`, you can also **memoize** the same recurrence top-down with an
`@lru_cache`-decorated recursive function and get identical results —
tabulation and memoization are two directions through the same table.
Tabulation is usually preferred here because it's simpler to reason about
and trivially collapses to O(1) space (two rolling variables instead of a
full array).

## Reading the trigger

Ask: *"does today's answer depend only on a small, fixed number of
yesterday's answers?"* Counting ways to climb steps, tile a strip, or step
through a sequence one or two positions at a time — anything where `dp[i]`
is a simple combination of `dp[i-1]` and `dp[i-2]` — is the Fibonacci-style
shape. The giveaway in a naive recursive solution is a call tree that
branches the way `fib(n)` does: exponential blowup from recomputing the same
small subproblems.
